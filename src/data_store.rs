use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::{json, Value};
use tower_http::trace;
use tracing::{error, info};
use uuid::Uuid;

use crate::entity::ODataEntity;
use crate::query::query_collection_from;
use crate::BASE_PATH;

// ── Low-level record helpers (formerly draft.rs) ────────────────────

/// Find a record by key field + IsActiveEntity.
fn find_record<'a>(
    records: &'a [Value],
    key_field: &str,
    key_value: &str,
    is_active: bool,
) -> Option<&'a Value> {
    records.iter().find(|r| {
        r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active)
    })
}

/// Find a record mutably by key field + IsActiveEntity.
fn find_record_mut<'a>(
    records: &'a mut [Value],
    key_field: &str,
    key_value: &str,
    is_active: bool,
) -> Option<&'a mut Value> {
    records.iter_mut().find(|r| {
        r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active)
    })
}

/// Remove all records matching key field + IsActiveEntity.
fn remove_records(records: &mut Vec<Value>, key_field: &str, key_value: &str, is_active: bool) {
    records.retain(|r| {
        !(r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active))
    });
}

/// Set the three draft boolean flags on a record.
pub(crate) fn set_draft_flags(record: &mut Value, is_active: bool, has_active: bool, has_draft: bool) {
    if let Some(obj) = record.as_object_mut() {
        obj.insert("IsActiveEntity".to_string(), json!(is_active));
        obj.insert("HasActiveEntity".to_string(), json!(has_active));
        obj.insert("HasDraftEntity".to_string(), json!(has_draft));
    }
}

/// Inject @odata.context into a record.
pub(crate) fn inject_odata_context(record: &mut Value, set_name: &str) {
    if let Some(obj) = record.as_object_mut() {
        obj.insert(
            "@odata.context".to_string(),
            json!(format!("{}/$metadata#{}/$entity", BASE_PATH, set_name)),
        );
    }
}

// ── EntityKey ───────────────────────────────────────────────────────

/// Composite key identifying one entity record.
/// Constructed from OData URL parentheses: Entity(Key1='val1',Key2='val2')
#[derive(Debug, Clone)]
pub struct EntityKey {
    pairs: Vec<(String, String)>,
}

impl EntityKey {
    /// Single key: Products('P001')
    pub fn single(field: &str, value: &str) -> Self {
        Self {
            pairs: vec![(field.to_string(), value.to_string())],
        }
    }

    /// Composite key from slice: &[("OrderID", "O001"), ("IsActiveEntity", "true")]
    pub fn composite(pairs: &[(&str, &str)]) -> Self {
        Self {
            pairs: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    /// Parse from OData URL key segment: "OrderID='O001',IsActiveEntity=true"
    /// Also handles simple keys: "'P001'"
    pub fn parse(segment: &str) -> Self {
        let segment = segment.trim();
        // Simple key: 'value' — needs a key field name from context,
        // so we store it as ("_key", value)
        if segment.starts_with('\'') && segment.ends_with('\'') {
            let value = segment[1..segment.len() - 1].to_string();
            return Self {
                pairs: vec![("_key".to_string(), value)],
            };
        }
        // Composite key: Key='val',IsActiveEntity=true
        let mut pairs = Vec::new();
        for part in segment.split(',') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim().to_string();
                let v = v.trim().trim_matches('\'').to_string();
                pairs.push((k, v));
            }
        }
        Self { pairs }
    }

    /// All key-value pairs.
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// Get value for a specific key field.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == field)
            .map(|(_, v)| v.as_str())
    }

    /// Resolve the primary key value using the entity's key_field.
    /// Handles both named keys and simple '_key' placeholder.
    pub fn resolve_key_value(&self, key_field: &str) -> Option<&str> {
        self.get(key_field).or_else(|| self.get("_key"))
    }

    /// Get IsActiveEntity from composite key, defaults to true.
    pub fn is_active(&self) -> bool {
        self.get("IsActiveEntity")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(true)
    }
}

// ── ParentKey ───────────────────────────────────────────────────────

/// Parent context for sub-collection / deep navigation.
#[derive(Debug, Clone)]
pub struct ParentKey {
    pub set_name: String,
    pub key: EntityKey,
}

impl ParentKey {
    pub fn new(set_name: &str, key: EntityKey) -> Self {
        Self {
            set_name: set_name.to_string(),
            key,
        }
    }
}

// ── ODataQuery ──────────────────────────────────────────────────────

/// Structured OData query parameters.
#[derive(Debug, Clone, Default)]
pub struct ODataQuery {
    pub filter: Option<String>,
    pub select: Vec<String>,
    pub expand: Vec<ExpandClause>,
    pub orderby: Option<OrderByClause>,
    pub top: Option<usize>,
    pub skip: Option<usize>,
    pub count: bool,
}

#[derive(Debug, Clone)]
pub struct ExpandClause {
    pub nav_property: String,
    pub select: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OrderByClause {
    pub field: String,
    pub descending: bool,
}

impl ODataQuery {
    /// Parse from raw URL query string: "$filter=Status eq 'A'&$top=10&..."
    pub fn parse(query_str: &str) -> Self {
        let mut q = Self::empty();
        if query_str.is_empty() {
            return q;
        }
        for pair in query_str.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let k = urlencoding::decode(k).unwrap_or_default().into_owned();
                let v = urlencoding::decode(v).unwrap_or_default().into_owned();
                match k.as_str() {
                    "$filter" => q.filter = Some(v),
                    "$select" => {
                        q.select = v.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "$expand" => {
                        q.expand = Self::parse_expand(&v);
                    }
                    "$orderby" => {
                        let parts: Vec<&str> = v.split_whitespace().collect();
                        if let Some(field) = parts.first() {
                            let descending = parts
                                .get(1)
                                .map(|s| s.eq_ignore_ascii_case("desc"))
                                .unwrap_or(false);
                            q.orderby = Some(OrderByClause {
                                field: field.to_string(),
                                descending,
                            });
                        }
                    }
                    "$top" => q.top = v.parse().ok(),
                    "$skip" => q.skip = v.parse().ok(),
                    "$count" => q.count = v.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }
        q
    }

    pub fn empty() -> Self {
        Self::default()
    }

    /// Convert back to the HashMap<String, String> format that query.rs expects.
    pub fn to_query_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(ref filter) = self.filter {
            map.insert("$filter".to_string(), filter.clone());
        }
        if !self.select.is_empty() {
            map.insert("$select".to_string(), self.select.join(","));
        }
        if !self.expand.is_empty() {
            let expand_str = self
                .expand
                .iter()
                .map(|e| {
                    if e.select.is_empty() {
                        e.nav_property.clone()
                    } else {
                        format!("{}($select={})", e.nav_property, e.select.join(","))
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            map.insert("$expand".to_string(), expand_str);
        }
        if let Some(ref orderby) = self.orderby {
            let dir = if orderby.descending { " desc" } else { "" };
            map.insert("$orderby".to_string(), format!("{}{}", orderby.field, dir));
        }
        if let Some(top) = self.top {
            map.insert("$top".to_string(), top.to_string());
        }
        if let Some(skip) = self.skip {
            map.insert("$skip".to_string(), skip.to_string());
        }
        if self.count {
            map.insert("$count".to_string(), "true".to_string());
        }
        map
    }

    /// Parse $expand value, extracting nav property names and nested $select.
    fn parse_expand(expand: &str) -> Vec<ExpandClause> {
        let mut result = Vec::new();
        let mut depth = 0;
        let mut current = String::new();
        let mut nested = String::new();
        let mut in_nested = false;

        for ch in expand.chars() {
            match ch {
                '(' if depth == 0 => {
                    depth += 1;
                    in_nested = true;
                }
                '(' => {
                    depth += 1;
                    nested.push(ch);
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        in_nested = false;
                    } else {
                        nested.push(ch);
                    }
                }
                ',' if depth == 0 => {
                    let nav_property = current.trim().to_string();
                    if !nav_property.is_empty() {
                        result.push(ExpandClause {
                            nav_property,
                            select: Self::parse_nested_select(&nested),
                        });
                    }
                    current.clear();
                    nested.clear();
                }
                _ if in_nested => {
                    nested.push(ch);
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        let nav_property = current.trim().to_string();
        if !nav_property.is_empty() {
            result.push(ExpandClause {
                nav_property,
                select: Self::parse_nested_select(&nested),
            });
        }
        result
    }

    /// Parse nested options like "$select=DraftUUID,InProcessByUser"
    fn parse_nested_select(nested: &str) -> Vec<String> {
        for part in nested.split('&') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("$select=") {
                return val.split(',').map(|s| s.trim().to_string()).collect();
            }
        }
        Vec::new()
    }
}

// ── StoreError ──────────────────────────────────────────────────────

/// Domain errors for data store operations.
#[derive(Debug)]
pub enum StoreError {
    NotFound(String),
    BadRequest(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound(msg) => write!(f, "Not found: {}", msg),
            StoreError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

// ── DataStore Trait ─────────────────────────────────────────────────

/// Trait for data storage backends.
/// All locking/transaction management is internal to the implementation.
/// Callers pass only string identifiers, structured query types, and JSON values.
pub trait DataStore: Send + Sync {
    // ── Collections ──
    fn get_collection(
        &self,
        set_name: &str,
        query: &ODataQuery,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError>;

    fn count(&self, set_name: &str, query: &ODataQuery, parent: Option<&ParentKey>) -> usize;

    // ── Single Entity CRUD ──
    fn read_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        query: &ODataQuery,
    ) -> Result<Value, StoreError>;

    fn create_entity(
        &self,
        set_name: &str,
        data: &Value,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError>;

    fn patch_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        patch: &Value,
    ) -> Result<Value, StoreError>;

    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError>;

    // ── Draft Actions ──
    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    // ── Sibling Entity (draft ↔ active) ──
    fn read_sibling_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
    ) -> Result<Value, StoreError>;

    // ── Property / Ad-hoc ──
    fn get_property(
        &self,
        set_name: &str,
        key: &EntityKey,
        property: &str,
    ) -> Result<Value, StoreError>;

    fn get_records(&self, set_name: &str) -> Vec<Value>;

    // ── Persistence ──
    fn commit(&self);

    // ── Entity Updates ──
    fn update_entities(&self, entities: Vec<&'static dyn ODataEntity>);
}

// ── InMemoryDataStore ───────────────────────────────────────────────

/// In-memory data store backed by JSON files.
/// Loads all data on initialization, persists on commit().
pub struct InMemoryDataStore {
    store: RwLock<HashMap<String, Vec<Value>>>,
    entities: RwLock<Vec<&'static dyn ODataEntity>>,
    data_dir: PathBuf,
}

impl InMemoryDataStore {
    /// Create a new in-memory store, loading data from JSON files.
    pub fn new(data_dir: PathBuf, entities: Vec<&'static dyn ODataEntity>) -> Self {
        let mut store = HashMap::new();
        for entity in &entities {
            let set_name = entity.set_name();
            let mut records = load_entity_data(set_name, &data_dir, *entity);
            for record in &mut records {
                if let Some(obj) = record.as_object_mut() {
                    obj.entry("IsActiveEntity".to_string())
                        .or_insert(Value::Bool(true));
                    obj.entry("HasActiveEntity".to_string())
                        .or_insert(Value::Bool(false));
                    obj.entry("HasDraftEntity".to_string())
                        .or_insert(Value::Bool(false));
                }
            }
            store.insert(set_name.to_string(), records);
        }

        Self {
            store: RwLock::new(store),
            entities: RwLock::new(entities),
            data_dir,
        }
    }

    #[tracing::instrument(skip(self))]
    fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities
            .read()
            .unwrap()
            .iter()
            .find(|e| e.set_name() == set_name)
            .copied()
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    fn entities_snapshot(&self) -> Vec<&'static dyn ODataEntity> {
        self.entities.read().unwrap().clone()
    }

    fn record_count(&self, set_name: &str) -> usize {
        self.store
            .read()
            .unwrap()
            .get(set_name)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl DataStore for InMemoryDataStore {
    #[tracing::instrument(skip(self))]
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
        let store = self.store.read().unwrap();
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

                // Fremdschluessel-Feld auf dem Kind ermitteln:
                // NavigationProperty.foreign_key hat Vorrang, sonst parent_key_field.
                let child_fk = parent_entity
                    .navigation_properties()
                    .iter()
                    .find(|np| np.target_type == entity.type_name())
                    .and_then(|np| np.foreign_key)
                    .unwrap_or(parent_key_field);

                let child_records: Vec<Value> = store
                    .get(set_name)
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| {
                                r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                                    && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                        == Some(parent_is_active)
                            })
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(query_collection_from(
                    entity,
                    &child_records,
                    &qs,
                    &entities_snap,
                    &store,
                ))
            }
            None => {
                let records = store.get(set_name);
                match records {
                    Some(data) => Ok(query_collection_from(
                        entity,
                        data,
                        &qs,
                        &entities_snap,
                        &store,
                    )),
                    None => Ok(query_collection_from(
                        entity,
                        &entity.mock_data(),
                        &qs,
                        &entities_snap,
                        &store,
                    )),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn count(&self, set_name: &str, _query: &ODataQuery, parent: Option<&ParentKey>) -> usize {
        let store = self.store.read().unwrap();

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

                store
                    .get(set_name)
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| {
                                r.get(parent_key_field).and_then(|v| v.as_str())
                                    == Some(parent_key_value)
                                    && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                        == Some(parent_is_active)
                            })
                            .count()
                    })
                    .unwrap_or(0)
            }
            None => store.get(set_name).map(|v| v.len()).unwrap_or(0),
        }
    }

    #[tracing::instrument(skip(self))]
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
        let store = self.store.read().unwrap();
        let qs = query.to_query_map();

        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let record =
            find_record(records, entity.key_field(), key_value, is_active).ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        let mut result = record.clone();
        inject_odata_context(&mut result, set_name);

        if let Some(expand_str) = qs.get("$expand") {
            if !expand_str.is_empty() {
                let nav_names: Vec<String> = query
                    .expand
                    .iter()
                    .map(|e| e.nav_property.clone())
                    .collect();
                let nav_refs: Vec<&str> = nav_names.iter().map(|s| s.as_str()).collect();
                entity.expand_record(&mut result, &nav_refs, &entities_snap, &store);
                if nav_refs.iter().any(|n| *n == "DraftAdministrativeData") {
                    inject_draft_admin_data(&mut result, entity.key_field());
                }
                if nav_refs.iter().any(|n| *n == "SiblingEntity") {
                    inject_sibling_entity(&mut result, entity.key_field(), records);
                }
            }
        }
        // Resolve value_source text fields
        resolve_value_texts(entity, &mut result, &store);
        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    fn create_entity(
        &self,
        set_name: &str,
        data: &Value,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let mut store = self.store.write().unwrap();

        let mut new_record = data.clone();
        if let Some(obj) = new_record.as_object_mut() {
            // Inject parent key if sub-item
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

            // Generate key if not present
            let key_field = entity.key_field();
            if !obj.contains_key(key_field) {
                obj.insert(key_field.to_string(), json!(Uuid::new_v4().to_string()));
            }

            // Draft flags
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

            // Default values for missing fields
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

        let mut result = new_record.clone();
        inject_odata_context(&mut result, set_name);
        store
            .entry(set_name.to_string())
            .or_insert_with(Vec::new)
            .push(new_record);

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
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
        let mut store = self.store.write().unwrap();

        let records = store
            .get_mut(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let record = find_record_mut(records, entity.key_field(), key_value, is_active)
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

        let mut result = record.clone();
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();
        let mut store = self.store.write().unwrap();

        let found = store
            .get(set_name)
            .map(|r| find_record(r, entity.key_field(), key_value, is_active).is_some())
            .unwrap_or(false);

        if !found {
            return Err(StoreError::NotFound("Entity not found".to_string()));
        }

        if let Some(records) = store.get_mut(set_name) {
            remove_records(records, entity.key_field(), key_value, is_active);

            if !is_active {
                if let Some(active) = find_record_mut(records, entity.key_field(), key_value, true)
                {
                    if let Some(obj) = active.as_object_mut() {
                        obj.insert("HasDraftEntity".to_string(), json!(false));
                    }
                }
                // Remove child drafts
                remove_child_drafts(&mut store, entity, key_value, &entities_snap);
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        info!("start");
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, _) = self.resolve_key(set_name, key)?;
        info!("key: {}", key_value);
        let entities_snap = self.entities_snapshot();
        let mut store = self.store.write().unwrap();

        let records = store
            .get_mut(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        let active = find_record(records, entity.key_field(), key_value, true)
            .ok_or_else(|| StoreError::NotFound("Active entity not found".to_string()))?
            .clone();
        info!("active record: {}", active);
        // Mark active as having a draft
        if let Some(active_rec) = find_record_mut(records, entity.key_field(), key_value, true) {
            if let Some(obj) = active_rec.as_object_mut() {
                obj.insert("HasDraftEntity".to_string(), json!(true));
            } else {
                panic!("active record not found for draft edit");
            }
        }

        // Create draft copy
        let mut draft_rec = active;
        set_draft_flags(&mut draft_rec, false, true, false);
        inject_odata_context(&mut draft_rec, set_name);
        let result = draft_rec.clone();
        records.push(draft_rec);
        info!("records: {}", records.len());
        // Copy children as drafts
        copy_children_as_drafts(&mut store, entity, key_value, &entities_snap);

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, _) = self.resolve_key(set_name, key)?;
        let entities_snap = self.entities_snapshot();
        let mut store = self.store.write().unwrap();

        let records = store
            .get_mut(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        info!(
            "  [draftActivate] {}('{}') – activating draft",
            set_name, key_value
        );

        let draft_rec = find_record(records, entity.key_field(), key_value, false)
            .ok_or_else(|| StoreError::NotFound("Draft not found".to_string()))?
            .clone();
        info!("draft record: {}", draft_rec);
        let has_active = draft_rec
            .get("HasActiveEntity")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        info!("has active: {}", has_active);
        if has_active {
            if let Some(active) = find_record_mut(records, entity.key_field(), key_value, true) {
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
            }
        } else {
            let mut new_active = draft_rec.clone();
            set_draft_flags(&mut new_active, true, false, false);
            records.push(new_active);
        }

        // Remove draft
        remove_records(records, entity.key_field(), key_value, false);

        let result = find_record(records, entity.key_field(), key_value, true).cloned();

        // Activate children
        activate_children(&mut store, entity, key_value, &entities_snap);

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

    #[tracing::instrument(skip(self))]
    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let store = self.store.read().unwrap();

        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        info!(
            "  [draftPrepare] {}('{}') is_active={}",
            set_name, key_value, is_active
        );

        let record = find_record(records, entity.key_field(), key_value, is_active)
            .ok_or_else(|| StoreError::NotFound("Entity not found for draftPrepare".to_string()))?;

        let mut result = record.clone();
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    fn read_sibling_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let store = self.store.read().unwrap();

        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        // The sibling is the same key with flipped IsActiveEntity
        let sibling = find_record(records, entity.key_field(), key_value, !is_active)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Sibling entity with {}='{}' not found",
                    entity.key_field(),
                    key_value
                ))
            })?;

        let mut result = sibling.clone();
        inject_odata_context(&mut result, set_name);
        Ok(result)
    }

    #[tracing::instrument(skip(self))]
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
        let store = self.store.read().unwrap();

        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        let record =
            find_record(records, entity.key_field(), key_value, is_active).ok_or_else(|| {
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

    #[tracing::instrument(skip(self))]
    fn get_records(&self, set_name: &str) -> Vec<Value> {
        let store = self.store.read().unwrap();
        store.get(set_name).cloned().unwrap_or_default()
    }

    #[tracing::instrument(skip(self))]
    fn commit(&self) {
        info!("  [commit] Persisting data to {}", self.data_dir.display());
        let entities_snap = self.entities_snapshot();
        let store = self.store.read().unwrap();
        for entity in &entities_snap {
            let set_name = entity.set_name();
            if let Some(records) = store.get(set_name) {
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
                        eprintln!("  WARNING: Could not write {}: {}", json_path.display(), e);
                    }
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn update_entities(&self, new_entities: Vec<&'static dyn ODataEntity>) {
        // Register any new entity sets that don't have data yet
        let mut store = self.store.write().unwrap();
        for entity in &new_entities {
            let set_name = entity.set_name();
            if !store.contains_key(set_name) {
                // Load data from disk for newly added entities
                let mut records = load_entity_data(set_name, &self.data_dir, *entity);
                for record in &mut records {
                    if let Some(obj) = record.as_object_mut() {
                        obj.entry("IsActiveEntity".to_string())
                            .or_insert(Value::Bool(true));
                        obj.entry("HasActiveEntity".to_string())
                            .or_insert(Value::Bool(false));
                        obj.entry("HasDraftEntity".to_string())
                            .or_insert(Value::Bool(false));
                    }
                }
                store.insert(set_name.to_string(), records);
            }
        }
        drop(store);
        *self.entities.write().unwrap() = new_entities;
    }
}

// ── Internal helpers (moved from draft.rs for InMemoryDataStore) ────

pub(crate) fn is_draft_field(k: &str) -> bool {
    k == "IsActiveEntity" || k == "HasActiveEntity" || k == "HasDraftEntity"
}

pub(crate) fn inject_draft_admin_data(record: &mut Value, key_field: &str) {
    if let Some(obj) = record.as_object_mut() {
        let is_draft = obj.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false);
        if is_draft {
            obj.insert(
                "DraftAdministrativeData".to_string(),
                json!({
                    "DraftUUID": format!("draft-{}", obj.get(key_field).and_then(|v| v.as_str()).unwrap_or("unknown")),
                    "InProcessByUser": ""
                }),
            );
        } else {
            obj.entry("DraftAdministrativeData".to_string())
                .or_insert(Value::Null);
        }
    }
}

/// Inject SiblingEntity into a record.
/// For a draft with an active sibling → returns the active record.
/// For an active entity with a draft → returns the draft record.
/// Otherwise → null.
pub(crate) fn inject_sibling_entity(record: &mut Value, key_field: &str, records: &[Value]) {
    if let Some(obj) = record.as_object_mut() {
        let is_active = obj
            .get("IsActiveEntity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let has_sibling = if is_active {
            obj.get("HasDraftEntity")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        } else {
            obj.get("HasActiveEntity")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        };
        let sibling = if has_sibling {
            if let Some(key_value) = obj.get(key_field).and_then(|v| v.as_str()) {
                find_record(records, key_field, key_value, !is_active)
                    .cloned()
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        } else {
            Value::Null
        };
        obj.insert("SiblingEntity".to_string(), sibling);
    }
}

/// Resolve value_source text fields on a single record.
/// For each field with value_source + text_path, looks up the Code in
/// FieldValueListItems and sets the _text field to the Description.
fn resolve_value_texts(
    entity: &dyn ODataEntity,
    record: &mut Value,
    store: &HashMap<String, Vec<Value>>,
) {
    let fields = match entity.fields_def() {
        Some(f) => f,
        None => return,
    };
    let vs_fields: Vec<(&str, &str, &str)> = fields
        .iter()
        .filter_map(|f| Some((f.name, f.value_source?, f.text_path?)))
        .collect();
    if vs_fields.is_empty() {
        return;
    }
    let items = match store.get("FieldValueListItems") {
        Some(i) => i,
        None => return,
    };
    if let Some(obj) = record.as_object_mut() {
        for (field_name, list_id, text_field) in vs_fields {
            if let Some(code) = obj.get(field_name).and_then(|v| v.as_str()) {
                let desc = items
                    .iter()
                    .find(|item| {
                        item.get("ListID").and_then(|v| v.as_str()) == Some(list_id)
                            && item.get("Code").and_then(|v| v.as_str()) == Some(code)
                    })
                    .and_then(|item| item.get("Description").and_then(|v| v.as_str()))
                    .unwrap_or(code);
                obj.insert(text_field.to_string(), Value::String(desc.to_string()));
            }
        }
    }
}

/// Resolve the FK field name on the child that points back to the parent.
/// Uses NavigationProperty.foreign_key if declared, otherwise falls back to parent key_field.
pub(crate) fn resolve_child_fk<'a>(
    parent_entity: &'a dyn ODataEntity,
    child_entity: &'a dyn ODataEntity,
) -> &'a str {
    parent_entity
        .navigation_properties()
        .iter()
        .find(|np| np.target_type == child_entity.type_name())
        .and_then(|np| np.foreign_key)
        .unwrap_or(parent_entity.key_field())
}

fn copy_children_as_drafts(
    store: &mut HashMap<String, Vec<Value>>,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        let child_fk = resolve_child_fk(parent_entity, *child);
        let drafts: Vec<Value> = store
            .get(child.set_name())
            .map(|recs| {
                recs.iter()
                    .filter(|r| {
                        r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(true)
                    })
                    .map(|r| {
                        let mut d = r.clone();
                        set_draft_flags(&mut d, false, true, false);
                        d
                    })
                    .collect()
            })
            .unwrap_or_default();
        if let Some(child_recs) = store.get_mut(child.set_name()) {
            child_recs.extend(drafts);
        }
    }
}

fn activate_children(
    store: &mut HashMap<String, Vec<Value>>,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        let child_fk = resolve_child_fk(parent_entity, *child);
        if let Some(child_records) = store.get_mut(child.set_name()) {
            let draft_items: Vec<Value> = child_records
                .iter()
                .filter(|r| {
                    r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                        && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false)
                })
                .cloned()
                .collect();

            // Remove all records for this parent (active + draft)
            child_records
                .retain(|r| r.get(child_fk).and_then(|v| v.as_str()) != Some(parent_key_value));

            for mut item in draft_items {
                set_draft_flags(&mut item, true, false, false);
                child_records.push(item);
            }
        }
    }
}

fn remove_child_drafts(
    store: &mut HashMap<String, Vec<Value>>,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        let child_fk = resolve_child_fk(parent_entity, *child);
        if let Some(child_recs) = store.get_mut(child.set_name()) {
            child_recs.retain(|r| {
                !(r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                    && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false))
            });
        }
    }
}

// ── Data loading ────────────────────────────────────────────────────

fn load_entity_data(set_name: &str, data_dir: &Path, entity: &dyn ODataEntity) -> Vec<Value> {
    let json_path = data_dir.join(format!("{}.json", set_name));
    if json_path.is_file() {
        match std::fs::read_to_string(&json_path) {
            Ok(content) => match serde_json::from_str::<Vec<Value>>(&content) {
                Ok(records) => {
                    info!(
                        "  {} : {} records from {}",
                        set_name,
                        records.len(),
                        json_path.display()
                    );
                    return records;
                }
                Err(e) => {
                    eprintln!(
                        "  WARNING: {} is not a valid JSON array: {} – falling back to mock_data()",
                        json_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "  WARNING: Could not read {}: {} – falling back to mock_data()",
                    json_path.display(),
                    e
                );
            }
        }
    } else {
        info!(
            "  {} : mock_data() (no file {})",
            set_name,
            json_path.display()
        );
    }
    entity.mock_data()
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EntityKey tests ─────────────────────────────────────────────

    #[test]
    fn entity_key_single() {
        let key = EntityKey::single("ProductID", "P001");
        assert_eq!(key.get("ProductID"), Some("P001"));
        assert_eq!(key.resolve_key_value("ProductID"), Some("P001"));
        assert!(key.is_active()); // default true
    }

    #[test]
    fn entity_key_composite() {
        let key = EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "true")]);
        assert_eq!(key.get("OrderID"), Some("O001"));
        assert_eq!(key.get("IsActiveEntity"), Some("true"));
        assert!(key.is_active());
    }

    #[test]
    fn entity_key_composite_inactive() {
        let key = EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "false")]);
        assert!(!key.is_active());
    }

    #[test]
    fn entity_key_parse_simple() {
        let key = EntityKey::parse("'P001'");
        assert_eq!(key.get("_key"), Some("P001"));
        assert_eq!(key.resolve_key_value("ProductID"), Some("P001"));
    }

    #[test]
    fn entity_key_parse_composite() {
        let key = EntityKey::parse("OrderID='O001',IsActiveEntity=true");
        assert_eq!(key.get("OrderID"), Some("O001"));
        assert_eq!(key.get("IsActiveEntity"), Some("true"));
        assert!(key.is_active());
    }

    #[test]
    fn entity_key_parse_composite_with_quotes() {
        let key = EntityKey::parse("ProductID='P001',IsActiveEntity=false");
        assert_eq!(key.get("ProductID"), Some("P001"));
        assert!(!key.is_active());
        assert_eq!(key.resolve_key_value("ProductID"), Some("P001"));
    }

    #[test]
    fn entity_key_missing_field_returns_none() {
        let key = EntityKey::single("ProductID", "P001");
        assert_eq!(key.get("OrderID"), None);
    }

    // ── ParentKey tests ─────────────────────────────────────────────

    #[test]
    fn parent_key_construction() {
        let parent = ParentKey::new("Orders", EntityKey::single("OrderID", "O001"));
        assert_eq!(parent.set_name, "Orders");
        assert_eq!(parent.key.get("OrderID"), Some("O001"));
    }

    // ── ODataQuery tests ────────────────────────────────────────────

    #[test]
    fn odata_query_empty() {
        let q = ODataQuery::empty();
        assert!(q.filter.is_none());
        assert!(q.select.is_empty());
        assert!(q.expand.is_empty());
        assert!(q.orderby.is_none());
        assert!(q.top.is_none());
        assert!(q.skip.is_none());
        assert!(!q.count);
    }

    #[test]
    fn odata_query_parse_filter() {
        let q = ODataQuery::parse("$filter=Status%20eq%20'A'");
        assert_eq!(q.filter, Some("Status eq 'A'".to_string()));
    }

    #[test]
    fn odata_query_parse_select() {
        let q = ODataQuery::parse("$select=ProductID,ProductName,Price");
        assert_eq!(q.select, vec!["ProductID", "ProductName", "Price"]);
    }

    #[test]
    fn odata_query_parse_expand_simple() {
        let q = ODataQuery::parse("$expand=Items");
        assert_eq!(q.expand.len(), 1);
        assert_eq!(q.expand[0].nav_property, "Items");
        assert!(q.expand[0].select.is_empty());
    }

    #[test]
    fn odata_query_parse_expand_with_select() {
        let q =
            ODataQuery::parse("$expand=DraftAdministrativeData($select=DraftUUID,InProcessByUser)");
        assert_eq!(q.expand.len(), 1);
        assert_eq!(q.expand[0].nav_property, "DraftAdministrativeData");
        assert_eq!(q.expand[0].select, vec!["DraftUUID", "InProcessByUser"]);
    }

    #[test]
    fn odata_query_parse_expand_multiple() {
        let q = ODataQuery::parse("$expand=Items,DraftAdministrativeData($select=DraftUUID)");
        assert_eq!(q.expand.len(), 2);
        assert_eq!(q.expand[0].nav_property, "Items");
        assert_eq!(q.expand[1].nav_property, "DraftAdministrativeData");
    }

    #[test]
    fn odata_query_parse_orderby_asc() {
        let q = ODataQuery::parse("$orderby=Price");
        let ob = q.orderby.unwrap();
        assert_eq!(ob.field, "Price");
        assert!(!ob.descending);
    }

    #[test]
    fn odata_query_parse_orderby_desc() {
        let q = ODataQuery::parse("$orderby=Price%20desc");
        let ob = q.orderby.unwrap();
        assert_eq!(ob.field, "Price");
        assert!(ob.descending);
    }

    #[test]
    fn odata_query_parse_top_skip() {
        let q = ODataQuery::parse("$top=10&$skip=20");
        assert_eq!(q.top, Some(10));
        assert_eq!(q.skip, Some(20));
    }

    #[test]
    fn odata_query_parse_count() {
        let q = ODataQuery::parse("$count=true");
        assert!(q.count);
    }

    #[test]
    fn odata_query_parse_combined() {
        let q = ODataQuery::parse(
            "$filter=Status%20eq%20'A'&$orderby=Price%20desc&$top=5&$skip=0&$count=true&$select=ProductID,Price",
        );
        assert_eq!(q.filter, Some("Status eq 'A'".to_string()));
        assert_eq!(q.select, vec!["ProductID", "Price"]);
        let ob = q.orderby.unwrap();
        assert_eq!(ob.field, "Price");
        assert!(ob.descending);
        assert_eq!(q.top, Some(5));
        assert_eq!(q.skip, Some(0));
        assert!(q.count);
    }

    #[test]
    fn odata_query_to_query_map_roundtrip() {
        let q = ODataQuery {
            filter: Some("Status eq 'A'".to_string()),
            select: vec!["ProductID".to_string(), "Price".to_string()],
            expand: vec![ExpandClause {
                nav_property: "Items".to_string(),
                select: vec![],
            }],
            orderby: Some(OrderByClause {
                field: "Price".to_string(),
                descending: true,
            }),
            top: Some(10),
            skip: Some(5),
            count: true,
        };
        let map = q.to_query_map();
        assert_eq!(map.get("$filter").unwrap(), "Status eq 'A'");
        assert_eq!(map.get("$select").unwrap(), "ProductID,Price");
        assert_eq!(map.get("$expand").unwrap(), "Items");
        assert_eq!(map.get("$orderby").unwrap(), "Price desc");
        assert_eq!(map.get("$top").unwrap(), "10");
        assert_eq!(map.get("$skip").unwrap(), "5");
        assert_eq!(map.get("$count").unwrap(), "true");
    }

    // ── StoreError tests ────────────────────────────────────────────

    #[test]
    fn store_error_display() {
        let e = StoreError::NotFound("test".to_string());
        assert_eq!(format!("{}", e), "Not found: test");
        let e = StoreError::BadRequest("bad".to_string());
        assert_eq!(format!("{}", e), "Bad request: bad");
    }

    // ── InMemoryDataStore tests ─────────────────────────────────────

    use crate::annotations::*;

    /// Minimal test entity for unit tests.
    #[derive(Debug)]
    struct TestProductEntity;

    impl ODataEntity for TestProductEntity {
        fn set_name(&self) -> &'static str {
            "Products"
        }
        fn key_field(&self) -> &'static str {
            "ProductID"
        }
        fn type_name(&self) -> &'static str {
            "Product"
        }
        fn mock_data(&self) -> Vec<Value> {
            vec![
                json!({"ProductID": "P001", "ProductName": "Laptop", "Price": "1299.99", "Status": "A"}),
                json!({"ProductID": "P002", "ProductName": "Mouse", "Price": "29.99", "Status": "A"}),
                json!({"ProductID": "P003", "ProductName": "Monitor", "Price": "499.99", "Status": "D"}),
            ]
        }
        fn entity_set(&self) -> String {
            String::new()
        }
        fn fields_def(&self) -> Option<&'static [FieldDef]> {
            static FIELDS: &[FieldDef] = &[
                FieldDef {
                    name: "ProductID",
                    label: "Product ID",
                    edm_type: "Edm.String",
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "ProductName",
                    label: "Name",
                    edm_type: "Edm.String",
                    max_length: Some(80),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "Price",
                    label: "Price",
                    edm_type: "Edm.Decimal",
                    max_length: None,
                    precision: Some(10),
                    scale: Some(2),
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "Status",
                    label: "Status",
                    edm_type: "Edm.String",
                    max_length: Some(1),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
            ];
            Some(FIELDS)
        }
    }

    #[derive(Debug)]
    struct TestOrderEntity;

    impl ODataEntity for TestOrderEntity {
        fn set_name(&self) -> &'static str {
            "Orders"
        }
        fn key_field(&self) -> &'static str {
            "OrderID"
        }
        fn type_name(&self) -> &'static str {
            "Order"
        }
        fn mock_data(&self) -> Vec<Value> {
            vec![
                json!({"OrderID": "O001", "CustomerName": "Alice", "TotalAmount": "100.00"}),
                json!({"OrderID": "O002", "CustomerName": "Bob", "TotalAmount": "200.00"}),
            ]
        }
        fn entity_set(&self) -> String {
            String::new()
        }
    }

    #[derive(Debug)]
    struct TestOrderItemEntity;

    impl ODataEntity for TestOrderItemEntity {
        fn set_name(&self) -> &'static str {
            "OrderItems"
        }
        fn key_field(&self) -> &'static str {
            "ItemID"
        }
        fn type_name(&self) -> &'static str {
            "OrderItem"
        }
        fn mock_data(&self) -> Vec<Value> {
            vec![
                json!({"ItemID": "I001", "OrderID": "O001", "ProductID": "P001", "Quantity": 2}),
                json!({"ItemID": "I002", "OrderID": "O001", "ProductID": "P002", "Quantity": 5}),
                json!({"ItemID": "I003", "OrderID": "O002", "ProductID": "P001", "Quantity": 1}),
            ]
        }
        fn entity_set(&self) -> String {
            String::new()
        }
        fn parent_set_name(&self) -> Option<&'static str> {
            Some("Orders")
        }
    }

    fn create_test_store() -> InMemoryDataStore {
        // Use a temp dir that doesn't exist so it falls back to mock_data
        let data_dir = std::env::temp_dir().join("fiori-test-nonexistent");
        let entities: Vec<&'static dyn ODataEntity> =
            vec![&TestProductEntity, &TestOrderEntity, &TestOrderItemEntity];
        InMemoryDataStore::new(data_dir, entities)
    }

    #[test]
    fn store_get_collection_returns_all() {
        let store = create_test_store();
        let q = ODataQuery::empty();
        let result = store.get_collection("Products", &q, None).unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn store_get_collection_with_filter() {
        let store = create_test_store();
        let q = ODataQuery::parse("$filter=Status eq 'A'");
        let result = store.get_collection("Products", &q, None).unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn store_get_collection_with_top_skip() {
        let store = create_test_store();
        let q = ODataQuery::parse("$top=1&$skip=1");
        let result = store.get_collection("Products", &q, None).unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn store_get_collection_with_orderby() {
        let store = create_test_store();
        let q = ODataQuery::parse("$orderby=Price desc");
        let result = store.get_collection("Products", &q, None).unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        // Laptop (1299.99) should be first
        assert_eq!(
            values[0].get("ProductName").unwrap().as_str().unwrap(),
            "Laptop"
        );
    }

    #[test]
    fn store_get_collection_with_count() {
        let store = create_test_store();
        let q = ODataQuery::parse("$count=true");
        let result = store.get_collection("Products", &q, None).unwrap();
        assert_eq!(result.get("@odata.count").unwrap().as_i64().unwrap(), 3);
    }

    #[test]
    fn store_get_collection_not_found() {
        let store = create_test_store();
        let q = ODataQuery::empty();
        let result = store.get_collection("NonExistent", &q, None);
        assert!(result.is_err());
    }

    #[test]
    fn store_get_collection_sub_collection() {
        let store = create_test_store();
        let parent = ParentKey::new("Orders", EntityKey::single("OrderID", "O001"));
        let q = ODataQuery::empty();
        let result = store
            .get_collection("OrderItems", &q, Some(&parent))
            .unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2); // I001 and I002 belong to O001
    }

    #[test]
    fn store_count_all() {
        let store = create_test_store();
        let q = ODataQuery::empty();
        assert_eq!(store.count("Products", &q, None), 3);
    }

    #[test]
    fn store_count_sub_collection() {
        let store = create_test_store();
        let parent = ParentKey::new("Orders", EntityKey::single("OrderID", "O001"));
        let q = ODataQuery::empty();
        assert_eq!(store.count("OrderItems", &q, Some(&parent)), 2);
    }

    #[test]
    fn store_read_entity() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        let q = ODataQuery::empty();
        let result = store.read_entity("Products", &key, &q).unwrap();
        assert_eq!(
            result.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop"
        );
        assert!(result.get("@odata.context").is_some());
    }

    #[test]
    fn store_read_entity_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P999");
        let q = ODataQuery::empty();
        let result = store.read_entity("Products", &key, &q);
        assert!(result.is_err());
    }

    #[test]
    fn store_read_entity_parsed_key() {
        let store = create_test_store();
        let key = EntityKey::parse("'P002'");
        let q = ODataQuery::empty();
        let result = store.read_entity("Products", &key, &q).unwrap();
        assert_eq!(
            result.get("ProductName").unwrap().as_str().unwrap(),
            "Mouse"
        );
    }

    #[test]
    fn store_create_entity() {
        let store = create_test_store();
        let data = json!({"ProductName": "Keyboard", "Price": "79.99", "Status": "A"});
        let result = store.create_entity("Products", &data, None).unwrap();
        assert!(result.get("ProductID").is_some()); // auto-generated
        assert_eq!(
            result.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            false
        ); // draft
        assert!(result.get("@odata.context").is_some());

        // Verify it's in the store
        let q = ODataQuery::empty();
        assert_eq!(store.count("Products", &q, None), 4);
    }

    #[test]
    fn store_create_sub_item() {
        let store = create_test_store();
        let parent = ParentKey::new("Orders", EntityKey::single("OrderID", "O002"));
        let data = json!({"ProductID": "P003", "Quantity": 3});
        let result = store
            .create_entity("OrderItems", &data, Some(&parent))
            .unwrap();
        assert_eq!(result.get("OrderID").unwrap().as_str().unwrap(), "O002");
        assert!(result.get("ItemID").is_some());
    }

    #[test]
    fn store_patch_entity() {
        let store = create_test_store();
        // First create a draft to patch (drafts are editable)
        let key = EntityKey::single("ProductID", "P001");
        let edit_result = store.draft_edit("Products", &key).unwrap();
        assert_eq!(
            edit_result
                .get("IsActiveEntity")
                .unwrap()
                .as_bool()
                .unwrap(),
            false
        );

        // Patch the draft
        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        let patch = json!({"ProductName": "Laptop Pro Max"});
        let result = store.patch_entity("Products", &draft_key, &patch).unwrap();
        assert_eq!(
            result.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop Pro Max"
        );
    }

    #[test]
    fn store_patch_entity_immutable_field_ignored() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        store.draft_edit("Products", &key).unwrap();

        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        let patch = json!({"ProductID": "HACKED", "ProductName": "Changed"});
        let result = store.patch_entity("Products", &draft_key, &patch).unwrap();
        // ProductID is immutable, should not change
        assert_eq!(result.get("ProductID").unwrap().as_str().unwrap(), "P001");
        assert_eq!(
            result.get("ProductName").unwrap().as_str().unwrap(),
            "Changed"
        );
    }

    #[test]
    fn store_patch_entity_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P999");
        let patch = json!({"ProductName": "X"});
        let result = store.patch_entity("Products", &key, &patch);
        assert!(result.is_err());
    }

    #[test]
    fn store_delete_entity() {
        let store = create_test_store();
        // Create a draft then delete it
        let key = EntityKey::single("ProductID", "P001");
        store.draft_edit("Products", &key).unwrap();

        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        store.delete_entity("Products", &draft_key).unwrap();

        // Draft should be gone, active should have HasDraftEntity=false
        let active_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "true")]);
        let q = ODataQuery::empty();
        let active = store.read_entity("Products", &active_key, &q).unwrap();
        assert_eq!(
            active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
            false
        );

        // Draft should not exist
        let draft_read = store.read_entity("Products", &draft_key, &q);
        assert!(draft_read.is_err());
    }

    #[test]
    fn store_delete_entity_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P999");
        let result = store.delete_entity("Products", &key);
        assert!(result.is_err());
    }

    // ── Draft lifecycle tests ───────────────────────────────────────

    #[test]
    fn store_draft_edit_creates_draft() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        let draft = store.draft_edit("Products", &key).unwrap();

        assert_eq!(
            draft.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            false
        );
        assert_eq!(
            draft.get("HasActiveEntity").unwrap().as_bool().unwrap(),
            true
        );
        assert_eq!(
            draft.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop"
        );

        // Active entity should now have HasDraftEntity=true
        let active_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "true")]);
        let q = ODataQuery::empty();
        let active = store.read_entity("Products", &active_key, &q).unwrap();
        assert_eq!(
            active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
            true
        );
    }

    #[test]
    fn store_draft_edit_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P999");
        let result = store.draft_edit("Products", &key);
        assert!(result.is_err());
    }

    #[test]
    fn store_draft_activate_updates_active() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");

        let n = store.record_count("Products");
        assert_eq!(n, 3); // ensure initial count

        // Edit → creates draft
        store.draft_edit("Products", &key).unwrap();

        assert_eq!(n + 1, store.record_count("Products")); // draft added

        // Patch draft
        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        let patch = json!({"ProductName": "Laptop Pro 16"});
        store.patch_entity("Products", &draft_key, &patch).unwrap();

        // Activate
        let activated = store.draft_activate("Products", &key).unwrap();
        assert_eq!(
            activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            true
        );
        assert_eq!(
            activated.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop Pro 16"
        );
        assert_eq!(
            activated.get("HasDraftEntity").unwrap().as_bool().unwrap(),
            false
        );

        assert_eq!(n, store.record_count("Products"));

        // Draft should be gone
        let q = ODataQuery::empty();
        let draft_read = store.read_entity("Products", &draft_key, &q);
        assert!(draft_read.is_err());
    }

    #[test]
    fn store_draft_activate_new_entity() {
        let store = create_test_store();
        // Create a brand new entity (no active counterpart)
        let data = json!({"ProductName": "New Product", "Price": "9.99"});
        let created = store.create_entity("Products", &data, None).unwrap();
        let new_key_value = created.get("ProductID").unwrap().as_str().unwrap();

        let new_key = EntityKey::single("ProductID", new_key_value);
        let activated = store.draft_activate("Products", &new_key).unwrap();
        assert_eq!(
            activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            true
        );
        assert_eq!(
            activated.get("ProductName").unwrap().as_str().unwrap(),
            "New Product"
        );
    }

    #[test]
    fn store_draft_prepare() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        store.draft_edit("Products", &key).unwrap();

        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        let result = store.draft_prepare("Products", &draft_key).unwrap();
        assert_eq!(
            result.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop"
        );
        assert!(result.get("@odata.context").is_some());
    }

    #[test]
    fn store_draft_prepare_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P999");
        let result = store.draft_prepare("Products", &key);
        assert!(result.is_err());
    }

    // ── Draft with children ─────────────────────────────────────────

    #[test]
    fn store_draft_edit_copies_children() {
        let store = create_test_store();
        let key = EntityKey::single("OrderID", "O001");
        store.draft_edit("Orders", &key).unwrap();

        // Check that child drafts were created
        let parent = ParentKey::new(
            "Orders",
            EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "false")]),
        );
        let q = ODataQuery::empty();
        let children = store
            .get_collection("OrderItems", &q, Some(&parent))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2); // I001 and I002 as drafts
        for v in values {
            assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), false);
        }
    }

    #[test]
    fn store_draft_activate_activates_children() {
        let store = create_test_store();
        let key = EntityKey::single("OrderID", "O001");
        store.draft_edit("Orders", &key).unwrap();

        // Activate parent
        store.draft_activate("Orders", &key).unwrap();

        // Children should be active again
        let parent = ParentKey::new(
            "Orders",
            EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "true")]),
        );
        let q = ODataQuery::empty();
        let children = store
            .get_collection("OrderItems", &q, Some(&parent))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2);
        for v in values {
            assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), true);
        }
    }

    #[test]
    fn store_delete_draft_removes_children() {
        let store = create_test_store();
        let key = EntityKey::single("OrderID", "O001");
        store.draft_edit("Orders", &key).unwrap();

        // Delete draft (discard)
        let draft_key = EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "false")]);
        store.delete_entity("Orders", &draft_key).unwrap();

        // Draft children should be gone
        let parent_draft = ParentKey::new("Orders", draft_key.clone());
        let q = ODataQuery::empty();
        let children = store
            .get_collection("OrderItems", &q, Some(&parent_draft))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 0);

        // Active children should still be there
        let parent_active = ParentKey::new(
            "Orders",
            EntityKey::composite(&[("OrderID", "O001"), ("IsActiveEntity", "true")]),
        );
        let active_children = store
            .get_collection("OrderItems", &q, Some(&parent_active))
            .unwrap();
        let active_values = active_children.get("value").unwrap().as_array().unwrap();
        assert_eq!(active_values.len(), 2);
    }

    // ── Property access ─────────────────────────────────────────────

    #[test]
    fn store_get_property() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        let val = store.get_property("Products", &key, "ProductName").unwrap();
        assert_eq!(val.as_str().unwrap(), "Laptop");
    }

    #[test]
    fn store_get_property_not_found() {
        let store = create_test_store();
        let key = EntityKey::single("ProductID", "P001");
        let result = store.get_property("Products", &key, "NonExistentField");
        assert!(result.is_err());
    }

    // ── get_records ─────────────────────────────────────────────────

    #[test]
    fn store_get_records() {
        let store = create_test_store();
        let records = store.get_records("Products");
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn store_get_records_nonexistent() {
        let store = create_test_store();
        let records = store.get_records("NonExistent");
        assert!(records.is_empty());
    }

    // ── Commit ──────────────────────────────────────────────────────

    #[test]
    fn store_commit_writes_json_files() {
        let tmp_dir = std::env::temp_dir().join(format!("fiori-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let entities: Vec<&'static dyn ODataEntity> = vec![&TestProductEntity];
        let store = InMemoryDataStore::new(tmp_dir.clone(), entities);

        // Patch a product
        let key = EntityKey::single("ProductID", "P001");
        store.draft_edit("Products", &key).unwrap();
        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        store
            .patch_entity("Products", &draft_key, &json!({"ProductName": "Laptop V2"}))
            .unwrap();
        store.draft_activate("Products", &key).unwrap();

        // Commit
        store.commit();

        // Verify JSON file
        let json_path = tmp_dir.join("Products.json");
        let content = std::fs::read_to_string(&json_path).unwrap();
        let records: Vec<Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(records.len(), 3);

        // Verify draft flags are NOT in the persisted data
        for r in &records {
            assert!(r.get("IsActiveEntity").is_none());
            assert!(r.get("HasActiveEntity").is_none());
            assert!(r.get("HasDraftEntity").is_none());
        }

        // Verify update was saved
        let laptop = records
            .iter()
            .find(|r| r.get("ProductID").unwrap().as_str() == Some("P001"))
            .unwrap();
        assert_eq!(
            laptop.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop V2"
        );

        // Cleanup
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    // ── Full lifecycle integration test ─────────────────────────────

    #[test]
    fn store_full_draft_lifecycle() {
        let store = create_test_store();
        let q = ODataQuery::empty();

        // 1. Read active entity
        let key = EntityKey::single("ProductID", "P001");
        let active = store.read_entity("Products", &key, &q).unwrap();
        assert_eq!(
            active.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop"
        );

        // 2. Draft edit
        let draft = store.draft_edit("Products", &key).unwrap();
        assert_eq!(
            draft.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            false
        );

        // 3. Patch draft
        let draft_key = EntityKey::composite(&[("ProductID", "P001"), ("IsActiveEntity", "false")]);
        store
            .patch_entity(
                "Products",
                &draft_key,
                &json!({"ProductName": "Laptop 2026"}),
            )
            .unwrap();

        // 4. Draft prepare
        let prepared = store.draft_prepare("Products", &draft_key).unwrap();
        assert_eq!(
            prepared.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop 2026"
        );

        // 5. Activate
        let activated = store.draft_activate("Products", &key).unwrap();
        assert_eq!(
            activated.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop 2026"
        );
        assert_eq!(
            activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            true
        );

        // 6. Verify active entity is updated
        let final_read = store.read_entity("Products", &key, &q).unwrap();
        assert_eq!(
            final_read.get("ProductName").unwrap().as_str().unwrap(),
            "Laptop 2026"
        );

        // 7. Verify no draft remains
        let draft_read = store.read_entity("Products", &draft_key, &q);
        assert!(draft_read.is_err());
    }

    #[test]
    fn store_draft_discard_lifecycle() {
        let store = create_test_store();
        let q = ODataQuery::empty();

        // 1. Draft edit
        let key = EntityKey::single("ProductID", "P002");
        store.draft_edit("Products", &key).unwrap();

        // 2. Patch draft
        let draft_key = EntityKey::composite(&[("ProductID", "P002"), ("IsActiveEntity", "false")]);
        store
            .patch_entity("Products", &draft_key, &json!({"ProductName": "Changed"}))
            .unwrap();

        // 3. Discard (delete draft)
        store.delete_entity("Products", &draft_key).unwrap();

        // 4. Active should be unchanged
        let active = store.read_entity("Products", &key, &q).unwrap();
        assert_eq!(
            active.get("ProductName").unwrap().as_str().unwrap(),
            "Mouse"
        );
        assert_eq!(
            active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
            false
        );
    }

    // ── FieldValueList draft tests (custom FK: ListID) ──────────────

    #[derive(Debug)]
    struct TestValueListEntity;

    impl ODataEntity for TestValueListEntity {
        fn set_name(&self) -> &'static str {
            "FieldValueLists"
        }
        fn key_field(&self) -> &'static str {
            "ID"
        }
        fn type_name(&self) -> &'static str {
            "FieldValueList"
        }
        fn mock_data(&self) -> Vec<Value> {
            vec![
                json!({"ID": "VL-001", "ListName": "EdmTypes", "Description": "OData EDM Datentypen"}),
                json!({"ID": "VL-002", "ListName": "StatusCodes", "Description": "Status"}),
            ]
        }
        fn entity_set(&self) -> String {
            String::new()
        }
        fn fields_def(&self) -> Option<&'static [FieldDef]> {
            static FIELDS: &[FieldDef] = &[
                FieldDef {
                    name: "ID",
                    label: "ID",
                    edm_type: "Edm.Guid",
                    max_length: None,
                    precision: None,
                    scale: None,
                    immutable: true,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "ListName",
                    label: "Listenname",
                    edm_type: "Edm.String",
                    max_length: Some(40),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "Description",
                    label: "Beschreibung",
                    edm_type: "Edm.String",
                    max_length: Some(120),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
            ];
            Some(FIELDS)
        }
        fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
            static NAV: &[NavigationPropertyDef] = &[NavigationPropertyDef {
                name: "Items",
                target_type: "FieldValueListItem",
                is_collection: true,
                foreign_key: Some("ListID"),
            }];
            NAV
        }
    }

    #[derive(Debug)]
    struct TestValueListItemEntity;

    impl ODataEntity for TestValueListItemEntity {
        fn set_name(&self) -> &'static str {
            "FieldValueListItems"
        }
        fn key_field(&self) -> &'static str {
            "ID"
        }
        fn type_name(&self) -> &'static str {
            "FieldValueListItem"
        }
        fn mock_data(&self) -> Vec<Value> {
            vec![
                json!({"ID": "ITEM-001", "ListID": "VL-001", "Code": "Edm.String",  "Description": "Zeichenkette", "SortOrder": 0}),
                json!({"ID": "ITEM-002", "ListID": "VL-001", "Code": "Edm.Int32",   "Description": "Ganzzahl",     "SortOrder": 1}),
                json!({"ID": "ITEM-003", "ListID": "VL-002", "Code": "Active",      "Description": "Aktiv",        "SortOrder": 0}),
            ]
        }
        fn entity_set(&self) -> String {
            String::new()
        }
        fn parent_set_name(&self) -> Option<&'static str> {
            Some("FieldValueLists")
        }
        fn fields_def(&self) -> Option<&'static [FieldDef]> {
            static FIELDS: &[FieldDef] = &[
                FieldDef {
                    name: "ID",
                    label: "ID",
                    edm_type: "Edm.Guid",
                    max_length: None,
                    precision: None,
                    scale: None,
                    immutable: true,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "ListID",
                    label: "Listen-ID",
                    edm_type: "Edm.Guid",
                    max_length: None,
                    precision: None,
                    scale: None,
                    immutable: true,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "Code",
                    label: "Code",
                    edm_type: "Edm.String",
                    max_length: Some(40),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "Description",
                    label: "Beschreibung",
                    edm_type: "Edm.String",
                    max_length: Some(120),
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
                FieldDef {
                    name: "SortOrder",
                    label: "Reihenfolge",
                    edm_type: "Edm.Int32",
                    max_length: None,
                    precision: None,
                    scale: None,
                    immutable: false,
                    computed: false,
                    semantic_object: None,
                    value_source: None,
                    value_list: None,
                    text_path: None,
                },
            ];
            Some(FIELDS)
        }
    }

    fn create_vl_store() -> InMemoryDataStore {
        let data_dir = std::env::temp_dir().join("fiori-test-vl-nonexistent");
        let entities: Vec<&'static dyn ODataEntity> =
            vec![&TestValueListEntity, &TestValueListItemEntity];
        InMemoryDataStore::new(data_dir, entities)
    }

    #[test]
    fn vl_read_items_via_parent() {
        let store = create_vl_store();
        let parent = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
        );
        let q = ODataQuery::empty();
        let result = store
            .get_collection("FieldValueListItems", &q, Some(&parent))
            .unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2); // ITEM-001 and ITEM-002 belong to VL-001
    }

    #[test]
    fn vl_read_items_other_parent() {
        let store = create_vl_store();
        let parent = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-002"), ("IsActiveEntity", "true")]),
        );
        let q = ODataQuery::empty();
        let result = store
            .get_collection("FieldValueListItems", &q, Some(&parent))
            .unwrap();
        let values = result.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 1); // ITEM-003 belongs to VL-002
    }

    #[test]
    fn vl_draft_edit_copies_children_with_custom_fk() {
        let store = create_vl_store();
        let key = EntityKey::single("ID", "VL-001");
        store.draft_edit("FieldValueLists", &key).unwrap();

        // Draft children should exist, filtered by ListID (not ID)
        let parent_draft = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
        );
        let q = ODataQuery::empty();
        let children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_draft))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 2);
        for v in values {
            assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), false);
            assert_eq!(v.get("ListID").unwrap().as_str().unwrap(), "VL-001");
        }
    }

    #[test]
    fn vl_create_item_sets_list_id_not_id() {
        let store = create_vl_store();
        let key = EntityKey::single("ID", "VL-001");
        store.draft_edit("FieldValueLists", &key).unwrap();

        // Create a new child item via sub-collection POST
        let parent = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
        );
        let data = json!({"Code": "Edm.Boolean", "Description": "Wahrheitswert", "SortOrder": 2});
        let created = store
            .create_entity("FieldValueListItems", &data, Some(&parent))
            .unwrap();

        // ListID should be set to the parent's key (VL-001)
        assert_eq!(created.get("ListID").unwrap().as_str().unwrap(), "VL-001");
        // ID should be auto-generated and NOT be the parent's key
        let item_id = created.get("ID").unwrap().as_str().unwrap();
        assert_ne!(
            item_id, "VL-001",
            "Child ID must not be overwritten with parent key"
        );
        assert!(
            uuid::Uuid::parse_str(item_id).is_ok(),
            "Edm.Guid key should be a valid UUID, got: {}",
            item_id
        );
    }

    #[test]
    fn vl_create_item_visible_in_subcollection() {
        let store = create_vl_store();
        let key = EntityKey::single("ID", "VL-001");
        store.draft_edit("FieldValueLists", &key).unwrap();

        let parent_draft = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
        );
        let data = json!({"Code": "Edm.Date", "Description": "Datum", "SortOrder": 3});
        store
            .create_entity("FieldValueListItems", &data, Some(&parent_draft))
            .unwrap();

        // Should now have 3 draft items (2 copied + 1 new)
        let q = ODataQuery::empty();
        let children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_draft))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn vl_patch_child_item() {
        let store = create_vl_store();
        let key = EntityKey::single("ID", "VL-001");
        store.draft_edit("FieldValueLists", &key).unwrap();

        // Patch the first draft child
        let draft_item_key =
            EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "false")]);
        let patch = json!({"Description": "Zeichenkette (aktualisiert)"});
        let result = store
            .patch_entity("FieldValueListItems", &draft_item_key, &patch)
            .unwrap();
        assert_eq!(
            result.get("Description").unwrap().as_str().unwrap(),
            "Zeichenkette (aktualisiert)"
        );
        // ListID should remain unchanged
        assert_eq!(result.get("ListID").unwrap().as_str().unwrap(), "VL-001");
    }

    #[test]
    fn vl_activate_with_new_child() {
        let store = create_vl_store();
        let q = ODataQuery::empty();
        let key = EntityKey::single("ID", "VL-001");

        // 1. Edit → draft
        store.draft_edit("FieldValueLists", &key).unwrap();

        // 2. Create new child
        let parent_draft = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
        );
        let data = json!({"Code": "Edm.Guid", "Description": "GUID", "SortOrder": 9});
        store
            .create_entity("FieldValueListItems", &data, Some(&parent_draft))
            .unwrap();

        // 3. Activate parent
        let activated = store.draft_activate("FieldValueLists", &key).unwrap();
        assert_eq!(
            activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            true
        );

        // 4. Active children should include the new item
        let parent_active = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
        );
        let children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_active))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 3); // 2 original + 1 new
        for v in values {
            assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), true);
            assert_eq!(v.get("ListID").unwrap().as_str().unwrap(), "VL-001");
        }

        // 5. No draft children remain
        let children_draft = store
            .get_collection("FieldValueListItems", &q, Some(&parent_draft))
            .unwrap();
        let draft_values = children_draft.get("value").unwrap().as_array().unwrap();
        assert_eq!(draft_values.len(), 0);
    }

    #[test]
    fn vl_activate_with_patched_child() {
        let store = create_vl_store();
        let q = ODataQuery::empty();
        let key = EntityKey::single("ID", "VL-001");

        // Edit, patch child, activate
        store.draft_edit("FieldValueLists", &key).unwrap();
        let draft_item_key =
            EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "false")]);
        store
            .patch_entity(
                "FieldValueListItems",
                &draft_item_key,
                &json!({"Description": "String (updated)"}),
            )
            .unwrap();
        store.draft_activate("FieldValueLists", &key).unwrap();

        // Read active child
        let active_item_key =
            EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "true")]);
        let item = store
            .read_entity("FieldValueListItems", &active_item_key, &q)
            .unwrap();
        assert_eq!(
            item.get("Description").unwrap().as_str().unwrap(),
            "String (updated)"
        );
    }

    #[test]
    fn vl_discard_draft_removes_children() {
        let store = create_vl_store();
        let q = ODataQuery::empty();
        let key = EntityKey::single("ID", "VL-001");

        // Edit → add new item → discard
        store.draft_edit("FieldValueLists", &key).unwrap();
        let parent_draft = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
        );
        store
            .create_entity(
                "FieldValueListItems",
                &json!({"Code": "Edm.Byte", "Description": "Byte", "SortOrder": 10}),
                Some(&parent_draft),
            )
            .unwrap();

        // Discard draft
        let draft_key = EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]);
        store.delete_entity("FieldValueLists", &draft_key).unwrap();

        // Draft children gone
        let children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_draft))
            .unwrap();
        assert_eq!(children.get("value").unwrap().as_array().unwrap().len(), 0);

        // Active children unchanged
        let parent_active = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
        );
        let active = store
            .get_collection("FieldValueListItems", &q, Some(&parent_active))
            .unwrap();
        assert_eq!(active.get("value").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn vl_other_list_unaffected_by_draft() {
        let store = create_vl_store();
        let q = ODataQuery::empty();
        let key = EntityKey::single("ID", "VL-001");

        // Edit VL-001 (EdmTypes) → should NOT create drafts for VL-002's children
        store.draft_edit("FieldValueLists", &key).unwrap();

        // VL-002's active items remain unchanged
        let parent_vl2 = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", "VL-002"), ("IsActiveEntity", "true")]),
        );
        let children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_vl2))
            .unwrap();
        let values = children.get("value").unwrap().as_array().unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].get("Code").unwrap().as_str().unwrap(), "Active");
    }

    #[test]
    fn vl_full_lifecycle_create_list_add_items_activate() {
        let store = create_vl_store();
        let q = ODataQuery::empty();

        // 1. Create a brand new FieldValueList
        let list_data = json!({"ListName": "Priorities", "Description": "Prioritaeten"});
        let created = store
            .create_entity("FieldValueLists", &list_data, None)
            .unwrap();
        let new_list_id = created.get("ID").unwrap().as_str().unwrap().to_string();
        assert_eq!(
            created.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            false
        );

        // 2. Add items to it
        let parent = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", &new_list_id), ("IsActiveEntity", "false")]),
        );
        store
            .create_entity(
                "FieldValueListItems",
                &json!({"Code": "HIGH", "Description": "Hoch", "SortOrder": 0}),
                Some(&parent),
            )
            .unwrap();
        store
            .create_entity(
                "FieldValueListItems",
                &json!({"Code": "MED",  "Description": "Mittel", "SortOrder": 1}),
                Some(&parent),
            )
            .unwrap();
        store
            .create_entity(
                "FieldValueListItems",
                &json!({"Code": "LOW",  "Description": "Niedrig", "SortOrder": 2}),
                Some(&parent),
            )
            .unwrap();

        // 3. Verify draft items
        let draft_children = store
            .get_collection("FieldValueListItems", &q, Some(&parent))
            .unwrap();
        assert_eq!(
            draft_children
                .get("value")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            3
        );

        // 4. Activate the list
        let new_key = EntityKey::single("ID", &new_list_id);
        let activated = store.draft_activate("FieldValueLists", &new_key).unwrap();
        assert_eq!(
            activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
            true
        );
        assert_eq!(
            activated.get("ListName").unwrap().as_str().unwrap(),
            "Priorities"
        );

        // 5. Active children should all be there
        let parent_active = ParentKey::new(
            "FieldValueLists",
            EntityKey::composite(&[("ID", &new_list_id), ("IsActiveEntity", "true")]),
        );
        let active_children = store
            .get_collection("FieldValueListItems", &q, Some(&parent_active))
            .unwrap();
        let items = active_children.get("value").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 3);
        let codes: Vec<&str> = items
            .iter()
            .map(|v| v.get("Code").unwrap().as_str().unwrap())
            .collect();
        assert!(codes.contains(&"HIGH"));
        assert!(codes.contains(&"MED"));
        assert!(codes.contains(&"LOW"));
    }
}
