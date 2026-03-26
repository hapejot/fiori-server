use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::entity::ODataEntity;
use crate::BASE_PATH;

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
pub fn query_collection(entity: &dyn ODataEntity, qs: &HashMap<String, String>, entities: &[&dyn ODataEntity]) -> Value {
    query_collection_from(entity, &entity.mock_data(), qs, entities)
}

/// Fuehrt eine OData-Abfrage auf bereits geladenen Daten aus.
pub fn query_collection_from(entity: &dyn ODataEntity, data: &[Value], qs: &HashMap<String, String>, entities: &[&dyn ODataEntity]) -> Value {
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
                entity.expand_record(r, &nav_refs, entities);
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
                        let filtered: serde_json::Map<String, Value> = obj
                            .iter()
                            .filter(|(k, _)| {
                                fields.contains(&k.as_str())
                                    || expanded_names.iter().any(|n| n == k.as_str())
                            })
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
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
