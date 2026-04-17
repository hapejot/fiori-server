//! PostgreSQL-backed data store.
//!
//! Uses a single `entity_records` table with JSONB `data` column.
//! Records are keyed by (entity_set, key_value, is_active).
//! OData query logic ($filter, $orderby, $expand, etc.) is delegated to
//! the existing in-memory `query_collection_from` function so that
//! behaviour is identical to the in-memory backend.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::{error, info};

use crate::data_store::*;
use crate::entity::ODataEntity;
use crate::query::query_collection_from;

/// PostgreSQL-backed DataStore.
pub struct PgDataStore {
    pool: PgPool,
    entities: RwLock<Vec<&'static dyn ODataEntity>>,
    data_dir: PathBuf,
}

impl PgDataStore {
    /// Create a new PgDataStore, run migrations, and seed data from JSON/mock_data.
    pub async fn new(
        database_url: &str,
        data_dir: PathBuf,
        entities: Vec<&'static dyn ODataEntity>,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;

        // Run migration (raw_sql supports multiple statements)
        sqlx::raw_sql(include_str!("../migrations/001_create_entity_records.sql"))
            .execute(&pool)
            .await?;

        let store = Self {
            pool,
            entities: RwLock::new(entities.clone()),
            data_dir: data_dir.clone(),
        };

        // Seed data: for each entity, if the table has no rows for that set, load from JSON/mock.
        for entity in &entities {
            let set_name = entity.set_name();
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM entity_records WHERE entity_set = $1")
                    .bind(set_name)
                    .fetch_one(&store.pool)
                    .await?;

            if count.0 == 0 {
                let records = load_seed_data(set_name, &data_dir, *entity);
                info!("  [pg] Seeding {} with {} records", set_name, records.len());
                for mut record in records {
                    add_draft_defaults(&mut record);
                    let key_value = extract_key_value(&record, entity.key_field());
                    let is_active = record
                        .get("IsActiveEntity")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    sqlx::query(
                        "INSERT INTO entity_records (entity_set, key_value, is_active, data)
                         VALUES ($1, $2, $3, $4)
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(set_name)
                    .bind(&key_value)
                    .bind(is_active)
                    .bind(&record)
                    .execute(&store.pool)
                    .await?;
                }
            }
        }

        Ok(store)
    }

    fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities
            .read()
            .unwrap()
            .iter()
            .find(|e| e.set_name() == set_name)
            .copied()
    }

    fn entities_snapshot(&self) -> Vec<&'static dyn ODataEntity> {
        self.entities.read().unwrap().clone()
    }

    fn resolve_key<'a>(
        &self,
        set_name: &str,
        key: &'a EntityKey,
    ) -> Result<(&'a str, bool), StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let key_value = key.resolve_key_value(entity.key_field()).ok_or_else(|| {
            StoreError::BadRequest(format!(
                "Key field '{}' not found in key",
                entity.key_field()
            ))
        })?;
        let is_active = key.is_active();
        Ok((key_value, is_active))
    }

    // ── Blocking helpers that use tokio::task::block_in_place ────────
    // The DataStore trait is synchronous (used from sync handler code),
    // so we bridge to async via block_in_place + block_on.

    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(f))
    }

    /// Load all records for an entity set from Postgres into Vec<Value>.
    fn load_records(&self, set_name: &str) -> Vec<Value> {
        self.block_on(async {
            sqlx::query_scalar::<_, Value>("SELECT data FROM entity_records WHERE entity_set = $1")
                .bind(set_name)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default()
        })
    }

    /// Load all records as a HashMap (for expand_record).
    fn load_all_store(&self) -> HashMap<String, Vec<Value>> {
        let entities_snap = self.entities_snapshot();
        let mut store = HashMap::new();
        for entity in &entities_snap {
            store.insert(
                entity.set_name().to_string(),
                self.load_records(entity.set_name()),
            );
        }
        store
    }

    /// Fetch a single record.
    fn fetch_record(&self, set_name: &str, key_value: &str, is_active: bool) -> Option<Value> {
        self.block_on(async {
            sqlx::query_scalar::<_, Value>(
                "SELECT data FROM entity_records
                 WHERE entity_set = $1 AND key_value = $2 AND is_active = $3",
            )
            .bind(set_name)
            .bind(key_value)
            .bind(is_active)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
        })
    }

    /// Upsert a record.
    #[tracing::instrument(skip(self), set_name, key_value, is_active, data)]
    fn upsert_record(&self, set_name: &str, key_value: &str, is_active: bool, data: &Value) {
        info!("upsert");
        self.block_on(async {
            sqlx::query(
                "INSERT INTO entity_records (entity_set, key_value, is_active, data, updated_at)
                 VALUES ($1, $2, $3, $4, now())
                 ON CONFLICT (entity_set, key_value, is_active) DO UPDATE
                 SET data = $4, updated_at = now()",
            )
            .bind(set_name)
            .bind(key_value)
            .bind(is_active)
            .bind(data)
            .execute(&self.pool)
            .await
            .ok();
        })
    }

    /// Delete a record.
    fn delete_record(&self, set_name: &str, key_value: &str, is_active: bool) {
        self.block_on(async {
            sqlx::query(
                "DELETE FROM entity_records
                 WHERE entity_set = $1 AND key_value = $2 AND is_active = $3",
            )
            .bind(set_name)
            .bind(key_value)
            .bind(is_active)
            .execute(&self.pool)
            .await
            .ok();
        })
    }
}

impl DataStore for PgDataStore {
    fn get_collection(
        &self,
        set_name: &str,
        query: &ODataQuery,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let entities_snap = self.entities_snapshot();
        let all_store = self.load_all_store();
        let qs = query.to_query_map();

        match parent {
            Some(parent_ref) => {
                let parent_entity = self.find_entity(&parent_ref.set_name).ok_or_else(|| {
                    StoreError::NotFound(format!(
                        "Parent entity set '{}' not found",
                        parent_ref.set_name
                    ))
                })?;
                let parent_key_field = parent_entity.key_field();
                let parent_key_value = parent_ref
                    .key
                    .resolve_key_value(parent_key_field)
                    .ok_or_else(|| {
                        StoreError::BadRequest("Parent key value not found".to_string())
                    })?;
                let parent_is_active = parent_ref.key.is_active();

                let child_fk = parent_entity
                    .navigation_properties()
                    .iter()
                    .find(|np| np.target_type == entity.type_name())
                    .and_then(|np| np.foreign_key)
                    .unwrap_or(parent_key_field);

                let records = all_store.get(set_name).cloned().unwrap_or_default();
                let child_records: Vec<Value> = records
                    .into_iter()
                    .filter(|r| {
                        r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                == Some(parent_is_active)
                    })
                    .collect();
                Ok(query_collection_from(
                    entity,
                    &child_records,
                    &qs,
                    &entities_snap,
                    &all_store,
                ))
            }
            None => {
                let records = all_store.get(set_name).cloned().unwrap_or_default();
                Ok(query_collection_from(
                    entity,
                    &records,
                    &qs,
                    &entities_snap,
                    &all_store,
                ))
            }
        }
    }

    fn count(&self, set_name: &str, _query: &ODataQuery, parent: Option<&ParentKey>) -> usize {
        match parent {
            Some(parent_ref) => {
                let parent_entity = match self.find_entity(&parent_ref.set_name) {
                    Some(e) => e,
                    None => return 0,
                };
                let parent_key_field = parent_entity.key_field();
                let parent_key_value = match parent_ref.key.resolve_key_value(parent_key_field) {
                    Some(v) => v,
                    None => return 0,
                };
                let parent_is_active = parent_ref.key.is_active();
                let records = self.load_records(set_name);
                records
                    .iter()
                    .filter(|r| {
                        r.get(parent_key_field).and_then(|v| v.as_str()) == Some(parent_key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                == Some(parent_is_active)
                    })
                    .count()
            }
            None => self.block_on(async {
                let row: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM entity_records WHERE entity_set = $1")
                        .bind(set_name)
                        .fetch_one(&self.pool)
                        .await
                        .unwrap_or((0,));
                row.0 as usize
            }),
        }
    }

    fn read_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        query: &ODataQuery,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();
        let qs = query.to_query_map();

        let record = self
            .fetch_record(set_name, key_value, is_active)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        let mut result = record;
        inject_odata_context(&mut result, set_name);

        if let Some(expand_str) = qs.get("$expand") {
            if !expand_str.is_empty() {
                let all_store = self.load_all_store();
                let nav_names: Vec<String> = query
                    .expand
                    .iter()
                    .map(|e| e.nav_property.clone())
                    .collect();
                let nav_refs: Vec<&str> = nav_names.iter().map(|s| s.as_str()).collect();
                entity.expand_record(&mut result, &nav_refs, &entities_snap, &all_store);
                if nav_refs.iter().any(|n| *n == "DraftAdministrativeData") {
                    inject_draft_admin_data(&mut result, entity.key_field());
                }
                if nav_refs.iter().any(|n| *n == "SiblingEntity") {
                    let records = all_store.get(set_name).cloned().unwrap_or_default();
                    inject_sibling_entity(&mut result, entity.key_field(), &records);
                }
            }
        }
        Ok(result)
    }

    fn create_entity(
        &self,
        set_name: &str,
        data: &Value,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        let mut new_record = data.clone();
        if let Some(obj) = new_record.as_object_mut() {
            if let Some(parent_ref) = parent {
                if let Some(parent_entity) = self.find_entity(&parent_ref.set_name) {
                    let parent_key_value = parent_ref
                        .key
                        .resolve_key_value(parent_entity.key_field())
                        .unwrap_or("");
                    let child_fk = resolve_child_fk(parent_entity, entity);
                    obj.entry(child_fk.to_string())
                        .or_insert_with(|| json!(parent_key_value));
                }
            }

            let key_field = entity.key_field();
            if !obj.contains_key(key_field) {
                obj.insert(
                    key_field.to_string(),
                    json!(uuid::Uuid::new_v4().to_string()),
                );
            }

            obj.insert("IsActiveEntity".to_string(), json!(false));
            obj.insert("HasActiveEntity".to_string(), json!(false));
            obj.insert("HasDraftEntity".to_string(), json!(false));

            // Entity-specific default values (e.g. Currency="EUR", Status="A")
            if let Some(defaults) = entity.default_values() {
                if let Some(def_obj) = defaults.as_object() {
                    for (k, v) in def_obj {
                        obj.entry(k.clone()).or_insert(v.clone());
                    }
                }
            }

            if let Some(fields) = entity.fields_def() {
                for f in fields {
                    obj.entry(f.name.to_string())
                        .or_insert_with(|| match f.edm_type {
                            "Edm.Int32" | "Edm.Byte" => json!(0),
                            "Edm.Decimal" => json!("0"),
                            "Edm.Boolean" => json!(false),
                            _ => json!(""),
                        });
                }
            }
        }

        let key_value = extract_key_value(&new_record, entity.key_field());
        let is_active = new_record
            .get("IsActiveEntity")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut result = new_record.clone();
        inject_odata_context(&mut result, set_name);
        self.upsert_record(set_name, &key_value, is_active, &new_record);

        Ok(result)
    }

    fn patch_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        patch: &Value,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;

        let mut record = self
            .fetch_record(set_name, key_value, is_active)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        let readonly_fields: Vec<&str> = entity
            .fields_def()
            .unwrap_or(&[])
            .iter()
            .filter(|f| f.immutable || f.computed)
            .map(|f| f.name)
            .collect();

        if let Some(patch_obj) = patch.as_object() {
            if let Some(rec_obj) = record.as_object_mut() {
                for (k, v) in patch_obj {
                    if is_draft_field(k) || readonly_fields.contains(&k.as_str()) {
                        continue;
                    }
                    rec_obj.insert(k.clone(), v.clone());
                }
            }
        }

        self.upsert_record(set_name, key_value, is_active, &record);

        let mut result = record;
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();

        self.fetch_record(set_name, key_value, is_active)
            .ok_or_else(|| StoreError::NotFound("Entity not found".to_string()))?;

        self.delete_record(set_name, key_value, is_active);

        if !is_active {
            // Clear HasDraftEntity on active record
            if let Some(mut active) = self.fetch_record(set_name, key_value, true) {
                if let Some(obj) = active.as_object_mut() {
                    obj.insert("HasDraftEntity".to_string(), json!(false));
                }
                self.upsert_record(set_name, key_value, true, &active);
            }
            // Remove child drafts
            for child in &entities_snap {
                if child.parent_set_name() != Some(entity.set_name()) {
                    continue;
                }
                let child_fk = resolve_child_fk(entity, *child);
                let child_records = self.load_records(child.set_name());
                for r in &child_records {
                    if r.get(child_fk).and_then(|v| v.as_str()) == Some(key_value)
                        && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false)
                    {
                        let child_key = extract_key_value(r, child.key_field());
                        self.delete_record(child.set_name(), &child_key, false);
                    }
                }
            }
        }

        Ok(())
    }

    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, _) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();

        let active = self
            .fetch_record(set_name, key_value, true)
            .ok_or_else(|| StoreError::NotFound("Active entity not found".to_string()))?;

        // Mark active as having a draft
        let mut updated_active = active.clone();
        if let Some(obj) = updated_active.as_object_mut() {
            obj.insert("HasDraftEntity".to_string(), json!(true));
        }
        self.upsert_record(set_name, key_value, true, &updated_active);

        // Create draft copy
        let mut draft_rec = active;
        inject_draft_flags(&mut draft_rec, false, true, false);
        inject_odata_context(&mut draft_rec, set_name);
        let result = draft_rec.clone();
        self.upsert_record(set_name, key_value, false, &draft_rec);

        // Copy children as drafts
        for child in &entities_snap {
            if child.parent_set_name() != Some(entity.set_name()) {
                continue;
            }
            let child_fk = resolve_child_fk(entity, *child);
            let child_records = self.load_records(child.set_name());
            for r in &child_records {
                if r.get(child_fk).and_then(|v| v.as_str()) == Some(key_value)
                    && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(true)
                {
                    let mut d = r.clone();
                    inject_draft_flags(&mut d, false, true, false);
                    let child_key = extract_key_value(&d, child.key_field());
                    self.upsert_record(child.set_name(), &child_key, false, &d);
                }
            }
        }

        Ok(result)
    }

    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, _) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();

        let draft_rec = self
            .fetch_record(set_name, key_value, false)
            .ok_or_else(|| StoreError::NotFound("Draft not found".to_string()))?;

        let has_active = draft_rec
            .get("HasActiveEntity")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if has_active {
            // Merge draft into active
            if let Some(mut active) = self.fetch_record(set_name, key_value, true) {
                if let (Some(active_obj), Some(draft_obj)) =
                    (active.as_object_mut(), draft_rec.as_object())
                {
                    for (k, v) in draft_obj {
                        if !is_draft_field(k) && !k.starts_with("@odata") {
                            active_obj.insert(k.clone(), v.clone());
                        }
                    }
                    active_obj.insert("HasDraftEntity".to_string(), json!(false));
                }
                self.upsert_record(set_name, key_value, true, &active);
            }
        } else {
            // New entity: promote draft to active
            let mut new_active = draft_rec.clone();
            inject_draft_flags(&mut new_active, true, false, false);
            self.upsert_record(set_name, key_value, true, &new_active);
        }

        // Remove draft
        self.delete_record(set_name, key_value, false);

        // Activate children
        for child in &entities_snap {
            if child.parent_set_name() != Some(entity.set_name()) {
                continue;
            }
            let child_fk = resolve_child_fk(entity, *child);
            let child_records = self.load_records(child.set_name());

            // Remove all active children for this parent
            for r in &child_records {
                if r.get(child_fk).and_then(|v| v.as_str()) == Some(key_value)
                    && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(true)
                {
                    let child_key = extract_key_value(r, child.key_field());
                    self.delete_record(child.set_name(), &child_key, true);
                }
            }

            // Promote draft children to active
            for r in &child_records {
                if r.get(child_fk).and_then(|v| v.as_str()) == Some(key_value)
                    && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false)
                {
                    let mut item = r.clone();
                    inject_draft_flags(&mut item, true, false, false);
                    let child_key = extract_key_value(&item, child.key_field());
                    self.upsert_record(child.set_name(), &child_key, true, &item);
                    self.delete_record(child.set_name(), &child_key, false);
                }
            }
        }

        let result = self.fetch_record(set_name, key_value, true);
        match result {
            Some(mut r) => {
                inject_odata_context(&mut r, set_name);
                Ok(r)
            }
            None => Err(StoreError::NotFound(
                "Activated entity not found".to_string(),
            )),
        }
    }

    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;

        let record = self
            .fetch_record(set_name, key_value, is_active)
            .ok_or_else(|| StoreError::NotFound("Entity not found for draftPrepare".to_string()))?;

        let mut result = record;
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    fn read_sibling_entity(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;

        let sibling = self
            .fetch_record(set_name, key_value, !is_active)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Sibling entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        let mut result = sibling;
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    fn get_property(
        &self,
        set_name: &str,
        key: &EntityKey,
        property: &str,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;

        let record = self
            .fetch_record(set_name, key_value, is_active)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        record
            .get(property)
            .cloned()
            .ok_or_else(|| StoreError::NotFound(format!("Property '{}' not found", property)))
    }

    fn get_records(&self, set_name: &str) -> Vec<Value> {
        self.load_records(set_name)
    }

    fn commit(&self) {
        // For Postgres, data is already persisted on each write.
        // We also write JSON files for compatibility with activate_config.
        info!("  [pg commit] Data already persisted in Postgres");
        let entities_snap = self.entities_snapshot();
        for entity in &entities_snap {
            let set_name = entity.set_name();
            let records = self.load_records(set_name);
            let active: Vec<&Value> = records
                .iter()
                .filter(|r| {
                    r.get("IsActiveEntity")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                })
                .collect();
            let clean: Vec<Value> = active
                .into_iter()
                .map(|r| {
                    let mut c = r.clone();
                    if let Some(obj) = c.as_object_mut() {
                        obj.remove("IsActiveEntity");
                        obj.remove("HasActiveEntity");
                        obj.remove("HasDraftEntity");
                    }
                    c
                })
                .collect();
            let json_path = self.data_dir.join(format!("{}.json", set_name));
            if let Ok(content) = serde_json::to_string_pretty(&clean) {
                if let Err(e) = std::fs::write(&json_path, content) {
                    error!("  WARNING: Could not write {}: {}", json_path.display(), e);
                }
            }
        }
    }

    fn update_entities(&self, new_entities: Vec<&'static dyn ODataEntity>) {
        // Seed any new entity sets that don't have data yet
        for entity in &new_entities {
            let set_name = entity.set_name();
            let count: usize = self.block_on(async {
                let row: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM entity_records WHERE entity_set = $1")
                        .bind(set_name)
                        .fetch_one(&self.pool)
                        .await
                        .unwrap_or((0,));
                row.0 as usize
            });

            if count == 0 {
                let records = load_seed_data(set_name, &self.data_dir, *entity);
                info!(
                    "  [pg] Seeding new entity {} with {} records",
                    set_name,
                    records.len()
                );
                for mut record in records {
                    add_draft_defaults(&mut record);
                    let key_value = extract_key_value(&record, entity.key_field());
                    let is_active = record
                        .get("IsActiveEntity")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    self.upsert_record(set_name, &key_value, is_active, &record);
                }
            }
        }
        *self.entities.write().unwrap() = new_entities;
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn add_draft_defaults(record: &mut Value) {
    if let Some(obj) = record.as_object_mut() {
        obj.entry("IsActiveEntity".to_string())
            .or_insert(Value::Bool(true));
        obj.entry("HasActiveEntity".to_string())
            .or_insert(Value::Bool(false));
        obj.entry("HasDraftEntity".to_string())
            .or_insert(Value::Bool(false));
    }
}

fn extract_key_value(record: &Value, key_field: &str) -> String {
    record
        .get(key_field)
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn load_seed_data(
    set_name: &str,
    data_dir: &std::path::Path,
    entity: &dyn ODataEntity,
) -> Vec<Value> {
    let json_path = data_dir.join(format!("{}.json", set_name));
    if json_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(records) = serde_json::from_str::<Vec<Value>>(&content) {
                info!(
                    "  [pg] {} : {} records from {}",
                    set_name,
                    records.len(),
                    json_path.display()
                );
                return records;
            }
        }
    }
    info!("  [pg] {} : mock_data()", set_name);
    entity.mock_data()
}
