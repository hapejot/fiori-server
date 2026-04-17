use axum::body::Bytes;
use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::data_store::{EntityKey, ODataQuery, ParentKey, StoreError};
use crate::entity::ODataEntity;
use crate::routing::{resolve_odata_path, ODataPath};
use crate::BASE_PATH;

fn http_reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn cors_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Accept, Authorization, OData-Version, OData-MaxVersion, X-Requested-With"),
        ("Access-Control-Expose-Headers", "OData-Version"),
    ]
}

pub fn json_response(data: Value) -> Response {
    let body = serde_json::to_string_pretty(&data).unwrap_or_default();
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Type",
            "application/json;odata.metadata=minimal;charset=utf-8",
        )
        .header("OData-Version", "4.0");
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Body::from(body)).unwrap()
}

fn json_response_with_status(status: StatusCode, data: Value) -> Response {
    let body = serde_json::to_string_pretty(&data).unwrap_or_default();
    let mut builder = Response::builder()
        .status(status)
        .header(
            "Content-Type",
            "application/json;odata.metadata=minimal;charset=utf-8",
        )
        .header("OData-Version", "4.0");
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Body::from(body)).unwrap()
}

fn store_error_to_response(err: StoreError) -> Response {
    match err {
        StoreError::NotFound(msg) => error_response(404, &msg),
        StoreError::BadRequest(msg) => error_response(400, &msg),
    }
}

fn store_result_to_response(result: Result<Value, StoreError>) -> Response {
    match result {
        Ok(val) => json_response(val),
        Err(e) => store_error_to_response(e),
    }
}

fn store_result_to_response_with_status(
    result: Result<Value, StoreError>,
    status: StatusCode,
) -> Response {
    match result {
        Ok(val) => json_response_with_status(status, val),
        Err(e) => store_error_to_response(e),
    }
}

fn store_delete_to_response(result: Result<(), StoreError>) -> Response {
    match result {
        Ok(()) => {
            let mut builder = Response::builder().status(StatusCode::NO_CONTENT);
            for (k, v) in cors_headers() {
                builder = builder.header(k, v);
            }
            builder.body(Body::empty()).unwrap()
        }
        Err(e) => store_error_to_response(e),
    }
}

/// Build an EntityKey from the parsed routing info.
fn entity_key_from_routing(key: &crate::routing::EntityKeyInfo) -> EntityKey {
    EntityKey::composite(&[
        ("ID", &key.key_value),
        (
            "IsActiveEntity",
            if key.is_active { "true" } else { "false" },
        ),
    ])
}

pub fn error_response(code: u16, message: &str) -> Response {
    let body = json!({"error": {"code": code.to_string(), "message": message}});
    let status = match code {
        404 => StatusCode::NOT_FOUND,
        405 => StatusCode::METHOD_NOT_ALLOWED,
        400 => StatusCode::BAD_REQUEST,
        403 => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let json_body = serde_json::to_string(&body).unwrap_or_default();
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", "application/json;charset=utf-8")
        .header("OData-Version", "4.0");
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Body::from(json_body)).unwrap()
}

fn options_response() -> Response {
    let mut builder = Response::builder().status(StatusCode::OK);
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Body::empty()).unwrap()
}

pub async fn metadata_handler(State(state): State<Arc<AppState>>) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml;charset=utf-8")
        .header("OData-Version", "4.0");
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder
        .body(Body::from(state.metadata_xml.read().unwrap().clone()))
        .unwrap()
}

/// Service-Dokument – wird dynamisch aus der Entity-Registry erzeugt.
pub async fn service_document(State(state): State<Arc<AppState>>) -> Response {
    let entities = state.entities.read().unwrap();
    let sets: Vec<Value> = entities
        .iter()
        .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
        .collect();
    json_response(json!({
        "@odata.context": format!("{}/$metadata", BASE_PATH),
        "value": sets
    }))
}

/// Generischer Collection-Handler fuer beliebige EntitySets.
pub async fn collection_handler(State(state): State<Arc<AppState>>, uri: Uri) -> Response {
    let path = uri.path();
    let query_str = uri.query().unwrap_or("");
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);

    info!("retrieve collection");
    if let ODataPath::Collection { entity: e } = parsed.path {
        info!("entity {}", e.set_name());
        let set_name = match &parsed.path {
            ODataPath::Collection { entity } => entity.set_name(),
            _ => return error_response(404, "Entity set not found"),
        };
        let query = ODataQuery::parse(query_str);
        return store_result_to_response(state.data_store.get_collection(set_name, &query, None));
    }
    error_response(404, "Entity set not found")
}

/// Generischer $count-Handler fuer beliebige EntitySets.
pub async fn count_handler(State(state): State<Arc<AppState>>, uri: Uri) -> Response {
    let path = uri.path();
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);
    if let ODataPath::Count { entity } = parsed.path {
        let query = ODataQuery::empty();
        let count = state.data_store.count(entity.set_name(), &query, None);
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain;charset=utf-8")
            .header("OData-Version", "4.0");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(count.to_string())).unwrap();
    }
    error_response(404, "Entity set not found")
}

/// Generischer Single-Entity-Handler: /SetName('key') or /SetName(Key='val',IsActiveEntity=true)
fn handle_single_entity(path: &str, query_str: &str, state: &AppState) -> Response {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let entity_key = entity_key_from_routing(&key);
        let query = ODataQuery::parse(query_str);
        return store_result_to_response(state.data_store.read_entity(
            entity.set_name(),
            &entity_key,
            &query,
        ));
    }
    error_response(404, "Entity not found.")
}

/// Generischer PATCH-Handler: /SetName(key) – aktualisiert Felder in-memory.
fn handle_patch_entity(path: &str, body: &Value, state: &AppState) -> Response {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let entity_key = entity_key_from_routing(&key);
        return store_result_to_response(state.data_store.patch_entity(
            entity.set_name(),
            &entity_key,
            body,
        ));
    }
    error_response(404, "Entity not found.")
}

/// Handler fuer DELETE: Draft verwerfen (Discard).
/// DELETE /SetName(key) – entfernt Draft und setzt HasDraftEntity=false am aktiven Entity.
fn handle_delete_entity(path: &str, state: &AppState) -> Response {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let entity_key = entity_key_from_routing(&key);
        return store_delete_to_response(
            state
                .data_store
                .delete_entity(entity.set_name(), &entity_key),
        );
    }
    error_response(404, "Entity not found.")
}

/// Handler fuer Draft-Actions: draftEdit, draftActivate, draftPrepare.
/// POST /SetName(key)/Namespace.actionName
fn handle_draft_action(path: &str, state: &AppState) -> Response {
    // Extract action info under read lock, then release it so activate_config can write-lock
    let action_info = {
        let entities = state.entities.read().unwrap();
        let parsed = resolve_odata_path(path, &entities);
        match parsed.path {
            ODataPath::Action {
                entity,
                key,
                action,
            } => {
                let entity_key = entity_key_from_routing(&key);
                Some((
                    entity.set_name().to_string(),
                    entity_key,
                    key.key_value,
                    action,
                ))
            }
            _ => None,
        }
    };

    let Some((set_name, entity_key, key_value, action)) = action_info else {
        return error_response(404, "Entity not found for action.");
    };

    match action.as_str() {
        "draftEdit" => {
            store_result_to_response(state.data_store.draft_edit(&set_name, &entity_key))
        }
        "draftActivate" => {
            let result = state.data_store.draft_activate(&set_name, &entity_key);
            if result.is_ok() {
                info!("  [action] draftActivate succeeded, calling commit()");
                state.data_store.commit();
            }
            store_result_to_response(result)
        }
        "draftPrepare" => {
            store_result_to_response(state.data_store.draft_prepare(&set_name, &entity_key))
        }
        "publishConfig" => {
            match crate::entities::meta::publish_entity_config(
                &key_value,
                state.data_store.as_ref(),
            ) {
                Ok(val) => {
                    state.activate_config();
                    let body = serde_json::json!({
                        "@odata.context": format!("{}/$metadata#EntityConfigs/$entity", crate::BASE_PATH),
                        "value": val
                    });
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
                        .unwrap()
                }
                Err(msg) => error_response(400, &msg),
            }
        }
        _ => error_response(400, &format!("Unknown action: {}", action)),
    }
}

// ── $batch handler ──────────────────────────────────────────────────
#[tracing::instrument(skip(state, headers, body))]
pub async fn batch_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let content_type = headers
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
                        l.starts_with("GET ")
                            || l.starts_with("POST ")
                            || l.starts_with("PATCH ")
                            || l.starts_with("DELETE ")
                    });
                    if let Some(cs_req_line) = cs_request_line {
                        let cs_parts: Vec<&str> = cs_req_line.split_whitespace().collect();
                        let cs_method = cs_parts.first().copied().unwrap_or("");
                        let cs_rel_url = cs_parts.get(1).copied().unwrap_or("");
                        let cs_body = extract_batch_body(cs_segment);

                        let (cs_status, cs_resp_json) = match cs_method {
                            "GET" => (200, handle_batch_get(cs_rel_url, &state)),
                            "PATCH" => handle_batch_patch(cs_rel_url, &cs_body, &state),
                            "POST" => handle_batch_post(cs_rel_url, &cs_body, &state),
                            "DELETE" => handle_batch_delete(cs_rel_url, &state),
                            _ => (200, json!({})),
                        };
                        let cs_resp_body = serde_json::to_string(&cs_resp_json).unwrap_or_default();
                        cs_response_parts.push(format!(
                            "Content-Type: application/http\r\n\
                             Content-Transfer-Encoding: binary\r\n\
                             \r\n\
                             HTTP/1.1 {} {}\r\n\
                             Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                             OData-Version: 4.0\r\n\
                             Content-Length: {}\r\n\
                             \r\n\
                             {}",
                            cs_status,
                            http_reason_phrase(cs_status),
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
            l.starts_with("GET ")
                || l.starts_with("POST ")
                || l.starts_with("PATCH ")
                || l.starts_with("DELETE ")
        });
        if let Some(request_line) = request_line {
            // info!("+-- {}", request_line);
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let method = parts.first().copied().unwrap_or("");
            let rel_url = parts.get(1).copied().unwrap_or("");

            // Body aus dem Segment extrahieren (fuer POST/PATCH)
            let segment_body = extract_batch_body(segment);

            info!("method: {method}");
            let (status, resp_json) = match method {
                "GET" => (200, handle_batch_get(rel_url, &state)),
                "PATCH" => handle_batch_patch(rel_url, &segment_body, &state),
                "POST" => handle_batch_post(rel_url, &segment_body, &state),
                "DELETE" => handle_batch_delete(rel_url, &state),
                _ => (200, handle_batch_get(rel_url, &state)),
            };

            let resp_body = serde_json::to_string(&resp_json).unwrap_or_default();

            let part_resp = format!(
                "Content-Type: application/http\r\n\
                 Content-Transfer-Encoding: binary\r\n\
                 \r\n\
                 HTTP/1.1 {} {}\r\n\
                 Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                 OData-Version: 4.0\r\n\
                 Content-Length: {}\r\n\
                 \r\n\
                 {}",
                status,
                http_reason_phrase(status),
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

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Type",
            format!("multipart/mixed; boundary={}", resp_boundary),
        )
        .header("OData-Version", "4.0");
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Body::from(full_body)).unwrap()
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
#[tracing::instrument(skip(state, body))]
fn handle_batch_patch(rel_url: &str, body: &str, state: &AppState) -> (u16, Value) {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(rel_url, &entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let patch_data: Value = serde_json::from_str(body).unwrap_or(json!({}));
        let entity_key = entity_key_from_routing(&key);
        info!("{}", entity.set_name());
        match state
            .data_store
            .patch_entity(entity.set_name(), &entity_key, &patch_data)
        {
            Ok(val) => return (200, val),
            Err(e) => {
                return (
                    404,
                    json!({"error": {"code": "404", "message": format!("{}", e)}}),
                )
            }
        }
    }
    (
        404,
        json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    )
}

/// Batch-DELETE: Draft verwerfen innerhalb von $batch.
#[tracing::instrument(skip(state))]
fn handle_batch_delete(rel_url: &str, state: &AppState) -> (u16, Value) {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(rel_url, &entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let entity_key = entity_key_from_routing(&key);
        match state
            .data_store
            .delete_entity(entity.set_name(), &entity_key)
        {
            Ok(()) => return (204, json!({})),
            Err(e) => {
                return (
                    404,
                    json!({"error": {"code": "404", "message": format!("{}", e)}}),
                )
            }
        }
    }
    (
        404,
        json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    )
}

/// Batch-POST: behandelt Aktionen (draftEdit, draftActivate, draftPrepare) innerhalb von $batch.
#[tracing::instrument(skip(state, rel_url, body))]
fn handle_batch_post(rel_url: &str, body: &str, state: &AppState) -> (u16, Value) {
    // Extract routing info under read lock, then drop it so activate_config can write-lock
    enum PostTarget {
        Action {
            set_name: String,
            entity_key: EntityKey,
            key_value: String,
            action: String,
        },
        SubCollection {
            parent_set_name: String,
            parent_key: EntityKey,
            child_set_name: String,
        },
        Collection {
            set_name: String,
        },
        NotFound,
    }

    let target = {
        let entities = state.entities.read().unwrap();
        let parsed = resolve_odata_path(rel_url, &entities);
        match parsed.path {
            ODataPath::Action {
                entity,
                key,
                action,
            } => {
                let entity_key = entity_key_from_routing(&key);
                PostTarget::Action {
                    set_name: entity.set_name().to_string(),
                    entity_key,
                    key_value: key.key_value,
                    action,
                }
            }
            ODataPath::SubCollection {
                parent_entity,
                parent_key,
                child_entity,
                ..
            } => PostTarget::SubCollection {
                parent_set_name: parent_entity.set_name().to_string(),
                parent_key: entity_key_from_routing(&parent_key),
                child_set_name: child_entity.set_name().to_string(),
            },
            ODataPath::Collection { entity } => PostTarget::Collection {
                set_name: entity.set_name().to_string(),
            },
            _ => PostTarget::NotFound,
        }
    };

    match target {
        PostTarget::Action {
            set_name,
            entity_key,
            key_value,
            action,
        } => {
            info!("target action: {}", action);
            let result = match action.as_str() {
                "draftEdit" => state.data_store.draft_edit(&set_name, &entity_key),
                "draftActivate" => {
                    let res = state.data_store.draft_activate(&set_name, &entity_key);
                    if res.is_ok() {
                        info!("  [batch] draftActivate succeeded, calling commit()");
                        state.data_store.commit();
                    }
                    res
                }
                "draftPrepare" => state.data_store.draft_prepare(&set_name, &entity_key),
                "publishConfig" => {
                    match crate::entities::meta::publish_entity_config(
                        &key_value,
                        state.data_store.as_ref(),
                    ) {
                        Ok(val) => {
                            state.activate_config();
                            return (200, val);
                        }
                        Err(msg) => {
                            return (400, json!({"error": {"code": "400", "message": msg}}))
                        }
                    }
                }
                _ => {
                    return (
                        400,
                        json!({"error": {"code": "400", "message": format!("Unknown action: {}", action)}}),
                    )
                }
            };
            // info!("result: {:?}", result);
            match result {
                Ok(val) => (200, val),
                Err(e) => (
                    404,
                    json!({"error": {"code": "404", "message": format!("{}", e)}}),
                ),
            }
        }
        PostTarget::SubCollection {
            parent_set_name,
            parent_key,
            child_set_name,
        } => {
            let parent = ParentKey::new(&parent_set_name, parent_key);
            let data: Value = serde_json::from_str(body).unwrap_or(json!({}));
            match state
                .data_store
                .create_entity(&child_set_name, &data, Some(&parent))
            {
                Ok(val) => (201, val),
                Err(e) => (
                    400,
                    json!({"error": {"code": "400", "message": format!("{}", e)}}),
                ),
            }
        }
        PostTarget::Collection { set_name } => {
            let data: Value = serde_json::from_str(body).unwrap_or(json!({}));
            match state.data_store.create_entity(&set_name, &data, None) {
                Ok(val) => (201, val),
                Err(e) => (
                    400,
                    json!({"error": {"code": "400", "message": format!("{}", e)}}),
                ),
            }
        }
        PostTarget::NotFound => (
            404,
            json!({"error": {"code": "404", "message": format!("Unknown POST: {}", rel_url)}}),
        ),
    }
}

/// Generischer Batch-GET – loest Pfade ueber die Entity-Registry auf.
#[tracing::instrument(skip(state, rel_url))]
fn handle_batch_get(rel_url: &str, state: &AppState) -> Value {
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(rel_url, &entities);
    let query = ODataQuery::parse(&parsed.query_string);

    match parsed.path {
        ODataPath::ServiceRoot => {
            let sets: Vec<Value> = entities
                .iter()
                .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
                .collect();
            json!({
                "@odata.context": format!("{}/$metadata", BASE_PATH),
                "value": sets
            })
        }
        ODataPath::Collection { entity } => {
            match state
                .data_store
                .get_collection(entity.set_name(), &query, None)
            {
                Ok(val) => val,
                Err(e) => json!({"error": {"code": "404", "message": format!("{}", e)}}),
            }
        }
        ODataPath::Count { entity } => {
            let count = state.data_store.count(entity.set_name(), &query, None);
            json!({"value": count})
        }
        ODataPath::Entity { entity, key } => {
            let entity_key = entity_key_from_routing(&key);
            match state
                .data_store
                .read_entity(entity.set_name(), &entity_key, &query)
            {
                Ok(val) => val,
                Err(e) => json!({"error": {"code": "404", "message": format!("{}", e)}}),
            }
        }
        ODataPath::SubCollection {
            parent_entity,
            parent_key,
            child_entity,
            ..
        } => {
            let parent = ParentKey::new(
                parent_entity.set_name(),
                entity_key_from_routing(&parent_key),
            );
            match state
                .data_store
                .get_collection(child_entity.set_name(), &query, Some(&parent))
            {
                Ok(val) => val,
                Err(e) => json!({"error": {"code": "404", "message": format!("{}", e)}}),
            }
        }
        ODataPath::PropertyAccess {
            entity,
            key,
            property,
        } => {
            let entity_key = entity_key_from_routing(&key);
            if property == "SiblingEntity" {
                match state
                    .data_store
                    .read_sibling_entity(entity.set_name(), &entity_key)
                {
                    Ok(val) => val,
                    Err(e) => json!({"error": {"code": "404", "message": format!("{}", e)}}),
                }
            } else {
                match state
                    .data_store
                    .get_property(entity.set_name(), &entity_key, &property)
                {
                    Ok(val) => json!({ "value": val }),
                    Err(e) => json!({"error": {"code": "404", "message": format!("{}", e)}}),
                }
            }
        }
        _ => json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    }
}

// ── Sub-Collection handler ──────────────────────────────────────────
/// Liefert die Kind-Eintraege einer Komposition.
#[tracing::instrument(skip(state))]
fn handle_sub_collection(
    parent_entity: &dyn ODataEntity,
    parent_key: &crate::routing::EntityKeyInfo,
    child_entity: &dyn ODataEntity,
    query_str: &str,
    state: &AppState,
) -> Response {
    let parent = ParentKey::new(
        parent_entity.set_name(),
        entity_key_from_routing(parent_key),
    );
    let query = ODataQuery::parse(query_str);
    store_result_to_response(state.data_store.get_collection(
        child_entity.set_name(),
        &query,
        Some(&parent),
    ))
}

// ── Eincompilierte statische Dateien ────────────────────────────────
#[tracing::instrument]
fn serve_embedded_file(relative: &str) -> Option<Response> {
    let (content, content_type) = match relative {
        "flp-init.js" => (
            crate::EMBEDDED_FLP_INIT_JS,
            "application/javascript;charset=utf-8",
        ),
        "i18n/i18n.properties" => (crate::EMBEDDED_I18N_PROPERTIES, "text/plain;charset=utf-8"),
        "appconfig/fioriSandboxConfig.json" => (
            crate::EMBEDDED_SANDBOX_CONFIG,
            "application/json;charset=utf-8",
        ),
        _ => return None,
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type);
    for (k, v) in cors_headers() {
        builder = builder.header(k, v);
    }
    Some(builder.body(Body::from(content)).unwrap())
}

// ── Static file serving ─────────────────────────────────────────────
#[tracing::instrument(skip(state))]
fn handle_file(path: &str, state: &AppState) -> Response {
    let raw_path = urlencoding::decode(path).unwrap_or_default().into_owned();

    let mut relative = raw_path
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>();

    // Entity-spezifischer App-Pfad: /apps/{EntitySet}/...
    // Jede Entitaet bekommt ein eigenes Manifest mit passender Default-Route.
    let mut entity_hint: Option<String> = None;
    match relative[0] {
        "apps" => {
            let candidate = relative[1];
            if state
                .entity_manifests
                .read()
                .unwrap()
                .contains_key(candidate)
            {
                entity_hint = Some(String::from(candidate));
                relative = relative[2..].to_vec();
            }
        }
        _ => {}
    }

    // for prefix in &["products/demo/", "products.demo/"] {
    //     if relative.starts_with(prefix) {
    //         relative = relative[prefix.len()..].to_string();
    //         break;
    //     }
    // }

    // if relative.is_empty() || relative == "flp.html" {
    //     relative = "flp.html".to_string();
    // }

    // manifest.json wird dynamisch aus der Entity-Registry generiert
    if relative[0] == "manifest.json" {
        info!(
            "Serving manifest.json for entity: {}",
            entity_hint.as_deref().unwrap_or("default")
        );
        let manifest_body = if let Some(ref name) = entity_hint {
            let entity_manifests = state.entity_manifests.read().unwrap();
            entity_manifests.get(name).unwrap().clone()
        } else {
            let manifest_json = state.manifest_json.read().unwrap();
            manifest_json.clone()
        };
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(manifest_body.clone())).unwrap();
    }

    // apps.json dynamisch ausliefern (statische + generische Entitaeten)
    if relative[0] == "config" && relative[1] == "apps.json" {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder
            .body(Body::from(state.apps_json.read().unwrap().clone()))
            .unwrap();
    }

    // LREP Flexibility stubs — return empty responses so the UI5 flex
    // framework does not log 404 errors for every app startup.
    if raw_path.starts_with("/sap/bc/lrep/flex/") {
        let body = if raw_path.contains("/flex/settings") {
            r#"{"isKeyUser":false,"isVariantSharingEnabled":false,"isAtoAvailable":false,"isAtoEnabled":false,"isProductiveSystem":true}"#
        } else {
            // flex/data responses — empty changes
            r#"{"changes":[],"compVariants":[],"variantChanges":[],"variantDependentControlChanges":[],"variantManagementChanges":[],"variants":[]}"#
        };
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(body)).unwrap();
    }

    // CDM 3.1 Site-Dokument fuer den UShell CDM-Modus ausliefern
    if relative[0] == "cdm" && relative[1] == "site.json" {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder
            .body(Body::from(state.cdm_site_json.read().unwrap().clone()))
            .unwrap();
    }

    // Component.js wird dynamisch generiert — der Klassenname muss zum
    // sap.app.id im jeweiligen Manifest passen, sonst cached UI5 den
    // falschen Component fuer die zweite App.
    if relative[0] == "Component.js" {
        let app_id = entity_hint
            .as_ref()
            .map(|name| format!("{}.app", name.to_lowercase()))
            .unwrap_or_else(|| "products.demo".to_string());
        let body = format!(
            "sap.ui.define([\"sap/fe/core/AppComponent\"], function (AppComponent) {{\n\
             \t\"use strict\";\n\
             \treturn AppComponent.extend(\"{}.Component\", {{\n\
             \t\tmetadata: {{\n\
             \t\t\tmanifest: \"json\"\n\
             \t\t}}\n\
             \t}});\n\
             }});",
            app_id
        );
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/javascript;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(body)).unwrap();
    }

    // flp.html wird dynamisch generiert (Settings-gesteuert)
    if relative[0] == "flp.html" {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(state.flp_html.clone())).unwrap();
    }

    // ── Eincompilierte Dateien (kein Dateisystem noetig) ────────────
    if let Some(resp) = serve_embedded_file(&String::from(
        relative.iter().map(|s| *s).collect::<Vec<_>>().join("/"),
    )) {
        return resp;
    }
    if relative.last() == Some(&"") {
        relative.pop();
        relative.push("index.html");
    }
    let path = Path::new("webapp").join(relative.iter().collect::<PathBuf>());
    if path.exists() && path.is_file() {
        let res = serve_file(&path);
        return res;
    } else {
        if raw_path == "/" {
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html;charset=utf-8");
            for (k, v) in cors_headers() {
                builder = builder.header(k, v);
            }
            return builder.body(Body::from(state.flp_html.clone())).unwrap();
        }
        error!("File not found: {:?}", path);
        error!("                {}", raw_path);
        error_response(404, &format!("Resource not found: {}", raw_path))
    }
}

fn serve_file(path: &Path) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    match std::fs::read(path) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime.to_string())
            .body(Body::from(bytes))
            .unwrap(),
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

fn favicon_response() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/svg+xml")
        .header("Cache-Control", "public, max-age=86400")
        .body(Body::from(favicon_svg()))
        .unwrap()
}

#[tracing::instrument(skip(state, body))]
pub async fn catch_all(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> Response {
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    if method == Method::OPTIONS {
        return options_response();
    }

    if path == "/favicon.ico" || path == "/favicon.svg" {
        return favicon_response();
    }

    // Entity-bezogene Pfade ueber den zentralen Router aufloesen
    let entities = state.entities.read().unwrap();
    let parsed = resolve_odata_path(path, &entities);
    match parsed.path {
        ODataPath::Entity { .. } => match method {
            Method::GET => handle_single_entity(path, query, &state),
            Method::PATCH => {
                let json_body: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => return error_response(400, "Invalid JSON body"),
                };
                let resp = handle_patch_entity(path, &json_body, &state);
                state.data_store.commit();
                resp
            }
            Method::DELETE => {
                let resp = handle_delete_entity(path, &state);
                state.data_store.commit();
                resp
            }
            _ => error_response(405, "Method not allowed"),
        },
        ODataPath::Action { .. } => {
            if method == Method::POST {
                let resp = handle_draft_action(path, &state);
                resp
            } else {
                error_response(405, "Method not allowed")
            }
        }
        ODataPath::SubCollection {
            parent_entity,
            parent_key,
            child_entity,
            ..
        } => match method {
            Method::GET => {
                handle_sub_collection(parent_entity, &parent_key, child_entity, query, &state)
            }
            Method::POST => {
                let body_str = String::from_utf8_lossy(&body);
                let data: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
                let parent = ParentKey::new(
                    parent_entity.set_name(),
                    entity_key_from_routing(&parent_key),
                );
                let resp = store_result_to_response_with_status(
                    state
                        .data_store
                        .create_entity(child_entity.set_name(), &data, Some(&parent)),
                    StatusCode::CREATED,
                );
                state.data_store.commit();
                resp
            }
            _ => error_response(405, "Method not allowed"),
        },
        ODataPath::PropertyAccess {
            entity,
            key,
            property,
        } => {
            let entity_key = entity_key_from_routing(&key);
            if property == "SiblingEntity" {
                store_result_to_response(
                    state
                        .data_store
                        .read_sibling_entity(entity.set_name(), &entity_key),
                )
            } else {
                match state
                    .data_store
                    .get_property(entity.set_name(), &entity_key, &property)
                {
                    Ok(val) => json_response(json!({ "value": val })),
                    Err(e) => store_error_to_response(e),
                }
            }
        }
        ODataPath::Collection { entity } => match method {
            Method::POST => {
                let body_str = String::from_utf8_lossy(&body);
                let data: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
                let resp = store_result_to_response_with_status(
                    state
                        .data_store
                        .create_entity(entity.set_name(), &data, None),
                    StatusCode::CREATED,
                );
                state.data_store.commit();
                resp
            }
            _ => handle_file(path, &state),
        },
        _ => handle_file(path, &state),
    }
}
