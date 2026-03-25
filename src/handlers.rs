use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::{json, Value};
use std::path::Path;

use crate::app_state::AppState;
use crate::entity::{extract_set_name, ODataEntity};
use crate::query::{parse_query_string, query_collection};
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
            return json_response(query_collection(entity, &qs, &data.entities));
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer $count-Handler fuer beliebige EntitySets.
pub async fn count_handler(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(set_name) = extract_set_name(req.path()) {
        if let Some(entity) = data.find_entity(set_name) {
            let mut builder = HttpResponse::Ok();
            builder.insert_header(("Content-Type", "text/plain;charset=utf-8"));
            builder.insert_header(("OData-Version", "4.0"));
            for (k, v) in cors_headers() {
                builder.insert_header((k, v));
            }
            return builder.body(entity.mock_data().len().to_string());
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer Single-Entity-Handler: /SetName('key')
pub async fn single_entity_handler(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let path = req.path();
    let qs = parse_query_string(req.query_string());

    for entity in state.entities.iter() {
        let prefix = format!("{}/{}('", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            if let Some(key_value) = rest.strip_suffix("')") {
                let data = entity.mock_data();
                if let Some(record) = data
                    .iter()
                    .find(|r| r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value))
                {
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
                let cs_resp = format!("--{}--\r\n", cs_boundary);
                let part_resp = format!(
                    "Content-Type: multipart/mixed; boundary={}\r\nContent-Length: {}\r\n\r\n{}",
                    cs_boundary,
                    cs_resp.len(),
                    cs_resp
                );
                response_parts.push(part_resp);
            }
            continue;
        }

        let lines: Vec<&str> = segment.lines().collect();
        let request_line = lines.iter().find(|l| l.starts_with("GET "));
        if let Some(request_line) = request_line {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let rel_url = parts.get(1).copied().unwrap_or("");
            let resp_json = handle_batch_get(rel_url, &data.entities);
            let resp_body = serde_json::to_string(&resp_json).unwrap_or_default();

            let part_resp = format!(
                "Content-Type: application/http\r\n\
                 Content-Transfer-Encoding: binary\r\n\
                 \r\n\
                 HTTP/1.1 200 OK\r\n\
                 Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                 OData-Version: 4.0\r\n\
                 Content-Length: {}\r\n\
                 \r\n\
                 {}",
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

/// Generischer Batch-GET – loest Pfade ueber die Entity-Registry auf.
fn handle_batch_get(rel_url: &str, entities: &[&dyn ODataEntity]) -> Value {
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

        // Collection
        if path == set_path {
            return query_collection(*entity, &qs, entities);
        }

        // $count
        if path == count_path {
            return json!({"value": entity.mock_data().len()});
        }

        // Single entity: /SetName('key')
        let prefix = format!("{}('", set_path);
        if let Some(rest) = path.strip_prefix(&prefix) {
            if let Some(key_value) = rest.strip_suffix("')") {
                let data = entity.mock_data();
                if let Some(record) = data
                    .iter()
                    .find(|r| r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value))
                {
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

pub async fn catch_all(req: HttpRequest, _body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let path = req.path();

    if req.method() == actix_web::http::Method::OPTIONS {
        return options_handler().await;
    }

    if path == "/favicon.ico" || path == "/favicon.svg" {
        return favicon_response();
    }

    // Single entity: /BASE_PATH/SetName('key') – generisch ueber Registry
    for entity in data.entities.iter() {
        let prefix = format!("{}/{}", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            if rest.starts_with("('") && rest.ends_with("')") {
                return single_entity_handler(req, data).await;
            }
        }
    }

    static_files(req, data).await
}
