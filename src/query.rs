use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::entity::ODataEntity;
use crate::BASE_PATH;

/// Inject SiblingEntity into a record.
/// For a draft with an active sibling → returns the active record.
/// For an active entity with a draft → returns the draft record.
/// Otherwise → null.
fn inject_sibling_entity(record: &mut Value, key_field: &str, all_records: &[Value]) {
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
                all_records
                    .iter()
                    .find(|r| {
                        r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                == Some(!is_active)
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

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if query.is_empty() {
        return map;
    }
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(
                urlencoding::decode(k).unwrap_or_default().into_owned(),
                urlencoding::decode(v).unwrap_or_default().into_owned(),
            );
        }
    }
    map
}

pub fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    // Boolean comparison
    if let (Some(a_b), Some(b_b)) = (a.as_bool(), b.as_bool()) {
        return a_b.cmp(&b_b);
    }
    if let (Some(a_f), Some(b_f)) = (value_as_f64(a), value_as_f64(b)) {
        a_f.partial_cmp(&b_f).unwrap_or(std::cmp::Ordering::Equal)
    } else {
        let a_s = a.as_str().unwrap_or("");
        let b_s = b.as_str().unwrap_or("");
        a_s.cmp(b_s)
    }
}

pub fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

pub fn match_filter(record: &Value, expr: &str) -> bool {
    static FILTER_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)([\w/]+)\s+(eq|ne|gt|ge|lt|le)\s+(.+)").unwrap());
    static AND_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)\s+and\s+").unwrap());
    static OR_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)\s+or\s+").unwrap());

    // Strip outer parentheses: (expr) -> expr
    let expr = expr.trim();
    let expr = if expr.starts_with('(') && expr.ends_with(')') {
        &expr[1..expr.len() - 1]
    } else {
        expr
    };

    // Handle OR: any branch matching is enough
    let or_branches: Vec<&str> = OR_RE.split(expr).collect();
    if or_branches.len() > 1 {
        return or_branches.iter().any(|branch| match_filter(record, branch.trim()));
    }

    let parts: Vec<&str> = AND_RE.split(expr).collect();

    for part in parts {
        let part = part.trim();
        // Strip parentheses around individual conditions
        let part = if part.starts_with('(') && part.ends_with(')') {
            &part[1..part.len() - 1]
        } else {
            part
        };
        if let Some(caps) = FILTER_RE.captures(part) {
            let field = &caps[1];
            let op = caps[2].to_lowercase();
            let raw_val = caps[3].trim();

            let record_obj = match record.as_object() {
                Some(o) => o,
                None => return false,
            };

            // Navigation property path: SiblingEntity/IsActiveEntity eq null
            if field.contains('/') {
                let nav_parts: Vec<&str> = field.splitn(2, '/').collect();
                // For draft mock: SiblingEntity/IsActiveEntity eq null
                // means "no sibling exists" → HasDraftEntity eq false (for active) or HasActiveEntity eq false (for draft)
                if nav_parts[0] == "SiblingEntity" && op == "eq" && raw_val.eq_ignore_ascii_case("null") {
                    let has_draft = record_obj.get("HasDraftEntity").and_then(|v| v.as_bool()).unwrap_or(false);
                    let is_active = record_obj.get("IsActiveEntity").and_then(|v| v.as_bool()).unwrap_or(true);
                    // SiblingEntity/IsActiveEntity eq null → no sibling at all
                    // Active entity without draft: HasDraftEntity=false → sibling is null ✓
                    // Draft with active: HasActiveEntity=true → sibling exists → not null ✗
                    // Draft without active (new): HasActiveEntity=false → sibling is null ✓
                    let sibling_is_null = if is_active { !has_draft } else {
                        !record_obj.get("HasActiveEntity").and_then(|v| v.as_bool()).unwrap_or(false)
                    };
                    if !sibling_is_null { return false; }
                    continue;
                }
                // Generic nav path: skip (not supported)
                continue;
            }

            let record_val = match record_obj.get(field) {
                Some(v) if !v.is_null() => v,
                _ => {
                    // field not found or null – only matches "eq null"
                    if op == "eq" && raw_val.eq_ignore_ascii_case("null") {
                        continue; // null eq null → true
                    }
                    return false;
                }
            };

            let filter_val: Value = if raw_val.starts_with('\'') && raw_val.ends_with('\'') {
                Value::String(raw_val[1..raw_val.len() - 1].to_string())
            } else if raw_val.eq_ignore_ascii_case("true") {
                Value::Bool(true)
            } else if raw_val.eq_ignore_ascii_case("false") {
                Value::Bool(false)
            } else if raw_val.eq_ignore_ascii_case("null") {
                Value::Null
            } else if let Ok(i) = raw_val.parse::<i64>() {
                Value::Number(i.into())
            } else if let Ok(f) = raw_val.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::String(raw_val.to_string()))
            } else {
                Value::String(raw_val.to_string())
            };

            let cmp = compare_values(record_val, &filter_val);
            let ok = match op.as_str() {
                "eq" => cmp == std::cmp::Ordering::Equal,
                "ne" => cmp != std::cmp::Ordering::Equal,
                "gt" => cmp == std::cmp::Ordering::Greater,
                "ge" => cmp != std::cmp::Ordering::Less,
                "lt" => cmp == std::cmp::Ordering::Less,
                "le" => cmp != std::cmp::Ordering::Greater,
                _ => true,
            };
            if !ok {
                return false;
            }
        }
    }
    true
}

/// Parst $expand-Werte und extrahiert Nav-Property-Namen,
/// ignoriert geklammerte Sub-Optionen wie ($select=DraftUUID,InProcessByUser).
pub fn parse_expand_names(expand: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in expand.chars() {
        match ch {
            '(' => {
                depth += 1;
            }
            ')' => {
                depth -= 1;
            }
            ',' if depth == 0 => {
                let name = current.trim().to_string();
                if !name.is_empty() {
                    names.push(name);
                }
                current.clear();
            }
            _ if depth == 0 => {
                current.push(ch);
            }
            _ => {} // inside parentheses → skip
        }
    }
    let name = current.trim().to_string();
    if !name.is_empty() {
        names.push(name);
    }
    names
}

/// Fuehrt eine OData-Abfrage auf den Mock-Daten einer Entitaet aus
/// ($filter, $orderby, $skip, $top, $expand, $select, $count).
pub fn query_collection(entity: &dyn ODataEntity, qs: &HashMap<String, String>, entities: &[&dyn ODataEntity], data_store: &HashMap<String, Vec<Value>>) -> Value {
    query_collection_from(entity, &entity.mock_data(), qs, entities, data_store)
}

/// Fuehrt eine OData-Abfrage auf bereits geladenen Daten aus.
pub fn query_collection_from(entity: &dyn ODataEntity, data: &[Value], qs: &HashMap<String, String>, entities: &[&dyn ODataEntity], data_store: &HashMap<String, Vec<Value>>) -> Value {
    let mut results: Vec<Value> = data.to_vec();

    // $filter
    if let Some(filter_expr) = qs.get("$filter") {
        if !filter_expr.is_empty() {
            results.retain(|r| match_filter(r, filter_expr));
        }
    }

    // $orderby
    if let Some(orderby) = qs.get("$orderby") {
        if !orderby.is_empty() {
            let parts: Vec<&str> = orderby.split_whitespace().collect();
            let field = parts[0];
            let desc = parts
                .get(1)
                .map(|s| s.eq_ignore_ascii_case("desc"))
                .unwrap_or(false);
            results.sort_by(|a, b| {
                let va = a.get(field).unwrap_or(&Value::Null);
                let vb = b.get(field).unwrap_or(&Value::Null);
                let cmp = compare_values(va, vb);
                if desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
    }

    let total = results.len();

    // $skip / $top
    let skip: usize = qs.get("$skip").and_then(|s| s.parse().ok()).unwrap_or(0);
    let top: usize = qs
        .get("$top")
        .and_then(|s| s.parse().ok())
        .unwrap_or(results.len());
    results = results.into_iter().skip(skip).take(top).collect();

    // $expand — parse nav names, handling nested options like DraftAdministrativeData($select=...)
    if let Some(expand) = qs.get("$expand") {
        if !expand.is_empty() {
            let nav_names = parse_expand_names(expand);
            let nav_refs: Vec<&str> = nav_names.iter().map(|s| s.as_str()).collect();
            for r in &mut results {
                entity.expand_record(r, &nav_refs, entities, data_store);
                // DraftAdministrativeData: inject null for active, minimal object for drafts
                if nav_refs.iter().any(|n| *n == "DraftAdministrativeData") {
                    if let Some(obj) = r.as_object_mut() {
                        let is_draft = obj.get("IsActiveEntity")
                            .and_then(|v| v.as_bool()) == Some(false);
                        if is_draft {
                            obj.insert("DraftAdministrativeData".to_string(), json!({
                                "DraftUUID": format!("draft-{}", obj.get(entity.key_field()).and_then(|v| v.as_str()).unwrap_or("unknown")),
                                "InProcessByUser": ""
                            }));
                        } else {
                            obj.entry("DraftAdministrativeData".to_string())
                                .or_insert(Value::Null);
                        }
                    }
                }
                // SiblingEntity: inject the active/draft counterpart
                if nav_refs.iter().any(|n| *n == "SiblingEntity") {
                    let all_records = data_store.get(&entity.entity_set()).map(|v| v.as_slice()).unwrap_or(&[]);
                    inject_sibling_entity(r, entity.key_field(), all_records);
                }
            }
        }
    }

    // Resolve value_source text fields (_{name}_text) from FieldValueListItems
    if let Some(fields) = entity.fields_def() {
        let vs_fields: Vec<(&str, &str, &str)> = fields
            .iter()
            .filter_map(|f| {
                let vs = f.value_source?;
                let tp = f.text_path?;
                Some((f.name, vs, tp))
            })
            .collect();
        if !vs_fields.is_empty() {
            if let Some(items) = data_store.get("FieldValueListItems") {
                // Build lookup: (ListID, Code) → Description
                let lookup: HashMap<(&str, &str), &str> = items
                    .iter()
                    .filter_map(|item| {
                        let list_id = item.get("ListID")?.as_str()?;
                        let code = item.get("Code")?.as_str()?;
                        let desc = item.get("Description")?.as_str()?;
                        Some(((list_id, code), desc))
                    })
                    .collect();
                for r in &mut results {
                    if let Some(obj) = r.as_object_mut() {
                        for &(field_name, list_id, text_field) in &vs_fields {
                            if let Some(code) = obj.get(field_name).and_then(|v| v.as_str()) {
                                let desc = lookup
                                    .get(&(list_id, code))
                                    .copied()
                                    .unwrap_or(code);
                                obj.insert(
                                    text_field.to_string(),
                                    Value::String(desc.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // $select — keep expanded nav properties too
    let expanded_names: Vec<String> = qs.get("$expand")
        .map(|e| parse_expand_names(e))
        .unwrap_or_default();
    if let Some(select) = qs.get("$select") {
        if !select.is_empty() {
            let fields: Vec<&str> = select.split(',').map(|s| s.trim()).collect();
            results = results
                .into_iter()
                .map(|r| {
                    if let Some(obj) = r.as_object() {
                        let mut filtered = serde_json::Map::new();
                        // Include requested fields, defaulting to null if absent
                        for &f in &fields {
                            let val = obj.get(f).cloned().unwrap_or(Value::Null);
                            filtered.insert(f.to_string(), val);
                        }
                        // Keep expanded nav properties
                        for nav in &expanded_names {
                            if let Some(v) = obj.get(nav.as_str()) {
                                filtered.insert(nav.clone(), v.clone());
                            }
                        }
                        Value::Object(filtered)
                    } else {
                        r
                    }
                })
                .collect();
        }
    }

    let mut body = json!({
        "@odata.context": format!("{}/$metadata#{}", BASE_PATH, entity.set_name()),
        "value": results
    });

    if qs
        .get("$count")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        body["@odata.count"] = json!(total);
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct TestEntity;
    impl ODataEntity for TestEntity {
        fn set_name(&self) -> &'static str { "Tests" }
        fn key_field(&self) -> &'static str { "ID" }
        fn type_name(&self) -> &'static str { "Test" }
        fn mock_data(&self) -> Vec<Value> { vec![] }
        fn entity_set(&self) -> String { String::new() }
        fn fields_def(&self) -> Option<&'static [FieldDef]> {
            static FIELDS: &[FieldDef] = &[
                FieldDef { name: "ID", label: "ID", edm_type: "Edm.String", max_length: Some(10), precision: None, scale: None, immutable: true, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
                FieldDef { name: "Name", label: "Name", edm_type: "Edm.String", max_length: Some(40), precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
                FieldDef { name: "Extra", label: "Extra", edm_type: "Edm.String", max_length: Some(40), precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
            ];
            Some(FIELDS)
        }
    }

    fn make_qs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn empty_ds() -> HashMap<String, Vec<Value>> { HashMap::new() }

    // ── $select tests ───────────────────────────────────────────

    #[test]
    fn select_returns_only_requested_fields() {
        let entity = TestEntity;
        let data = vec![json!({"ID": "1", "Name": "Alice", "Extra": "x"})];
        let qs = make_qs(&[("$select", "ID,Name")]);
        let result = query_collection_from(&entity, &data, &qs, &[], &empty_ds());
        let row = &result["value"][0];
        assert_eq!(row["ID"], "1");
        assert_eq!(row["Name"], "Alice");
        assert!(row.get("Extra").is_none());
    }

    #[test]
    fn select_includes_missing_fields_as_null() {
        let entity = TestEntity;
        // Record that does NOT have the "Extra" key at all
        let data = vec![json!({"ID": "1", "Name": "Alice"})];
        let qs = make_qs(&[("$select", "ID,Name,Extra")]);
        let result = query_collection_from(&entity, &data, &qs, &[], &empty_ds());
        let row = &result["value"][0];
        assert_eq!(row["ID"], "1");
        assert_eq!(row["Name"], "Alice");
        // Extra must be present as null, not absent
        assert!(row.get("Extra").is_some(), "missing field should be included");
        assert!(row["Extra"].is_null(), "missing field should be null");
    }

    #[test]
    fn select_preserves_expanded_nav_properties() {
        let entity = TestEntity;
        let mut data = vec![json!({"ID": "1", "Name": "Alice", "Children": [{"x": 1}]})];
        let qs = make_qs(&[("$select", "ID"), ("$expand", "Children")]);
        let result = query_collection_from(&entity, &data, &qs, &[], &empty_ds());
        let row = &result["value"][0];
        assert_eq!(row["ID"], "1");
        assert!(row.get("Children").is_some(), "expanded nav should be kept");
        assert!(row.get("Name").is_none(), "non-selected field should be removed");
    }

    #[test]
    fn select_empty_string_returns_all_fields() {
        let entity = TestEntity;
        let data = vec![json!({"ID": "1", "Name": "Alice", "Extra": "x"})];
        let qs = make_qs(&[("$select", "")]);
        let result = query_collection_from(&entity, &data, &qs, &[], &empty_ds());
        let row = &result["value"][0];
        assert_eq!(row["ID"], "1");
        assert_eq!(row["Name"], "Alice");
        assert_eq!(row["Extra"], "x");
    }

    #[test]
    fn select_without_param_returns_all_fields() {
        let entity = TestEntity;
        let data = vec![json!({"ID": "1", "Name": "Alice", "Extra": "x"})];
        let qs = make_qs(&[]);
        let result = query_collection_from(&entity, &data, &qs, &[], &empty_ds());
        let row = &result["value"][0];
        assert_eq!(row["ID"], "1");
        assert_eq!(row["Name"], "Alice");
        assert_eq!(row["Extra"], "x");
    }

    // ── parse_expand_names tests ────────────────────────────────

    #[test]
    fn parse_expand_simple() {
        assert_eq!(parse_expand_names("Items,Details"), vec!["Items", "Details"]);
    }

    #[test]
    fn parse_expand_with_nested_options() {
        let names = parse_expand_names("DraftAdministrativeData($select=DraftUUID,InProcessByUser),Items");
        assert_eq!(names, vec!["DraftAdministrativeData", "Items"]);
    }

    #[test]
    fn parse_expand_empty() {
        assert!(parse_expand_names("").is_empty());
    }

    // ── parse_query_string tests ────────────────────────────────

    #[test]
    fn parse_query_string_basic() {
        let qs = parse_query_string("$filter=Name eq 'X'&$top=10");
        assert_eq!(qs.get("$filter").unwrap(), "Name eq 'X'");
        assert_eq!(qs.get("$top").unwrap(), "10");
    }

    #[test]
    fn parse_query_string_empty() {
        assert!(parse_query_string("").is_empty());
    }

    // ── match_filter tests ──────────────────────────────────────

    #[test]
    fn filter_eq_string() {
        let rec = json!({"Name": "Alice"});
        assert!(match_filter(&rec, "Name eq 'Alice'"));
        assert!(!match_filter(&rec, "Name eq 'Bob'"));
    }

    #[test]
    fn filter_ne() {
        let rec = json!({"Status": "A"});
        assert!(match_filter(&rec, "Status ne 'B'"));
        assert!(!match_filter(&rec, "Status ne 'A'"));
    }

    #[test]
    fn filter_numeric_comparison() {
        let rec = json!({"Price": 100});
        assert!(match_filter(&rec, "Price gt 50"));
        assert!(!match_filter(&rec, "Price lt 50"));
        assert!(match_filter(&rec, "Price ge 100"));
        assert!(match_filter(&rec, "Price le 100"));
    }

    #[test]
    fn filter_and_combination() {
        let rec = json!({"Name": "Alice", "Status": "A"});
        assert!(match_filter(&rec, "Name eq 'Alice' and Status eq 'A'"));
        assert!(!match_filter(&rec, "Name eq 'Alice' and Status eq 'B'"));
    }

    #[test]
    fn filter_contains_passes_through() {
        // contains() is not parsed by the regex-based filter — it falls through as true
        let rec = json!({"Name": "Alice Wonder"});
        assert!(match_filter(&rec, "contains(Name,'Wonder')"));
        assert!(match_filter(&rec, "contains(Name,'Bob')"));
    }

    #[test]
    fn filter_boolean_eq() {
        let rec = json!({"IsActive": true});
        assert!(match_filter(&rec, "IsActive eq true"));
        assert!(!match_filter(&rec, "IsActive eq false"));
    }

    // ── compare_values tests ────────────────────────────────────

    #[test]
    fn compare_numbers() {
        assert_eq!(compare_values(&json!(1), &json!(2)), std::cmp::Ordering::Less);
        assert_eq!(compare_values(&json!(3), &json!(3)), std::cmp::Ordering::Equal);
    }

    #[test]
    fn compare_strings() {
        assert_eq!(compare_values(&json!("a"), &json!("b")), std::cmp::Ordering::Less);
    }

    #[test]
    fn compare_booleans() {
        assert_eq!(compare_values(&json!(false), &json!(true)), std::cmp::Ordering::Less);
    }
}
