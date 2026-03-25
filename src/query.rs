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
        LazyLock::new(|| regex::Regex::new(r"(?i)(\w+)\s+(eq|ne|gt|ge|lt|le)\s+(.+)").unwrap());
    static AND_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)\s+and\s+").unwrap());

    let parts: Vec<&str> = AND_RE.split(expr).collect();

    for part in parts {
        let part = part.trim();
        if let Some(caps) = FILTER_RE.captures(part) {
            let field = &caps[1];
            let op = caps[2].to_lowercase();
            let raw_val = caps[3].trim();

            let record_obj = match record.as_object() {
                Some(o) => o,
                None => return false,
            };
            let record_val = match record_obj.get(field) {
                Some(v) if !v.is_null() => v,
                _ => return false,
            };

            let filter_val: Value = if raw_val.starts_with('\'') && raw_val.ends_with('\'') {
                Value::String(raw_val[1..raw_val.len() - 1].to_string())
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

/// Fuehrt eine OData-Abfrage auf den Mock-Daten einer Entitaet aus
/// ($filter, $orderby, $skip, $top, $expand, $select, $count).
pub fn query_collection(entity: &dyn ODataEntity, qs: &HashMap<String, String>, entities: &[&dyn ODataEntity]) -> Value {
    let mut results = entity.mock_data();

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

    // $expand
    if let Some(expand) = qs.get("$expand") {
        if !expand.is_empty() {
            let nav_names: Vec<&str> = expand.split(',').map(|s| s.trim()).collect();
            for r in &mut results {
                entity.expand_record(r, &nav_names, entities);
            }
        }
    }

    // $select
    if let Some(select) = qs.get("$select") {
        if !select.is_empty() {
            let fields: Vec<&str> = select.split(',').map(|s| s.trim()).collect();
            results = results
                .into_iter()
                .map(|r| {
                    if let Some(obj) = r.as_object() {
                        let filtered: serde_json::Map<String, Value> = obj
                            .iter()
                            .filter(|(k, _)| fields.contains(&k.as_str()))
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
