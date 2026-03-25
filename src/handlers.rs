use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::{json, Value};
use std::path::Path;

use crate::app_state::AppState;
use crate::entity::extract_set_name;
use crate::query::{parse_query_string, query_collection, query_collection_from};
use crate::BASE_PATH;

fn cors_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Accept, Authorization, OData-Version, OData-MaxVersion, X-Requested-With"),
        ("Access-Control-Expose-Headers", "OData-Version"),
    ]
}

pub fn json_response(data: Value) -> HttpResponse {
    let body = serde_json::to_string_pretty(&data).unwrap_or_default();
    let mut builder = HttpResponse::Ok();
    builder.insert_header((
        "Content-Type",
        "application/json;odata.metadata=minimal;charset=utf-8",
    ));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(body)
}

pub fn error_response(code: u16, message: &str) -> HttpResponse {
    let body = json!({"error": {"code": code.to_string(), "message": message}});
    let mut builder = match code {
        404 => HttpResponse::NotFound(),
        405 => HttpResponse::MethodNotAllowed(),
        400 => HttpResponse::BadRequest(),
        403 => HttpResponse::Forbidden(),
        _ => HttpResponse::InternalServerError(),
    };
    builder.insert_header(("Content-Type", "application/json;charset=utf-8"));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.json(body)
}

pub async fn options_handler() -> HttpResponse {
    let mut builder = HttpResponse::Ok();
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.finish()
}

pub async fn metadata_handler(data: web::Data<AppState>) -> HttpResponse {
    let mut builder = HttpResponse::Ok();
    builder.insert_header(("Content-Type", "application/xml;charset=utf-8"));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(data.metadata_xml.clone())
}

/// Service-Dokument – wird dynamisch aus der Entity-Registry erzeugt.
pub async fn service_document(data: web::Data<AppState>) -> HttpResponse {
    let sets: Vec<Value> = data.entities
        .iter()
        .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
        .collect();
    json_response(json!({
        "@odata.context": format!("{}/$metadata", BASE_PATH),
        "value": sets
    }))
}

/// Generischer Collection-Handler fuer beliebige EntitySets.
pub async fn collection_handler(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(set_name) = extract_set_name(req.path()) {
        if let Some(entity) = data.find_entity(set_name) {
            let qs = parse_query_string(req.query_string());
            let store = data.data_store.read().unwrap();
            if let Some(records) = store.get(set_name) {
                return json_response(query_collection_from(entity, records, &qs, &data.entities));
            }
            return json_response(query_collection(entity, &qs, &data.entities));
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer $count-Handler fuer beliebige EntitySets.
pub async fn count_handler(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(set_name) = extract_set_name(req.path()) {
        if data.find_entity(set_name).is_some() {
            let store = data.data_store.read().unwrap();
            let count = store.get(set_name).map(|v| v.len()).unwrap_or(0);
            let mut builder = HttpResponse::Ok();
            builder.insert_header(("Content-Type", "text/plain;charset=utf-8"));
            builder.insert_header(("OData-Version", "4.0"));
            for (k, v) in cors_headers() {
                builder.insert_header((k, v));
            }
            return builder.body(count.to_string());
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer Single-Entity-Handler: /SetName('key') or /SetName(Key='val',IsActiveEntity=true)
pub async fn single_entity_handler(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let path = req.path();
    let qs = parse_query_string(req.query_string());
    let store = state.data_store.read().unwrap();

    for entity in state.entities.iter() {
        let prefix = format!("{}/{}", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            // Parse the key part: ('val') or (Key='val',IsActiveEntity=true)
            if let Some(keys) = parse_entity_key(rest, entity.key_field()) {
                let key_value = &keys.key_value;
                let is_active = keys.is_active;
                let records = store.get(entity.set_name());
                let data = records.map(|r| r.as_slice());
                if let Some(record) = data.and_then(|d| {
                    d.iter().find(|r| {
                        r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active)
                    })
                }) {
                    let mut result = record.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(
                            "@odata.context".to_string(),
                            json!(format!(
                                "{}/$metadata#{}/$entity",
                                BASE_PATH,
                                entity.set_name()
                            )),
                        );
                    }
                    // $expand
                    if let Some(expand) = qs.get("$expand") {
                        if !expand.is_empty() {
                            let nav_names: Vec<&str> =
                                expand.split(',').map(|s| s.trim()).collect();
                            entity.expand_record(&mut result, &nav_names, &state.entities);
                        }
                    }
                    return json_response(result);
                }
                return error_response(
                    404,
                    &format!(
                        "Entity with {}='{}' not found.",
                        entity.key_field(),
                        key_value
                    ),
                );
            }
        }
    }
    error_response(404, "Entity not found.")
}

/// Parsed entity key information aus dem URL-Rest nach dem SetName.
struct EntityKeyInfo {
    key_value: String,
    is_active: bool,
}

/// Parst den Key-Teil aus dem URL: ('val'), (Key='val',IsActiveEntity=true), etc.
/// Gibt None zurueck wenn das Format nicht passt.
fn parse_entity_key(rest: &str, key_field: &str) -> Option<EntityKeyInfo> {
    // Strip optional action suffix: /Namespace.actionName
    let key_part = rest.split('/').next().unwrap_or(rest);

    if key_part.starts_with("('") && key_part.ends_with("')") {
        // Simple key: ('P001')
        let key_value = key_part[2..key_part.len() - 2].to_string();
        return Some(EntityKeyInfo {
            key_value,
            is_active: true,
        });
    }
    if key_part.starts_with('(') && key_part.ends_with(')') {
        // Composite key: (ProductID='P001',IsActiveEntity=true)
        let inner = &key_part[1..key_part.len() - 1];
        let mut key_value = String::new();
        let mut is_active = true;
        for part in inner.split(',') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim();
                let v = v.trim();
                if k == key_field {
                    // Strip quotes
                    key_value = v.trim_matches('\'').to_string();
                } else if k == "IsActiveEntity" {
                    is_active = v.eq_ignore_ascii_case("true");
                }
            }
        }
        if !key_value.is_empty() {
            return Some(EntityKeyInfo {
                key_value,
                is_active,
            });
        }
    }
    None
}

/// Generischer PATCH-Handler: /SetName(key) – aktualisiert Felder in-memory.
pub async fn patch_entity_handler(
    req: HttpRequest,
    body: web::Json<Value>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let path = req.path();

    for entity in state.entities.iter() {
        let prefix = format!("{}/{}", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            if let Some(keys) = parse_entity_key(rest, entity.key_field()) {
                let key_value = &keys.key_value;
                let is_active = keys.is_active;
                let mut store = state.data_store.write().unwrap();
                if let Some(records) = store.get_mut(entity.set_name()) {
                    if let Some(record) = records.iter_mut().find(|r| {
                        r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value)
                            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active)
                    }) {
                        // Nur nicht-immutable Felder aktualisieren
                        let immutable_fields: Vec<&str> = entity
                            .fields_def()
                            .unwrap_or(&[])
                            .iter()
                            .filter(|f| f.immutable)
                            .map(|f| f.name)
                            .collect();

                        if let Some(patch_obj) = body.as_object() {
                            if let Some(rec_obj) = record.as_object_mut() {
                                for (key, value) in patch_obj {
                                    // Draft-keys und immutable Felder nicht aendern
                                    if key == "IsActiveEntity" || key == "HasActiveEntity" || key == "HasDraftEntity" {
                                        continue;
                                    }
                                    if !immutable_fields.contains(&key.as_str()) {
                                        rec_obj.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                        }

                        let mut result = record.clone();
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert(
                                "@odata.context".to_string(),
                                json!(format!(
                                    "{}/$metadata#{}/$entity",
                                    BASE_PATH,
                                    entity.set_name()
                                )),
                            );
                        }
                        return json_response(result);
                    }
                    return error_response(
                        404,
                        &format!("Entity with {}='{}' not found.", entity.key_field(), key_value),
                    );
                }
            }
        }
    }
    error_response(404, "Entity not found.")
}

/// Handler fuer Draft-Actions: draftEdit, draftActivate, draftPrepare.
/// POST /SetName(key)/Namespace.actionName
pub async fn draft_action_handler(
    req: HttpRequest,
    _body: web::Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    let path = req.path().trim_end_matches('/');
    for entity in state.entities.iter() {
        let set_prefix = format!("{}/{}(", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            if let Some(paren_end) = rest.find(")/") {
                let key_str = &rest[..paren_end];
                let action_part = &rest[paren_end + 2..];
                let key_info = match parse_entity_key(
                    &format!("({})", key_str),
                    entity.key_field(),
                ) {
                    Some(k) => k,
                    None => return error_response(400, "Invalid entity key"),
                };
                // action_part is like "ProductsService.draftEdit"
                let action_name = action_part
                    .rsplit('.')
                    .next()
                    .unwrap_or(action_part);
                let key_value = &key_info.key_value;
                let key_field = entity.key_field();
                let set_name = entity.set_name();

                match action_name {
                    "draftEdit" => {
                        let mut store = state.data_store.write().unwrap();
                        if let Some(records) = store.get_mut(set_name) {
                            let active_record = records.iter_mut().find(|r| {
                                r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
                                    && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                        == Some(true)
                            });
                            if let Some(active) = active_record {
                                if let Some(obj) = active.as_object_mut() {
                                    obj.insert(
                                        "HasDraftEntity".to_string(),
                                        Value::Bool(true),
                                    );
                                }
                                let mut draft = active.clone();
                                if let Some(obj) = draft.as_object_mut() {
                                    obj.insert(
                                        "IsActiveEntity".to_string(),
                                        Value::Bool(false),
                                    );
                                    obj.insert(
                                        "HasActiveEntity".to_string(),
                                        Value::Bool(true),
                                    );
                                    obj.insert(
                                        "HasDraftEntity".to_string(),
                                        Value::Bool(false),
                                    );
                                    obj.insert(
                                        "@odata.context".to_string(),
                                        json!(format!(
                                            "{}/$metadata#{}/$entity",
                                            BASE_PATH, set_name
                                        )),
                                    );
                                }
                                let result = draft.clone();
                                records.push(draft);
                                return json_response(result);
                            }
                            return error_response(404, "Active entity not found.");
                        }
                        return error_response(404, "Entity set not found.");
                    }
                    "draftActivate" => {
                        let mut store = state.data_store.write().unwrap();
                        if let Some(records) = store.get_mut(set_name) {
                            let draft_data = records
                                .iter()
                                .find(|r| {
                                    r.get(key_field).and_then(|v| v.as_str())
                                        == Some(key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            == Some(false)
                                })
                                .cloned();
                            if let Some(draft) = draft_data {
                                if let Some(active) = records.iter_mut().find(|r| {
                                    r.get(key_field).and_then(|v| v.as_str())
                                        == Some(key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            == Some(true)
                                }) {
                                    if let (Some(draft_obj), Some(active_obj)) =
                                        (draft.as_object(), active.as_object_mut())
                                    {
                                        for (k, v) in draft_obj {
                                            if k != "IsActiveEntity"
                                                && k != "HasActiveEntity"
                                                && k != "HasDraftEntity"
                                                && !k.starts_with("@odata")
                                            {
                                                active_obj.insert(k.clone(), v.clone());
                                            }
                                        }
                                        active_obj.insert(
                                            "HasDraftEntity".to_string(),
                                            Value::Bool(false),
                                        );
                                    }
                                    let mut result = active.clone();
                                    if let Some(obj) = result.as_object_mut() {
                                        obj.insert(
                                            "@odata.context".to_string(),
                                            json!(format!(
                                                "{}/$metadata#{}/$entity",
                                                BASE_PATH, set_name
                                            )),
                                        );
                                    }
                                    records.retain(|r| {
                                        !(r.get(key_field).and_then(|v| v.as_str())
                                            == Some(key_value)
                                            && r.get("IsActiveEntity")
                                                .and_then(|v| v.as_bool())
                                                == Some(false))
                                    });
                                    return json_response(result);
                                }
                            }
                            return error_response(404, "Draft entity not found.");
                        }
                        return error_response(404, "Entity set not found.");
                    }
                    "draftPrepare" => {
                        let store = state.data_store.read().unwrap();
                        if let Some(records) = store.get(set_name) {
                            if let Some(record) = records.iter().find(|r| {
                                r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
                                    && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                        == Some(key_info.is_active)
                            }) {
                                let mut result = record.clone();
                                if let Some(obj) = result.as_object_mut() {
                                    obj.insert(
                                        "@odata.context".to_string(),
                                        json!(format!(
                                            "{}/$metadata#{}/$entity",
                                            BASE_PATH, set_name
                                        )),
                                    );
                                }
                                return json_response(result);
                            }
                        }
                        return error_response(404, "Entity not found for draftPrepare.");
                    }
                    _ => {
                        return error_response(
                            400,
                            &format!("Unknown action: {}", action_name),
                        );
                    }
                }
            }
        }
    }
    error_response(404, "Entity not found for action.")
}

// ── $batch handler ──────────────────────────────────────────────────
pub async fn batch_handler(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let batch_boundary = content_type
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("boundary=")
        })
        .unwrap_or("");

    if batch_boundary.is_empty() {
        return error_response(400, "Missing batch boundary");
    }

    let raw_body = String::from_utf8_lossy(&body);
    let mut response_parts = Vec::new();
    let separator = format!("--{}", batch_boundary);

    for segment in raw_body.split(&separator) {
        let segment = segment.trim();
        if segment.is_empty() || segment == "--" {
            continue;
        }

        if segment.contains("multipart/mixed") {
            let cs_boundary = segment
                .lines()
                .find_map(|line| {
                    if line.contains("boundary=") {
                        line.split(';').find_map(|tok| {
                            let tok = tok.trim();
                            tok.strip_prefix("boundary=")
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            if !cs_boundary.is_empty() {
                // Changeset-Segmente verarbeiten (POST/PATCH innerhalb)
                let cs_separator = format!("--{}", cs_boundary);
                let mut cs_response_parts = Vec::new();
                for cs_segment in segment.split(&cs_separator) {
                    let cs_segment = cs_segment.trim();
                    if cs_segment.is_empty() || cs_segment == "--" {
                        continue;
                    }
                    let cs_request_line = cs_segment.lines().find(|l| {
                        l.starts_with("GET ") || l.starts_with("POST ") || l.starts_with("PATCH ") || l.starts_with("DELETE ")
                    });
                    if let Some(cs_req_line) = cs_request_line {
                        let cs_parts: Vec<&str> = cs_req_line.split_whitespace().collect();
                        let cs_method = cs_parts.first().copied().unwrap_or("");
                        let cs_rel_url = cs_parts.get(1).copied().unwrap_or("");
                        let cs_body = extract_batch_body(cs_segment);

                        let (cs_status, cs_resp_json) = match cs_method {
                            "GET" => (200, handle_batch_get(cs_rel_url, &data)),
                            "PATCH" => handle_batch_patch(cs_rel_url, &cs_body, &data),
                            "POST" => handle_batch_post(cs_rel_url, &cs_body, &data),
                            _ => (200, json!({})),
                        };
                        let cs_resp_body = serde_json::to_string(&cs_resp_json).unwrap_or_default();
                        cs_response_parts.push(format!(
                            "Content-Type: application/http\r\n\
                             Content-Transfer-Encoding: binary\r\n\
                             \r\n\
                             HTTP/1.1 {} OK\r\n\
                             Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                             OData-Version: 4.0\r\n\
                             Content-Length: {}\r\n\
                             \r\n\
                             {}",
                            cs_status,
                            cs_resp_body.len(),
                            cs_resp_body
                        ));
                    }
                }
                if cs_response_parts.is_empty() {
                    let cs_resp = format!("--{}--\r\n", cs_boundary);
                    let part_resp = format!(
                        "Content-Type: multipart/mixed; boundary={}\r\nContent-Length: {}\r\n\r\n{}",
                        cs_boundary,
                        cs_resp.len(),
                        cs_resp
                    );
                    response_parts.push(part_resp);
                } else {
                    let cs_inner = cs_response_parts
                        .iter()
                        .map(|p| format!("--{}\r\n{}", cs_boundary, p))
                        .collect::<Vec<_>>()
                        .join("\r\n");
                    let cs_full = format!("{}\r\n--{}--\r\n", cs_inner, cs_boundary);
                    let part_resp = format!(
                        "Content-Type: multipart/mixed; boundary={}\r\nContent-Length: {}\r\n\r\n{}",
                        cs_boundary,
                        cs_full.len(),
                        cs_full
                    );
                    response_parts.push(part_resp);
                }
            }
            continue;
        }

        let lines: Vec<&str> = segment.lines().collect();
        // Finde die Request-Zeile (GET, POST, PATCH, etc.)
        let request_line = lines.iter().find(|l| {
            l.starts_with("GET ") || l.starts_with("POST ") || l.starts_with("PATCH ")
        });
        if let Some(request_line) = request_line {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let method = parts.first().copied().unwrap_or("");
            let rel_url = parts.get(1).copied().unwrap_or("");

            // Body aus dem Segment extrahieren (fuer POST/PATCH)
            let segment_body = extract_batch_body(segment);

            let (status, resp_json) = match method {
                "GET" => (200, handle_batch_get(rel_url, &data)),
                "PATCH" => handle_batch_patch(rel_url, &segment_body, &data),
                "POST" => handle_batch_post(rel_url, &segment_body, &data),
                _ => (200, handle_batch_get(rel_url, &data)),
            };

            let resp_body = serde_json::to_string(&resp_json).unwrap_or_default();

            let part_resp = format!(
                "Content-Type: application/http\r\n\
                 Content-Transfer-Encoding: binary\r\n\
                 \r\n\
                 HTTP/1.1 {} OK\r\n\
                 Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                 OData-Version: 4.0\r\n\
                 Content-Length: {}\r\n\
                 \r\n\
                 {}",
                status,
                resp_body.len(),
                resp_body
            );
            response_parts.push(part_resp);
        }
    }

    let resp_boundary = format!("batch_resp_{}", std::process::id());
    let mut body_parts = Vec::new();
    for rp in &response_parts {
        body_parts.push(format!("--{}\r\n{}", resp_boundary, rp));
    }
    body_parts.push(format!("--{}--\r\n", resp_boundary));
    let full_body = body_parts.join("\r\n");

    let mut builder = HttpResponse::Ok();
    builder.insert_header((
        "Content-Type",
        format!("multipart/mixed; boundary={}", resp_boundary),
    ));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(full_body)
}

/// Extrahiert den JSON-Body aus einem Batch-Segment (nach der Leerzeile).
fn extract_batch_body(segment: &str) -> String {
    // Body kommt nach einer Leerzeile (doppelte Newline)
    if let Some(idx) = segment.find("\r\n\r\n") {
        let after_headers = &segment[idx + 4..];
        // Es koennte nochmal Headers + Leerzeile geben (HTTP request line + headers)
        if let Some(idx2) = after_headers.find("\r\n\r\n") {
            return after_headers[idx2 + 4..].trim().to_string();
        }
        return after_headers.trim().to_string();
    }
    if let Some(idx) = segment.find("\n\n") {
        let after_headers = &segment[idx + 2..];
        if let Some(idx2) = after_headers.find("\n\n") {
            return after_headers[idx2 + 2..].trim().to_string();
        }
        return after_headers.trim().to_string();
    }
    String::new()
}

/// Batch-PATCH: aktualisiert ein einzelnes Entity im data_store.
fn handle_batch_patch(rel_url: &str, body: &str, state: &web::Data<AppState>) -> (u16, Value) {
    let entities = &state.entities;
    let full_path = if rel_url.starts_with('/') {
        rel_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, rel_url)
    };
    let path = full_path.split('?').next().unwrap_or(&full_path).trim_end_matches('/');

    let patch_data: Value = serde_json::from_str(body).unwrap_or(json!({}));

    for entity in entities.iter() {
        let set_prefix = format!("{}/{}(", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            if let Some(key_str) = rest.strip_suffix(')') {
                let key_info = match parse_entity_key(&format!("({})", key_str), entity.key_field()) {
                    Some(k) => k,
                    None => return (400, json!({"error": {"code": "400", "message": "Invalid key"}})),
                };
                let mut store = state.data_store.write().unwrap();
                if let Some(records) = store.get_mut(entity.set_name()) {
                    if let Some(record) = records.iter_mut().find(|r| {
                        let key_match = r.get(entity.key_field()).and_then(|v| v.as_str())
                            == Some(&key_info.key_value);
                        let active_match = r
                            .get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                            == key_info.is_active;
                        key_match && active_match
                    }) {
                        if let (Some(rec_obj), Some(patch_obj)) =
                            (record.as_object_mut(), patch_data.as_object())
                        {
                            for (k, v) in patch_obj {
                                if k == "IsActiveEntity"
                                    || k == "HasActiveEntity"
                                    || k == "HasDraftEntity"
                                {
                                    continue;
                                }
                                rec_obj.insert(k.clone(), v.clone());
                            }
                        }
                        return (200, record.clone());
                    }
                }
                return (404, json!({"error": {"code": "404", "message": "Not found"}}));
            }
        }
    }
    (404, json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}))
}

/// Batch-POST: behandelt Aktionen (draftEdit, draftActivate, draftPrepare) innerhalb von $batch.
fn handle_batch_post(rel_url: &str, _body: &str, state: &web::Data<AppState>) -> (u16, Value) {
    let entities = &state.entities;
    let full_path = if rel_url.starts_with('/') {
        rel_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, rel_url)
    };
    let path = full_path.split('?').next().unwrap_or(&full_path).trim_end_matches('/');

    for entity in entities.iter() {
        let set_prefix = format!("{}/{}(", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            // Finde ')/' um Entity-Key von Action zu trennen
            if let Some(paren_end) = rest.find(")/") {
                let key_str = &rest[..paren_end];
                let action_part = &rest[paren_end + 2..];
                let key_info = match parse_entity_key(&format!("({})", key_str), entity.key_field()) {
                    Some(k) => k,
                    None => return (400, json!({"error": {"code": "400", "message": "Invalid key"}})),
                };
                let action_name = action_part.rsplit('.').next().unwrap_or(action_part);

                let mut store = state.data_store.write().unwrap();
                let records = store.get_mut(entity.set_name());

                match action_name {
                    "draftEdit" => {
                        if let Some(records) = records {
                            // Finde aktives Entity
                            if let Some(active) = records
                                .iter()
                                .find(|r| {
                                    r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(true)
                                })
                                .cloned()
                            {
                                // Draft-Kopie erstellen
                                let mut draft = active.clone();
                                if let Some(obj) = draft.as_object_mut() {
                                    obj.insert("IsActiveEntity".to_string(), json!(false));
                                    obj.insert("HasActiveEntity".to_string(), json!(true));
                                    obj.insert("HasDraftEntity".to_string(), json!(false));
                                }
                                // Aktives Entity markieren
                                if let Some(active_rec) = records.iter_mut().find(|r| {
                                    r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(true)
                                }) {
                                    if let Some(obj) = active_rec.as_object_mut() {
                                        obj.insert("HasDraftEntity".to_string(), json!(true));
                                    }
                                }
                                let result = draft.clone();
                                records.push(draft);
                                return (201, result);
                            }
                        }
                    }
                    "draftActivate" => {
                        if let Some(records) = records {
                            // Finde Draft
                            if let Some(draft) = records
                                .iter()
                                .find(|r| {
                                    r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            == Some(false)
                                })
                                .cloned()
                            {
                                // Aenderungen in aktives Entity uebernehmen
                                if let Some(active_rec) = records.iter_mut().find(|r| {
                                    r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(true)
                                }) {
                                    if let (Some(active_obj), Some(draft_obj)) =
                                        (active_rec.as_object_mut(), draft.as_object())
                                    {
                                        for (k, v) in draft_obj {
                                            if k == "IsActiveEntity"
                                                || k == "HasActiveEntity"
                                                || k == "HasDraftEntity"
                                            {
                                                continue;
                                            }
                                            active_obj.insert(k.clone(), v.clone());
                                        }
                                        active_obj
                                            .insert("HasDraftEntity".to_string(), json!(false));
                                    }
                                }
                                // Draft entfernen
                                records.retain(|r| {
                                    !(r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                            == Some(false))
                                });
                                // Aktiviertes Entity zurueckgeben
                                if let Some(activated) = records.iter().find(|r| {
                                    r.get(entity.key_field()).and_then(|v| v.as_str())
                                        == Some(&key_info.key_value)
                                        && r.get("IsActiveEntity")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(true)
                                }) {
                                    return (200, activated.clone());
                                }
                            }
                        }
                    }
                    "draftPrepare" => {
                        if let Some(records) = records {
                            if let Some(rec) = records.iter().find(|r| {
                                r.get(entity.key_field()).and_then(|v| v.as_str())
                                    == Some(&key_info.key_value)
                                    && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                                        == Some(key_info.is_active)
                            }) {
                                return (200, rec.clone());
                            }
                        }
                    }
                    _ => {}
                }
                return (
                    200,
                    json!({"error": {"code": "404", "message": format!("Action not found: {}", action_name)}}),
                );
            }
        }
    }
    (404, json!({"error": {"code": "404", "message": format!("Unknown POST: {}", rel_url)}}))
}

/// Generischer Batch-GET – loest Pfade ueber die Entity-Registry auf.
fn handle_batch_get(rel_url: &str, state: &web::Data<AppState>) -> Value {
    let entities = &state.entities;
    let store = state.data_store.read().unwrap();
    let full_path = if rel_url.starts_with('/') {
        rel_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, rel_url)
    };

    let (path_part, query_part) = full_path.split_once('?').unwrap_or((&full_path, ""));
    let path = path_part.trim_end_matches('/');
    let qs = parse_query_string(query_part);

    // Service root
    if path == BASE_PATH {
        let sets: Vec<Value> = entities
            .iter()
            .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
            .collect();
        return json!({
            "@odata.context": format!("{}/$metadata", BASE_PATH),
            "value": sets
        });
    }

    // Iterate entities for collection, $count, and single-entity routes
    for entity in entities.iter() {
        let set_path = format!("{}/{}", BASE_PATH, entity.set_name());
        let count_path = format!("{}/$count", set_path);
        let records = store.get(entity.set_name());

        // Collection
        if path == set_path {
            if let Some(data) = records {
                return query_collection_from(*entity, data, &qs, entities);
            }
            return query_collection(*entity, &qs, entities);
        }

        // $count
        if path == count_path {
            let count = records.map(|r| r.len()).unwrap_or(0);
            return json!({"value": count});
        }

        // Single entity: /SetName('key') or /SetName(KeyField='key',IsActiveEntity=true)
        let set_prefix = format!("{}/{}(", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            // Strip trailing ')'
            if let Some(key_str) = rest.strip_suffix(')') {
                if let Some(key_info) = parse_entity_key(&format!("({})", key_str), entity.key_field()) {
                let data = records.map(|r| r.as_slice());
                if let Some(record) = data.and_then(|d| {
                    d.iter().find(|r| {
                        let key_match = r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(&key_info.key_value);
                        let active_match = r.get("IsActiveEntity").and_then(|v| v.as_bool()).unwrap_or(true) == key_info.is_active;
                        key_match && active_match
                    })
                }) {
                    let mut result = record.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(
                            "@odata.context".to_string(),
                            json!(format!(
                                "{}/$metadata#{}/$entity",
                                BASE_PATH,
                                entity.set_name()
                            )),
                        );
                    }
                    if let Some(expand) = qs.get("$expand") {
                        if !expand.is_empty() {
                            let nav_names: Vec<&str> =
                                expand.split(',').map(|s| s.trim()).collect();
                            entity.expand_record(&mut result, &nav_names, entities);
                        }
                    }
                    return result;
                }
                return json!({"error": {"code": "404", "message": "Not found"}});
                }
            }
        }
    }

    json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}})
}

// ── Static file serving ─────────────────────────────────────────────
pub async fn static_files(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let raw_path = urlencoding::decode(req.path())
        .unwrap_or_default()
        .into_owned();
    let mut relative = raw_path.trim_start_matches('/').to_string();

    for prefix in &["products/demo/", "products.demo/"] {
        if relative.starts_with(prefix) {
            relative = relative[prefix.len()..].to_string();
            break;
        }
    }
    if relative.is_empty() || relative == "flp.html" {
        relative = "flp.html".to_string();
    }

    // manifest.json wird dynamisch aus der Entity-Registry generiert
    if relative == "manifest.json" {
        let mut builder = HttpResponse::Ok();
        builder.insert_header(("Content-Type", "application/json;charset=utf-8"));
        for (k, v) in cors_headers() {
            builder.insert_header((k, v));
        }
        return builder.body(data.manifest_json.clone());
    }

    // flp.html wird dynamisch generiert (Settings-gesteuert)
    if relative == "flp.html" {
        let mut builder = HttpResponse::Ok();
        builder.insert_header(("Content-Type", "text/html;charset=utf-8"));
        for (k, v) in cors_headers() {
            builder.insert_header((k, v));
        }
        return builder.body(data.flp_html.clone());
    }

    let wa_dir = crate::webapp_dir();
    if !wa_dir.exists() {
        return error_response(404, "webapp directory not found");
    }

    let candidate = wa_dir.join(&relative);
    // Path traversal protection
    let canonical = match candidate.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // SPA fallback: extensionless routes get the dynamic flp.html
            // but NOT for /sap/ paths (e.g. /sap/bc/lrep/flex/...)
            if Path::new(&relative).extension().is_none() && !raw_path.starts_with("/sap/") {
                let mut builder = HttpResponse::Ok();
                builder.insert_header(("Content-Type", "text/html;charset=utf-8"));
                return builder.body(data.flp_html.clone());
            }
            return error_response(404, &format!("Resource not found: {}", raw_path));
        }
    };
    let wa_canonical = match wa_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return error_response(403, "Access denied."),
    };
    if !canonical.starts_with(&wa_canonical) {
        return error_response(403, "Access denied.");
    }

    let target = if canonical.is_dir() {
        canonical.join("index.html")
    } else {
        canonical
    };

    if target.exists() && target.is_file() {
        return serve_file(&target);
    }

    // SPA fallback for extensionless routes (skip /sap/ API paths)
    if Path::new(&relative).extension().is_none() && !raw_path.starts_with("/sap/") {
        let mut builder = HttpResponse::Ok();
        builder.insert_header(("Content-Type", "text/html;charset=utf-8"));
        return builder.body(data.flp_html.clone());
    }

    error_response(404, &format!("Resource not found: {}", raw_path))
}

fn serve_file(path: &Path) -> HttpResponse {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    match std::fs::read(path) {
        Ok(bytes) => HttpResponse::Ok()
            .insert_header(("Content-Type", mime.to_string()))
            .body(bytes),
        Err(_) => error_response(500, "Failed to read file"),
    }
}

// ── Favicon: Dänischer Leuchtturm (SVG) ────────────────────────────
fn favicon_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#1a3a5c"/>
      <stop offset="100%" stop-color="#4a90c4"/>
    </linearGradient>
    <clipPath id="tower">
      <polygon points="26,56 22,22 42,22 38,56"/>
    </clipPath>
  </defs>
  <!-- Himmel -->
  <rect width="64" height="64" rx="12" fill="url(#sky)"/>
  <!-- Duene / Sand -->
  <ellipse cx="32" cy="60" rx="38" ry="10" fill="#d4a84b"/>
  <!-- Turm: abwechselnd rot/weiss, geclippt auf Turmform -->
  <g clip-path="url(#tower)">
    <rect x="20" y="22" width="24" height="34" fill="#ffffff"/>
    <rect x="20" y="22" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="32" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="42" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="52" width="24" height="4"  fill="#c0392b"/>
  </g>
  <!-- Galerie (Balkon) -->
  <rect x="19" y="19" width="26" height="4" rx="1" fill="#2c3e50"/>
  <!-- Laterne (Glashaus) -->
  <rect x="25" y="11" width="14" height="9" rx="2" fill="#f9e784" opacity="0.9"/>
  <rect x="25" y="11" width="14" height="9" rx="2" fill="none" stroke="#2c3e50" stroke-width="1"/>
  <!-- Dach -->
  <polygon points="24,11 32,5 40,11" fill="#2c3e50"/>
  <!-- Lichtstrahl -->
  <polygon points="39,15 58,6 58,12 39,17" fill="#f9e784" opacity="0.35"/>
  <polygon points="25,15 6,6 6,12 25,17" fill="#f9e784" opacity="0.25"/>
  <!-- Tuer -->
  <rect x="29" y="49" width="6" height="7" rx="3" fill="#2c3e50"/>
</svg>"##
}

fn favicon_response() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Content-Type", "image/svg+xml"))
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .body(favicon_svg())
}

pub async fn catch_all(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let path = req.path();

    if req.method() == actix_web::http::Method::OPTIONS {
        return options_handler().await;
    }

    if path == "/favicon.ico" || path == "/favicon.svg" {
        return favicon_response();
    }

    // Entity-bezogene Pfade: /BASE_PATH/SetName(...)
    for entity in data.entities.iter() {
        let set_prefix = format!("{}/{}(", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            // Finde das schliessende ')' - alles danach ist ggf. eine Action
            if let Some(paren_end) = rest.find(')') {
                let after_paren = &rest[paren_end + 1..];

                // POST Action: .../SetName(key)/Namespace.actionName
                if req.method() == actix_web::http::Method::POST && after_paren.starts_with('/') {
                    return draft_action_handler(req, body, data).await;
                }

                // PATCH: .../SetName(key)
                if req.method() == actix_web::http::Method::PATCH {
                    let json_body: Value = match serde_json::from_slice(&body) {
                        Ok(v) => v,
                        Err(_) => return error_response(400, "Invalid JSON body"),
                    };
                    return patch_entity_handler(req, web::Json(json_body), data).await;
                }

                // GET: .../SetName(key)
                return single_entity_handler(req, data).await;
            }
        }
    }

    static_files(req, data).await
}
