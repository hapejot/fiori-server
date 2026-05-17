use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use odata_params::filters::{Expr, Value as FilterValue};
use serde_json::{json, Map, Value};
use tracing::{error, info};
use uuid::Uuid;

use crate::entity::ODataEntity;
use crate::model::ResolvedEntity;
use crate::BASE_PATH;

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

/// Find a record by key field value.
fn find_record<'a>(records: &'a [Value], key_field: &str, key_value: &str) -> Option<&'a Value> {
    records
        .iter()
        .find(|r| r.get(key_field).and_then(|v| v.as_str()) == Some(key_value))
}

/// Find a list of records by field value.
fn find_all_record<'a>(records: &'a [Value], field: &str, value: &str) -> Vec<Value> {
    records
        .iter()
        .filter(|r| r.get(field).and_then(|v| v.as_str()) == Some(value))
        .cloned()
        .collect()
}

/// Find a record mutably by key field value.
fn find_record_mut<'a>(
    records: &'a mut [Value],
    key_field: &str,
    key_value: &str,
) -> Option<&'a mut Value> {
    records
        .iter_mut()
        .find(|r| r.get(key_field).and_then(|v| v.as_str()) == Some(key_value))
}

/// Remove all records matching key field value.
fn remove_records(records: &mut Vec<Value>, key_field: &str, key_value: &str) {
    records.retain(|r| r.get(key_field).and_then(|v| v.as_str()) != Some(key_value));
}

// ── ChangeSet ───────────────────────────────────────────────────────

/// Git-like changeset overlay. Holds full record copies (copy-on-write)
/// across all entity sets. One active changeset at a time.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// Full record copies (modified + created), keyed by entity set name.
    pub records: HashMap<String, Vec<Value>>,
}

impl ChangeSet {
    fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Check whether a record with the given key exists in the changeset.
    fn contains(&self, set_name: &str, key_field: &str, key_value: &str) -> bool {
        self.records
            .get(set_name)
            .map(|recs| find_record(recs, key_field, key_value).is_some())
            .unwrap_or(false)
    }
}

/// Inject draft flags into a record based on its location.
/// Baseline records get IsActiveEntity=true, changeset records get IsActiveEntity=false.
pub(crate) fn inject_draft_flags(
    record: &mut Value,
    is_active: bool,
    has_active: bool,
    has_draft: bool,
) {
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
    /// Parse from OData URL key segment: "OrderID='O001',IsActiveEntity=true"
    /// Also handles simple keys: "'P001'"
    pub fn parse(segment: &str) -> Self {
        let segment = segment.trim();
        if segment.starts_with('\'') && segment.ends_with('\'') {
            let value = segment[1..segment.len() - 1].to_string();
            return Self {
                pairs: vec![("_key".to_string(), value)],
            };
        }
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

// ── ODataQuery ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ODataFilterExpression {
    raw: String,
    ast: Arc<RwLock<Option<Expr>>>,
}

impl ODataFilterExpression {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.trim().to_string(),
            ast: Arc::new(RwLock::new(None)),
        }
    }

    pub fn empty() -> Self {
        Self {
            raw: String::new(),
            ast: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn try_parse(&self) -> bool {
        if self.is_empty() {
            return true;
        }
        match odata_params::filters::parse_str(self.raw.as_str()) {
            Ok(ast) => {
                let mut lock = self.ast.write().unwrap();
                *lock = Some(ast);
                true
            }
            Err(_) => false,
        }
    }

    pub fn ast(&self) -> Option<Expr> {
        self.ast.read().unwrap().clone()
    }

    pub fn eval(&self, record: &Value) -> bool {
        if self.is_empty() {
            return true;
        }
        self.try_parse();
        let r: bool = if let Ok(ast) = self.ast.write() {
            if let Some(ast) = ast.as_ref() {
                if let Some(r) = record.as_object() {
                    evaluate_cond(&ast, r)
                } else {
                    todo!();
                }
            } else {
                todo!();
            }
        } else {
            todo!("handle Lock error")
        };

        r
    }
}

fn evaluate_cond(expr: &Expr, record: &Map<String, Value>) -> bool {
    match expr {
        Expr::Or(expr, expr1) => evaluate_cond(expr, record) || evaluate_cond(expr1, record),
        Expr::And(expr, expr1) => evaluate_cond(expr, record) && evaluate_cond(expr1, record),
        Expr::Not(expr) => !evaluate_cond(expr, record),
        Expr::Compare(expr, compare_operator, expr1) => {
            let l = evaluate_value(expr, record);
            let r = evaluate_value(expr1, record);
            match compare_operator {
                odata_params::filters::CompareOperator::Equal => l == r,
                odata_params::filters::CompareOperator::NotEqual => l != r,
                odata_params::filters::CompareOperator::GreaterThan => l > r,
                odata_params::filters::CompareOperator::GreaterOrEqual => l >= r,
                odata_params::filters::CompareOperator::LessThan => l < r,
                odata_params::filters::CompareOperator::LessOrEqual => l <= r,
            }
        }
        Expr::In(expr, exprs) => todo!(),
        Expr::Function(_, exprs) => todo!(),
        Expr::Identifier(_) => todo!(),
        Expr::Value(value) => todo!(),
    }
}

fn evaluate_value(expr: &Expr, record: &Map<String, Value>) -> FilterValue {
    match expr {
        Expr::Identifier(name) => {
            let v = match record.get(name) {
                Some(value) => value,
                None => {
                    error!("Field '{}' not found in record: {:?}", name, record);
                    return FilterValue::Null;
                }
            };
            match v {
                Value::Null => FilterValue::Null,
                Value::Bool(b) => FilterValue::Bool(*b),
                Value::Number(number) => FilterValue::Number(number.as_i64().unwrap().into()),
                Value::String(s) => FilterValue::String(s.clone()),
                Value::Array(values) => todo!(),
                Value::Object(map) => todo!(),
            }
        }
        Expr::Value(value) => value.clone(),
        _ => todo!(),
    }
}

/// Structured OData query parameters.
#[derive(Debug, Clone, Default)]
pub struct ODataQuery {
    pub filter: ODataFilterExpression,
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
                    "$filter" => q.filter = ODataFilterExpression::new(&v),
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
        if !self.filter.is_empty() {
            map.insert("$filter".to_string(), self.filter.raw.clone());
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
        info!("Parsing $expand: {}", expand);
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

/// Trait for data storage backends.
/// All locking/transaction management is internal to the implementation.
/// Callers pass only string identifiers, structured query types, and JSON values.
pub trait DataStore: Send + Sync {
    /// Query a set of entity records with optional querying.
    fn get_collection(
        &self,
        set_name: &str,
        query: &ODataQuery,
        parent: Option<&ParentKey>,
        default: Option<&Value>,
    ) -> Result<Vec<Value>, StoreError>;

    /// Count the number of results that would be returned by an equivalent `get_collection` call (ignoring $top/$skip)
    fn count(&self, set_name: &str, query: &ODataQuery, parent: Option<&ParentKey>) -> usize;

    /// Retrieve a single entity record by key, with optional querying for draft read.
    /// returns a single row only
    fn read_record(
        &self,
        set_name: &str,
        key: &EntityKey,
        query: &ODataQuery,
    ) -> Result<Value, StoreError>;

    /// Create a new entity record in the specified set, returning the created record with key and default values populated.
    fn create_entity(
        &self,
        set_name: &str,
        data: &Value,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError>;

    /// Update an existing entity record by key with the provided data, returning the updated record.
    fn patch_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        patch: &Value,
    ) -> Result<Value, StoreError>;

    /// Delete an entity record by key.
    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError>;

    /// Draft-specific operations.
    /// `draft_edit` changes the draft entity, copying it to the changeset and returning the draft version.
    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    /// `draft_activate` activates a draft entity, making it the active version.
    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    /// `draft_prepare` prepares a draft entity for editing, without activating it.
    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    /// Read the sibling entity (draft ↔ active) for a given key.
    ///
    /// In the context of draft-enabled entities, a sibling entity refers to the counterpart of a
    /// given entity in its alternate state. For example, if you have an active entity (IsActiveEntity=true),
    /// its sibling would be the corresponding draft entity (IsActiveEntity=false) that holds the
    /// in-progress changes. Conversely, if you start with a draft entity, its sibling would be the
    /// active version that represents the last committed state. This allows you to easily navigate between
    /// the two versions of an entity during the editing and activation process.
    fn read_sibling_entity(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError>;

    /// Expand navigation properties for a record, used as a fallback if the query engine doesn't handle it.
    fn get_property(
        &self,
        set_name: &str,
        key: &EntityKey,
        property: &str,
    ) -> Result<Value, StoreError>;

    /// Returns all records of an entity
    /// This should not be used other then for testing/debugging purposes
    fn get_records(&self, set_name: &str) -> Vec<Value>;

    /// Persist any pending changes to the underlying storage medium (e.g. write to disk).
    fn commit(&self);

    // ------------ handle metadata ---------------------------------------

    /// Update the entity definitions in the store (e.g. after resolution or dynamic changes).?
    /// Why do we need this?
    fn update_entities(&self, entities: &[ODataEntity]);

    fn entities(&self) -> Vec<ODataEntity>;

    /// Initialize the store with a set of records for a given entity set, used for seeding synthetic records on startup.
    fn initialize_records(&self, set_name: &str, records: Vec<Value>);

    fn record_count(&self, set_name: &str) -> usize {
        self.count(set_name, &ODataQuery::empty(), None)
    }
}

pub struct DraftDataStore {
    parent: Arc<dyn DataStore>,
    /// Single active changeset overlay (None = no draft session active).
    changeset: RwLock<ChangeSet>,
}

impl DraftDataStore {
    pub fn new(parent: Arc<dyn DataStore>) -> Self {
        let changeset = RwLock::new(ChangeSet::new());
        Self { parent, changeset }
    }
}

impl DataStore for DraftDataStore {
    fn get_collection(
        &self,
        set_name: &str,
        query: &ODataQuery,
        parent: Option<&ParentKey>,
        default: Option<&Value>,
    ) -> Result<Vec<Value>, StoreError> {
        // TODO: introduce default result row into get collection 
        let default = json!({
            "IsActiveEntity": true,
            "HasActiveEntity": true,
            "HasDraftEntity": false,
        });
        let mut r = self.parent.get_collection(set_name, query, parent, None)?;
        Ok(r)
    }

    fn count(&self, set_name: &str, query: &ODataQuery, parent: Option<&ParentKey>) -> usize {
        todo!()
    }

    fn read_record(
        &self,
        set_name: &str,
        key: &EntityKey,
        query: &ODataQuery,
    ) -> Result<Value, StoreError> {
        todo!()
    }

    fn create_entity(
        &self,
        set_name: &str,
        data: &Value,
        parent: Option<&ParentKey>,
    ) -> Result<Value, StoreError> {
        todo!()
    }

    fn patch_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        patch: &Value,
    ) -> Result<Value, StoreError> {
        todo!()
    }

    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError> {
        todo!()
    }

    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn read_sibling_entity(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn get_property(
        &self,
        set_name: &str,
        key: &EntityKey,
        property: &str,
    ) -> Result<Value, StoreError> {
        todo!()
    }

    fn get_records(&self, set_name: &str) -> Vec<Value> {
        todo!()
    }

    fn commit(&self) {
        todo!()
    }

    fn update_entities(&self, entities: &[ODataEntity]) {
        todo!()
    }

    fn initialize_records(&self, set_name: &str, records: Vec<Value>) {
        todo!()
    }

    fn entities(&self) -> Vec<ODataEntity> {
        todo!()
    }
}

// ── InMemoryDataStore ───────────────────────────────────────────────

/// In-memory data store backed by JSON files.
/// Loads all data on initialization, persists on commit().
/// Draft editing uses a ChangeSet overlay (Git-like copy-on-write).
pub struct InMemoryDataStore {
    /// Baseline data: only active/committed records.
    store: RwLock<HashMap<String, Vec<Value>>>,
    entities: RwLock<Vec<ODataEntity>>,
    resolved_entities: RwLock<Vec<ResolvedEntity>>,
    data_dir: PathBuf,
}

impl InMemoryDataStore {
    /// Create a new in-memory store, loading data from JSON files.
    /// Baseline contains clean records without draft flags.
    pub fn new(data_dir: PathBuf, entities: Vec<ODataEntity>) -> Self {
        let mut store = HashMap::new();
        for entity in &entities {
            let set_name = entity.set_name();
            let records = load_entity_data(set_name, &data_dir, entity);
            store.insert(set_name.to_string(), records);
        }

        Self {
            store: RwLock::new(store),
            entities: RwLock::new(entities),
            resolved_entities: RwLock::new(vec![]),
            data_dir,
        }
    }

    fn find_entity(&self, set_name: &str) -> Option<ODataEntity> {
        for x in self.entities.read().unwrap().iter() {
            if x.set_name() == set_name {
                return Some(x.clone());
            }
        }
        None
    }
    fn find_entity_by_type(&self, type_name: &str) -> Option<ODataEntity> {
        for x in self.entities.read().unwrap().iter() {
            if x.type_name() == type_name {
                return Some(x.clone());
            }
        }
        None
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
    fn entities_snapshot(&self) -> Vec<ODataEntity> {
        self.entities.read().unwrap().clone()
    }

    #[tracing::instrument(skip(self))]
    fn resolved_entities_snapshot(&self) -> Vec<ResolvedEntity> {
        self.resolved_entities.read().unwrap().clone()
    }

    fn fallback_expand_collection(
        &self,
        set_name: &str,
        result: &mut Value,
        expand_names: &[&str],
        entities: &[ODataEntity],
        store: &HashMap<String, Vec<Value>>,
        resolved_entities: &[ResolvedEntity],
    ) {
        info!("fallback");
        let Some(rows) = result.get_mut("value").and_then(|v| v.as_array_mut()) else {
            return;
        };
        for row in rows {
            self.fallback_expand_record(
                set_name,
                row,
                expand_names,
                entities,
                store,
                resolved_entities,
            );
        }
    }

    fn fallback_expand_record(
        &self,
        set_name: &str,
        record: &mut Value,
        expand_names: &[&str],
        entities: &[ODataEntity],
        store: &HashMap<String, Vec<Value>>,
        resolved_entities: &[ResolvedEntity],
    ) {
        let Some(obj) = record.as_object_mut() else {
            return;
        };
        let Some(entity_resolved) = resolved_entities.iter().find(|e| e.set_name == set_name)
        else {
            return;
        };

        for nav_name in expand_names {
            // Respect explicit entity-specific expansion if it already populated the nav.
            if obj.contains_key(*nav_name) {
                continue;
            }
            let Some(nav) = entity_resolved
                .nav_properties
                .iter()
                .find(|n| n.name == *nav_name)
            else {
                continue;
            };

            let target_entity = entities
                .iter()
                .find(|e| e.set_name() == nav.target_set)
                .cloned()
                .or_else(|| {
                    entities
                        .iter()
                        .find(|e| e.type_name() == nav.target_type)
                        .cloned()
                });
            let Some(target_entity) = target_entity else {
                continue;
            };

            let target_records = store
                .get(target_entity.set_name())
                .cloned()
                .unwrap_or_else(|| target_entity.initial_data());

            if nav.is_collection {
                let parent_key = obj
                    .get("ID")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let Some(parent_key) = parent_key else {
                    continue;
                };
                let child_fk = nav.foreign_key.as_deref().unwrap_or("ID");
                let children: Vec<Value> = target_records
                    .into_iter()
                    .filter(|r| {
                        r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key.as_str())
                    })
                    .collect();
                obj.insert(nav.name.clone(), Value::Array(children));
            } else {
                let fk_field = nav
                    .foreign_key
                    .as_deref()
                    .unwrap_or(target_entity.key_field());
                let fk_value = obj
                    .get(fk_field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let Some(fk_value) = fk_value else {
                    continue;
                };
                let target_key = target_entity.key_field();
                let found = target_records
                    .into_iter()
                    .find(|r| r.get(target_key).and_then(|v| v.as_str()) == Some(fk_value.as_str()))
                    .unwrap_or(Value::Null);
                obj.insert(nav.name.clone(), found);
            }
        }
    }

    pub fn record_count(&self, arg: &str) -> usize {
        self.store
            .read()
            .unwrap()
            .get(arg)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn expand_record(&self, r: &mut Value, arg: &str) -> Result<(), StoreError> {
        eprintln!("expand record: {:#?}", r);
        Ok(())
    }

    fn query_collection_from(&self, query: &ODataQuery) -> Value {
        let filter = &query.filter;
        info!("query: {:?}", query);
        let x = odata_params::filters::parse_str(&filter.raw);
        info!("parsed filter: {:#?}", x);

        todo!("Implement {query:?}");
        json!([])
    }
}

impl DataStore for InMemoryDataStore {
    fn entities(&self) -> Vec<ODataEntity> {
        self.entities.try_read().unwrap().clone()
    }

    fn get_collection(
        &self,
        set_name: &str,
        query: &ODataQuery,
        parent: Option<&ParentKey>,
        _default: Option<&Value>,
    ) -> Result<Vec<Value>, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let entities_snap = self.entities_snapshot();
        let resolved_entities = self.resolved_entities_snapshot();
        let store = self.store.read().unwrap();
        // let qs = query.to_query_map();
        let expand_names: Vec<String> = query
            .expand
            .iter()
            .map(|e| e.nav_property.clone())
            .collect();
        let expand_refs: Vec<&str> = expand_names.iter().map(|s| s.as_str()).collect();
        match parent {
            Some(parent_ref) => todo!(),
            None => {
                let records: Vec<Value> = store
                    .get(set_name)
                    .unwrap()
                    .iter()
                    .filter(|r| query.filter.eval(*r))
                    .cloned()
                    .collect();

                Ok(records)
            }
        }
    }

    #[tracing::instrument(skip(self, _query, parent))]
    fn count(&self, set_name: &str, _query: &ODataQuery, parent: Option<&ParentKey>) -> usize {
        let store = self.store.read().unwrap();
        info!(".");

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

                let entity = match self.find_entity(set_name) {
                    Some(e) => e,
                    None => return 0,
                };
                let child_fk = parent_entity
                    .navigation_properties()
                    .iter()
                    .find(|np| np.target_type == entity.type_name())
                    .and_then(|np| np.foreign_key)
                    .unwrap_or(parent_key_field);

                store
                    .get(set_name)
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| {
                                r.get(child_fk).and_then(|v| v.as_str()) == Some(parent_key_value)
                            })
                            .count()
                    })
                    .unwrap_or(0)
            }
            None => store.get(set_name).map(|v| v.len()).unwrap_or(0),
        }
    }

    #[tracing::instrument(skip(self, query, key))]
    fn read_record(
        &self,
        set_name: &str,
        key: &EntityKey,
        query: &ODataQuery,
    ) -> Result<Value, StoreError> {
        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;

        let (key_value, _is_active) = self.resolve_key(set_name, key)?;
        let store = self.store.read().unwrap();
        let key_field = entity.key_field();

        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Data for '{}' not found", set_name)))?;
        let mut record = find_record(records, key_field, key_value)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    key_field, key_value
                ))
            })
            .cloned()?;

        inject_odata_context(&mut record, set_name);

        if !query.expand.is_empty() {
            for x in query.expand.iter() {
                if let Some(n) = entity
                    .navigation_properties()
                    .iter()
                    .find(|n| n.name == x.nav_property)
                {
                    let child_entity =
                        self.find_entity_by_type(n.target_type).ok_or_else(|| {
                            StoreError::NotFound(format!("Entity set '{}' not found", set_name))
                        })?;
                    let target_set = child_entity.set_name();
                    let records = store.get(target_set).ok_or_else(|| {
                        StoreError::NotFound(format!(
                            "Entity set '{}' not found for child data",
                            target_set
                        ))
                    })?;
                    if n.is_collection {
                        let rows = find_all_record(records, n.foreign_key.unwrap(), key_value);
                        record
                            .as_object_mut()
                            .unwrap()
                            .insert(n.name.into(), rows.into());
                    } else {
                        let k = record
                            .get(n.foreign_key.unwrap())
                            .unwrap()
                            .as_str()
                            .unwrap();
                        if let Some(r) = find_record(records, "ID", k) {
                            record
                                .as_object_mut()
                                .unwrap()
                                .insert(n.name.into(), r.clone());
                        } else {
                            todo!("Handle missing navigation target record for key '{}'", k);
                        }
                    }
                } else {
                    return Err(StoreError::BadRequest(format!(
                        "Navigation property '{}' not found on entity '{}'",
                        x.nav_property, set_name
                    )));
                }
            }
        }
        // Resolve value_source text fields
        resolve_value_texts(entity, &mut record, &store);
        Ok(record.clone())
    }

    #[tracing::instrument(skip(self, data, parent))]
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
            // Inject parent key if sub-item
            if let Some(parent_ref) = parent {
                if let Some(parent_entity) = self.find_entity(&parent_ref.set_name) {
                    let parent_key_value = parent_ref
                        .key
                        .resolve_key_value(parent_entity.key_field())
                        .unwrap_or("");
                    let child_fk = resolve_child_fk(parent_entity.clone(), entity.clone());
                    obj.entry(child_fk.to_string())
                        .or_insert_with(|| json!(parent_key_value));
                }
            }

            // Generate key if not present
            let key_field = entity.key_field();
            if !obj.contains_key(key_field) {
                obj.insert(key_field.to_string(), json!(Uuid::new_v4().to_string()));
            }

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
            } else if let Some(spec) = entity.entity_spec() {
                for field in &spec.fields {
                    obj.entry(field.name().to_string())
                        .or_insert_with(|| match field.edm_type() {
                            "Edm.Int32" | "Edm.Byte" => json!(0),
                            "Edm.Decimal" => json!("0"),
                            "Edm.Boolean" => json!(false),
                            _ => json!(""),
                        });
                }
            }
        } else {
            return Err(StoreError::BadRequest(
                "Invalid data format: expected JSON object".to_string(),
            ));
        }

        self.store
            .write()
            .unwrap()
            .entry(set_name.to_string())
            .or_default()
            .push(new_record.clone());
        // Computed fields
        entity.compute_fields(&mut new_record);
        inject_odata_context(&mut new_record, set_name);
        Ok(new_record)
    }

    #[tracing::instrument(skip(self, patch, key))]
    fn patch_entity(
        &self,
        set_name: &str,
        key: &EntityKey,
        patch: &Value,
    ) -> Result<Value, StoreError> {
        info!(".");

        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;
        let key_field = entity.key_field();

        let readonly_fields: Vec<&str> = entity
            .fields_def()
            .unwrap_or(&[])
            .iter()
            .filter(|f| f.immutable || f.computed)
            .map(|f| f.name)
            .collect();

        if is_active {
            // Patch baseline directly
            let mut store = self.store.write().unwrap();
            let records = store.get_mut(set_name).ok_or_else(|| {
                StoreError::NotFound(format!("Entity set '{}' not found", set_name))
            })?;
            let record = find_record_mut(records, key_field, key_value).ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    key_field, key_value
                ))
            })?;

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
            entity.compute_fields(record);

            let mut result = record;
            inject_odata_context(&mut result, set_name);
            Ok(result.clone())
        } else {
            Err(StoreError::NotFound(
                "Entity not found in changeset for patching".to_string(),
            ))
        }
    }

    #[tracing::instrument(skip(self))]
    fn delete_entity(&self, set_name: &str, key: &EntityKey) -> Result<(), StoreError> {
        info!(".");

        let entity = self
            .find_entity(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let (key_value, is_active) = self.resolve_key(set_name, key)?;

        if is_active {
            // Delete from baseline directly
            let mut store = self.store.write().unwrap();
            let found = store
                .get(set_name)
                .map(|r| find_record(r, entity.key_field(), key_value).is_some())
                .unwrap_or(false);
            if !found {
                return Err(StoreError::NotFound("Entity not found".to_string()));
            }
            if let Some(records) = store.get_mut(set_name) {
                remove_records(records, entity.key_field(), key_value);
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    fn draft_edit(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn draft_activate(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn draft_prepare(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
    }

    fn read_sibling_entity(&self, set_name: &str, key: &EntityKey) -> Result<Value, StoreError> {
        todo!()
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
        let key_field = entity.key_field();

        let store = self.store.read().unwrap();
        let records = store
            .get(set_name)
            .ok_or_else(|| StoreError::NotFound(format!("Entity set '{}' not found", set_name)))?;
        let record = find_record(records, key_field, key_value)
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "Entity with {}='{}' not found",
                    key_field, key_value
                ))
            })?
            .clone();

        record
            .get(property)
            .cloned()
            .ok_or_else(|| StoreError::NotFound(format!("Property '{}' not found", property)))
    }

    #[tracing::instrument(skip(self))]
    fn get_records(&self, set_name: &str) -> Vec<Value> {
        let mut result = vec![];
        let store = self.store.read().unwrap();
        if let Some(records) = store.get(set_name).cloned() {
            result.extend(records);
        }

        result
    }

    #[tracing::instrument(skip(self))]
    fn commit(&self) {
        info!("  [commit] Persisting data to {}", self.data_dir.display());
        let entities_snap = self.entities_snapshot();
        let store = self.store.read().unwrap();
        for entity in &entities_snap {
            let set_name = entity.set_name();
            if let Some(records) = store.get(set_name) {
                let json_path = self.data_dir.join(format!("{}.json", set_name));
                if let Ok(content) = serde_json::to_string_pretty(records) {
                    if let Err(e) = std::fs::write(&json_path, content) {
                        eprintln!("  WARNING: Could not write {}: {}", json_path.display(), e);
                    }
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn update_entities(&self, new_entities: &[ODataEntity]) {
        // Register any new entity sets that don't have data yet
        let mut store = self.store.write().unwrap();
        for entity in new_entities {
            let set_name = entity.set_name();
            if !store.contains_key(set_name) {
                let records = load_entity_data(set_name, &self.data_dir, entity);
                info!(
                    "inserted entity {} with {} records",
                    set_name,
                    records.len()
                );
                store.insert(set_name.to_string(), records);
            }
        }
        drop(store);
        *self.entities.write().unwrap() = new_entities.iter().cloned().collect::<Vec<_>>();
    }

    fn initialize_records(&self, set_name: &str, records: Vec<Value>) {
        let mut store = self.store.write().unwrap();
        let collection = store.entry(set_name.to_string()).or_insert_with(Vec::new);
        for record in records {
            let id = record.get("ID").and_then(|v| v.as_str()).unwrap_or("");
            let exists = collection
                .iter()
                .any(|r| r.get("ID").and_then(|v| v.as_str()) == Some(id));
            if !exists {
                collection.push(record);
            }
        }
    }
}

// ── Internal helpers ────────────────────────────────────────────────

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

/// Inject SiblingEntity using the changeset overlay model.
/// For an active entity with a copy in changeset → returns the draft record.
/// For a draft entity (in changeset) → returns the baseline record.
/// Otherwise → null.
pub(crate) fn inject_sibling_entity_from_changeset(
    record: &mut Value,
    key_field: &str,
    set_name: &str,
    store: &HashMap<String, Vec<Value>>,
    changeset: &Option<ChangeSet>,
) {
    if let Some(obj) = record.as_object_mut() {
        let is_active = obj
            .get("IsActiveEntity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let key_value = obj.get(key_field).and_then(|v| v.as_str()).unwrap_or("");

        let sibling = if is_active {
            // Active → look for draft sibling in changeset
            changeset
                .as_ref()
                .and_then(|cs| cs.records.get(set_name))
                .and_then(|recs| find_record(recs, key_field, key_value))
                .map(|r| {
                    let mut s = r.clone();
                    let has_active = store
                        .get(set_name)
                        .map(|recs| find_record(recs, key_field, key_value).is_some())
                        .unwrap_or(false);
                    inject_draft_flags(&mut s, false, has_active, false);
                    s
                })
                .unwrap_or(Value::Null)
        } else {
            // Draft → look for active sibling in baseline
            store
                .get(set_name)
                .and_then(|recs| find_record(recs, key_field, key_value))
                .map(|r| {
                    let mut s = r.clone();
                    inject_draft_flags(&mut s, true, false, true);
                    s
                })
                .unwrap_or(Value::Null)
        };
        obj.insert("SiblingEntity".to_string(), sibling);
    }
}

/// Injects SiblingEntity from flat record list (used by pg_store).
#[cfg_attr(not(feature = "postgres"), allow(dead_code))]
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
                records
                    .iter()
                    .find(|r| {
                        r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(!is_active)
                    })
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
    entity: ODataEntity,
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
    parent_entity: ODataEntity,
    child_entity: ODataEntity,
) -> String {
    parent_entity
        .navigation_properties()
        .iter()
        .find(|np| np.target_type == child_entity.type_name())
        .and_then(|np| np.foreign_key)
        .unwrap_or(parent_entity.key_field())
        .to_string()
}

// ── Data loading ────────────────────────────────────────────────────

fn load_entity_data(set_name: &str, data_dir: &Path, entity: &ODataEntity) -> Vec<Value> {
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
                        "  WARNING: {} is not a valid JSON array: {} - falling back to mock_data()",
                        json_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "  WARNING: Could not read {}: {} falling back to mock_data()",
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
    entity.initial_data()
}
