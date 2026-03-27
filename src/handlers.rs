use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use axum::body::Bytes;
use log::info;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::draft;
use crate::entity::ODataEntity;
use crate::query::{parse_query_string, query_collection, query_collection_from};
use crate::routing::{resolve_odata_path, ODataPath};
use crate::BASE_PATH;

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

fn draft_to_response((status, value): (u16, Value)) -> Response {
    match status {
        204 => {
            let mut builder = Response::builder().status(StatusCode::NO_CONTENT);
            for (k, v) in cors_headers() {
                builder = builder.header(k, v);
            }
            builder.body(Body::empty()).unwrap()
        }
        _ => {
            let http_status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
            json_response_with_status(http_status, value)
        }
    }
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
    builder.body(Body::from(state.metadata_xml.clone())).unwrap()
}

/// Service-Dokument – wird dynamisch aus der Entity-Registry erzeugt.
pub async fn service_document(State(state): State<Arc<AppState>>) -> Response {
    let sets: Vec<Value> = state
        .entities
        .iter()
        .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
        .collect();
    json_response(json!({
        "@odata.context": format!("{}/$metadata", BASE_PATH),
        "value": sets
    }))
}

/// Generischer Collection-Handler fuer beliebige EntitySets.
pub async fn collection_handler(
    State(state): State<Arc<AppState>>,
    uri: Uri,
) -> Response {
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Collection { entity } = parsed.path {
        let qs = parse_query_string(query);
        let store = state.data_store.read().unwrap();
        if let Some(records) = store.get(entity.set_name()) {
            return json_response(query_collection_from(entity, records, &qs, &state.entities));
        }
        return json_response(query_collection(entity, &qs, &state.entities));
    }
    error_response(404, "Entity set not found")
}

/// Generischer $count-Handler fuer beliebige EntitySets.
pub async fn count_handler(
    State(state): State<Arc<AppState>>,
    uri: Uri,
) -> Response {
    let path = uri.path();
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Count { entity } = parsed.path {
        let store = state.data_store.read().unwrap();
        let count = store.get(entity.set_name()).map(|v| v.len()).unwrap_or(0);
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
fn handle_single_entity(path: &str, query: &str, state: &AppState) -> Response {
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let store = state.data_store.read().unwrap();
        return draft_to_response(draft::read_entity(
            &store,
            entity,
            &key.key_value,
            key.is_active,
            query,
            &state.entities,
        ));
    }
    error_response(404, "Entity not found.")
}

/// Generischer PATCH-Handler: /SetName(key) – aktualisiert Felder in-memory.
fn handle_patch_entity(path: &str, body: &Value, state: &AppState) -> Response {
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let mut store = state.data_store.write().unwrap();
        return draft_to_response(draft::patch_entity(
            &mut store,
            entity,
            &key.key_value,
            key.is_active,
            body,
        ));
    }
    error_response(404, "Entity not found.")
}

/// Handler fuer DELETE: Draft verwerfen (Discard).
/// DELETE /SetName(key) – entfernt Draft und setzt HasDraftEntity=false am aktiven Entity.
fn handle_delete_entity(path: &str, state: &AppState) -> Response {
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let mut store = state.data_store.write().unwrap();
        return draft_to_response(draft::delete_entity(
            &mut store,
            entity,
            &key.key_value,
            key.is_active,
            &state.entities,
        ));
    }
    error_response(404, "Entity not found.")
}

/// Handler fuer Draft-Actions: draftEdit, draftActivate, draftPrepare.
/// POST /SetName(key)/Namespace.actionName
fn handle_draft_action(path: &str, state: &AppState) -> Response {
    let parsed = resolve_odata_path(path, &state.entities);
    if let ODataPath::Action {
        entity,
        key,
        action,
    } = parsed.path
    {
        return match action.as_str() {
            "draftEdit" => {
                let mut store = state.data_store.write().unwrap();
                draft_to_response(draft::draft_edit(
                    &mut store,
                    entity,
                    &key.key_value,
                    &state.entities,
                ))
            }
            "draftActivate" => {
                let mut store = state.data_store.write().unwrap();
                draft_to_response(draft::draft_activate(
                    &mut store,
                    entity,
                    &key.key_value,
                    &state.entities,
                ))
            }
            "draftPrepare" => {
                let store = state.data_store.read().unwrap();
                draft_to_response(draft::draft_prepare(
                    &store,
                    entity,
                    &key.key_value,
                    key.is_active,
                ))
            }
            _ => error_response(400, &format!("Unknown action: {}", action)),
        };
    }
    error_response(404, "Entity not found for action.")
}

// ── $batch handler ──────────────────────────────────────────────────
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
            l.starts_with("GET ")
                || l.starts_with("POST ")
                || l.starts_with("PATCH ")
                || l.starts_with("DELETE ")
        });
        if let Some(request_line) = request_line {
            info!("+-- {}", request_line);
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let method = parts.first().copied().unwrap_or("");
            let rel_url = parts.get(1).copied().unwrap_or("");

            // Body aus dem Segment extrahieren (fuer POST/PATCH)
            let segment_body = extract_batch_body(segment);

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

    // Nach Batch-Verarbeitung Daten persistieren
    state.save_data();

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
fn handle_batch_patch(rel_url: &str, body: &str, state: &AppState) -> (u16, Value) {
    let parsed = resolve_odata_path(rel_url, &state.entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let patch_data: Value = serde_json::from_str(body).unwrap_or(json!({}));
        let mut store = state.data_store.write().unwrap();
        return draft::patch_entity(&mut store, entity, &key.key_value, key.is_active, &patch_data);
    }
    (
        404,
        json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    )
}

/// Batch-DELETE: Draft verwerfen innerhalb von $batch.
fn handle_batch_delete(rel_url: &str, state: &AppState) -> (u16, Value) {
    let parsed = resolve_odata_path(rel_url, &state.entities);
    if let ODataPath::Entity { entity, key } = parsed.path {
        let mut store = state.data_store.write().unwrap();
        return draft::delete_entity(
            &mut store,
            entity,
            &key.key_value,
            key.is_active,
            &state.entities,
        );
    }
    (
        404,
        json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    )
}

/// Batch-POST: behandelt Aktionen (draftEdit, draftActivate, draftPrepare) innerhalb von $batch.
fn handle_batch_post(rel_url: &str, body: &str, state: &AppState) -> (u16, Value) {
    let parsed = resolve_odata_path(rel_url, &state.entities);
    if let ODataPath::Action {
        entity,
        key,
        action,
    } = parsed.path
    {
        let mut store = state.data_store.write().unwrap();
        return match action.as_str() {
            "draftEdit" => {
                draft::draft_edit(&mut store, entity, &key.key_value, &state.entities)
            }
            "draftActivate" => {
                draft::draft_activate(&mut store, entity, &key.key_value, &state.entities)
            }
            "draftPrepare" => draft::draft_prepare(&store, entity, &key.key_value, key.is_active),
            _ => (
                400,
                json!({"error": {"code": "400", "message": format!("Unknown action: {}", action)}}),
            ),
        };
    }

    // Sub-Collection POST: neues Kind-Element anlegen (z.B. Orders('O003')/Items)
    if let ODataPath::SubCollection {
        parent_entity,
        parent_key,
        child_entity,
        ..
    } = parsed.path
    {
        let mut store = state.data_store.write().unwrap();
        return draft::create_sub_item(&mut store, parent_entity, &parent_key, child_entity, body);
    }

    (
        404,
        json!({"error": {"code": "404", "message": format!("Unknown POST: {}", rel_url)}}),
    )
}

/// Generischer Batch-GET – loest Pfade ueber die Entity-Registry auf.
fn handle_batch_get(rel_url: &str, state: &AppState) -> Value {
    let entities = &state.entities;
    let store = state.data_store.read().unwrap();
    let parsed = resolve_odata_path(rel_url, entities);
    let qs = parse_query_string(&parsed.query_string);

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
            if let Some(data) = store.get(entity.set_name()) {
                query_collection_from(entity, data, &qs, entities)
            } else {
                query_collection(entity, &qs, entities)
            }
        }
        ODataPath::Count { entity } => {
            let count = store.get(entity.set_name()).map(|r| r.len()).unwrap_or(0);
            json!({"value": count})
        }
        ODataPath::Entity { entity, key } => {
            let (_, val) = draft::read_entity(
                &store,
                entity,
                &key.key_value,
                key.is_active,
                &parsed.query_string,
                entities,
            );
            val
        }
        ODataPath::SubCollection {
            parent_entity,
            parent_key,
            child_entity,
            ..
        } => draft::read_sub_collection(
            &store,
            parent_entity,
            &parent_key,
            child_entity,
            &parsed.query_string,
            entities,
        ),
        ODataPath::PropertyAccess {
            entity,
            key,
            property,
        } => {
            let records = store.get(entity.set_name());
            if let Some(record) = records.and_then(|r| {
                draft::find_record(r, entity.key_field(), &key.key_value, key.is_active)
            }) {
                if let Some(val) = record.get(&property) {
                    return json!({ "value": val });
                }
            }
            json!({"error": {"code": "404", "message": format!("Property '{}' not found", property)}})
        }
        _ => json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}}),
    }
}

// ── Sub-Collection handler ──────────────────────────────────────────
/// Liefert die Kind-Eintraege einer Komposition.
/// Z.B. Orders('O001')/Items → alle OrderItems mit OrderID == 'O001'.
/// Filtert nach dem IsActiveEntity-Status des Eltern-Datensatzes.
fn handle_sub_collection(
    parent_entity: &dyn ODataEntity,
    parent_key: &crate::routing::EntityKeyInfo,
    child_entity: &dyn ODataEntity,
    query: &str,
    state: &AppState,
) -> Response {
    let store = state.data_store.read().unwrap();
    json_response(draft::read_sub_collection(
        &store,
        parent_entity,
        parent_key,
        child_entity,
        query,
        &state.entities,
    ))
}

// ── Static file serving ─────────────────────────────────────────────
fn handle_static_files(path: &str, state: &AppState) -> Response {
    let raw_path = urlencoding::decode(path)
        .unwrap_or_default()
        .into_owned();
    let mut relative = raw_path.trim_start_matches('/').to_string();

    // Entity-spezifischer App-Pfad: /apps/{EntitySet}/...
    // Jede Entitaet bekommt ein eigenes Manifest mit passender Default-Route.
    let mut entity_hint: Option<String> = None;
    if relative.starts_with("apps/") {
        let rest = &relative["apps/".len()..];
        if let Some(slash_pos) = rest.find('/') {
            let candidate = rest[..slash_pos].to_string();
            if state.entity_manifests.contains_key(&candidate) {
                entity_hint = Some(candidate.clone());
                info!("entity hint: {}", candidate);
                relative = rest[slash_pos + 1..].to_string();
            }
        }
    }

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
        let manifest_body = entity_hint
            .as_ref()
            .and_then(|name| state.entity_manifests.get(name))
            .unwrap_or(&state.manifest_json);
        info!(
            "Serving manifest.json for entity: {}",
            entity_hint.as_deref().unwrap_or("default")
        );
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(manifest_body.clone())).unwrap();
    }

    // Component.js wird dynamisch generiert — der Klassenname muss zum
    // sap.app.id im jeweiligen Manifest passen, sonst cached UI5 den
    // falschen Component fuer die zweite App.
    if relative == "Component.js" {
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
    if relative == "flp.html" {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(state.flp_html.clone())).unwrap();
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
                let mut builder = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html;charset=utf-8");
                for (k, v) in cors_headers() {
                    builder = builder.header(k, v);
                }
                return builder.body(Body::from(state.flp_html.clone())).unwrap();
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
        info!("serving static file: {}", target.display());
        return serve_file(&target);
    }

    // SPA fallback for extensionless routes (skip /sap/ API paths)
    if Path::new(&relative).extension().is_none() && !raw_path.starts_with("/sap/") {
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html;charset=utf-8");
        for (k, v) in cors_headers() {
            builder = builder.header(k, v);
        }
        return builder.body(Body::from(state.flp_html.clone())).unwrap();
    }

    error_response(404, &format!("Resource not found: {}", raw_path))
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
        info!("favicon");
        return favicon_response();
    }

    // Entity-bezogene Pfade ueber den zentralen Router aufloesen
    let parsed = resolve_odata_path(path, &state.entities);
    info!("{parsed:?}");
    match parsed.path {
        ODataPath::Entity { .. } => match method {
            Method::GET => handle_single_entity(path, query, &state),
            Method::PATCH => {
                let json_body: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => return error_response(400, "Invalid JSON body"),
                };
                let resp = handle_patch_entity(path, &json_body, &state);
                state.save_data();
                resp
            }
            Method::DELETE => {
                let resp = handle_delete_entity(path, &state);
                state.save_data();
                resp
            }
            _ => error_response(405, "Method not allowed"),
        },
        ODataPath::Action { .. } => {
            if method == Method::POST {
                let resp = handle_draft_action(path, &state);
                state.save_data();
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
                let mut store = state.data_store.write().unwrap();
                let resp = draft_to_response(draft::create_sub_item(
                    &mut store,
                    parent_entity,
                    &parent_key,
                    child_entity,
                    &body_str,
                ));
                drop(store);
                state.save_data();
                resp
            }
            _ => error_response(405, "Method not allowed"),
        },
        ODataPath::PropertyAccess {
            entity,
            key,
            property,
        } => {
            let store = state.data_store.read().unwrap();
            let records = store.get(entity.set_name());
            if let Some(record) = records.and_then(|r| {
                draft::find_record(r, entity.key_field(), &key.key_value, key.is_active)
            }) {
                if let Some(val) = record.get(&property) {
                    return json_response(json!({ "value": val }));
                }
            }
            error_response(404, &format!("Property '{}' not found", property))
        }
        _ => handle_static_files(path, &state),
    }
}
