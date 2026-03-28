use serde_json::{json, Value};
use std::collections::HashMap;

use crate::entity::ODataEntity;
use crate::query::{parse_expand_names, parse_query_string, query_collection_from};
use crate::routing::EntityKeyInfo;
use crate::BASE_PATH;

pub type Store = HashMap<String, Vec<Value>>;

// ── Lookup-Helfer ───────────────────────────────────────────────────

/// Findet einen Datensatz per Key + IsActiveEntity.
pub fn find_record<'a>(
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

/// Findet einen Datensatz mutabel per Key + IsActiveEntity.
pub fn find_record_mut<'a>(
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

/// Entfernt alle Datensaetze mit gegebenem Key + IsActiveEntity.
pub fn remove_records(
    records: &mut Vec<Value>,
    key_field: &str,
    key_value: &str,
    is_active: bool,
) {
    records.retain(|r| {
        !(r.get(key_field).and_then(|v| v.as_str()) == Some(key_value)
            && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(is_active))
    });
}

// ── Draft-Flag-Helfer ───────────────────────────────────────────────

pub fn set_draft_flags(record: &mut Value, is_active: bool, has_active: bool, has_draft: bool) {
    if let Some(obj) = record.as_object_mut() {
        obj.insert("IsActiveEntity".to_string(), json!(is_active));
        obj.insert("HasActiveEntity".to_string(), json!(has_active));
        obj.insert("HasDraftEntity".to_string(), json!(has_draft));
    }
}

pub fn inject_odata_context(record: &mut Value, set_name: &str) {
    if let Some(obj) = record.as_object_mut() {
        obj.insert(
            "@odata.context".to_string(),
            json!(format!("{}/$metadata#{}/$entity", BASE_PATH, set_name)),
        );
    }
}

fn is_draft_field(k: &str) -> bool {
    k == "IsActiveEntity" || k == "HasActiveEntity" || k == "HasDraftEntity"
}

/// DraftAdministrativeData fuer $expand einfuegen.
fn inject_draft_admin_data(record: &mut Value, key_field: &str) {
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

// ── Kompositions-Kind-Helfer ────────────────────────────────────────

/// Kopiert aktive Kind-Datensaetze als Drafts (fuer draftEdit).
fn copy_children_as_drafts(
    store: &mut Store,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    let parent_key_field = parent_entity.key_field();
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        let drafts: Vec<Value> = store
            .get(child.set_name())
            .map(|recs| {
                recs.iter()
                    .filter(|r| {
                        r.get(parent_key_field).and_then(|v| v.as_str()) == Some(parent_key_value)
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

/// Uebernimmt Draft-Kinder als aktive Kinder (fuer draftActivate).
fn activate_children(
    store: &mut Store,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    let parent_key_field = parent_entity.key_field();
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        if let Some(child_records) = store.get_mut(child.set_name()) {
            // Draft-Items sammeln
            let draft_items: Vec<Value> = child_records
                .iter()
                .filter(|r| {
                    r.get(parent_key_field).and_then(|v| v.as_str()) == Some(parent_key_value)
                        && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false)
                })
                .cloned()
                .collect();

            // Alle Items dieses Parents entfernen (aktive + drafts)
            child_records.retain(|r| {
                r.get(parent_key_field).and_then(|v| v.as_str()) != Some(parent_key_value)
            });

            // Draft-Items als aktive Items einfuegen
            for mut item in draft_items {
                set_draft_flags(&mut item, true, false, false);
                child_records.push(item);
            }
        }
    }
}

/// Entfernt Draft-Kinder (fuer DELETE / Discard).
fn remove_child_drafts(
    store: &mut Store,
    parent_entity: &dyn ODataEntity,
    parent_key_value: &str,
    entities: &[&dyn ODataEntity],
) {
    let parent_key_field = parent_entity.key_field();
    for child in entities {
        if child.parent_set_name() != Some(parent_entity.set_name()) {
            continue;
        }
        if let Some(child_recs) = store.get_mut(child.set_name()) {
            child_recs.retain(|r| {
                !(r.get(parent_key_field).and_then(|v| v.as_str()) == Some(parent_key_value)
                    && r.get("IsActiveEntity").and_then(|v| v.as_bool()) == Some(false))
            });
        }
    }
}

// ── Core CRUD ───────────────────────────────────────────────────────

/// GET einzelnes Entity mit $expand-Unterstuetzung.
pub fn read_entity(
    store: &Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    is_active: bool,
    query: &str,
    entities: &[&dyn ODataEntity],
) -> (u16, Value) {
    let records = match store.get(entity.set_name()) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Entity set not found"}}),
            )
        }
    };
    let record = match find_record(records, entity.key_field(), key_value, is_active) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": format!("Entity with {}='{}' not found.", entity.key_field(), key_value)}}),
            )
        }
    };

    let qs = parse_query_string(query);
    let mut result = record.clone();
    inject_odata_context(&mut result, entity.set_name());

    if let Some(expand) = qs.get("$expand") {
        if !expand.is_empty() {
            let nav_names = parse_expand_names(expand);
            let nav_refs: Vec<&str> = nav_names.iter().map(|s| s.as_str()).collect();
            entity.expand_record(&mut result, &nav_refs, entities, store);
            if nav_refs.iter().any(|n| *n == "DraftAdministrativeData") {
                inject_draft_admin_data(&mut result, entity.key_field());
            }
        }
    }
    (200, result)
}

/// PATCH Entity – respektiert immutable Felder.
pub fn patch_entity(
    store: &mut Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    is_active: bool,
    patch: &Value,
) -> (u16, Value) {
    let records = match store.get_mut(entity.set_name()) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Entity set not found"}}),
            )
        }
    };
    let record = match find_record_mut(records, entity.key_field(), key_value, is_active) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": format!("Entity with {}='{}' not found.", entity.key_field(), key_value)}}),
            )
        }
    };

    let immutable_fields: Vec<&str> = entity
        .fields_def()
        .unwrap_or(&[])
        .iter()
        .filter(|f| f.immutable)
        .map(|f| f.name)
        .collect();

    if let Some(patch_obj) = patch.as_object() {
        if let Some(rec_obj) = record.as_object_mut() {
            for (k, v) in patch_obj {
                if is_draft_field(k) || immutable_fields.contains(&k.as_str()) {
                    continue;
                }
                rec_obj.insert(k.clone(), v.clone());
            }
        }
    }

    let mut result = record.clone();
    inject_odata_context(&mut result, entity.set_name());
    (200, result)
}

/// DELETE Entity (Draft verwerfen) – inklusive Kind-Drafts.
pub fn delete_entity(
    store: &mut Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    is_active: bool,
    entities: &[&dyn ODataEntity],
) -> (u16, Value) {
    let key_field = entity.key_field();
    let set_name = entity.set_name();

    let found = store
        .get(set_name)
        .map(|r| find_record(r, key_field, key_value, is_active).is_some())
        .unwrap_or(false);

    if !found {
        return (
            404,
            json!({"error": {"code": "404", "message": "Entity not found."}}),
        );
    }

    if let Some(records) = store.get_mut(set_name) {
        remove_records(records, key_field, key_value, is_active);

        // Draft geloescht → HasDraftEntity=false am aktiven, Kinder aufraeumen
        if !is_active {
            if let Some(active) = find_record_mut(records, key_field, key_value, true) {
                if let Some(obj) = active.as_object_mut() {
                    obj.insert("HasDraftEntity".to_string(), json!(false));
                }
            }
            remove_child_drafts(store, entity, key_value, entities);
        }
    }

    (204, json!(null))
}

// ── Draft-Actions ───────────────────────────────────────────────────

pub fn draft_edit(
    store: &mut Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    entities: &[&dyn ODataEntity],
) -> (u16, Value) {
    let key_field = entity.key_field();
    let set_name = entity.set_name();

    let records = match store.get_mut(set_name) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Entity set not found."}}),
            )
        }
    };

    let active = match find_record(records, key_field, key_value, true) {
        Some(a) => a.clone(),
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Active entity not found."}}),
            )
        }
    };

    // HasDraftEntity=true am aktiven Datensatz
    if let Some(active_rec) = find_record_mut(records, key_field, key_value, true) {
        if let Some(obj) = active_rec.as_object_mut() {
            obj.insert("HasDraftEntity".to_string(), json!(true));
        }
    }

    // Draft erzeugen
    let mut draft = active;
    set_draft_flags(&mut draft, false, true, false);
    inject_odata_context(&mut draft, set_name);
    let result = draft.clone();
    records.push(draft);

    // Kompositions-Kinder als Draft kopieren
    copy_children_as_drafts(store, entity, key_value, entities);

    (201, result)
}

pub fn draft_activate(
    store: &mut Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    entities: &[&dyn ODataEntity],
) -> (u16, Value) {
    let key_field = entity.key_field();
    let set_name = entity.set_name();

    let records = match store.get_mut(set_name) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Entity set not found."}}),
            )
        }
    };

    let draft = match find_record(records, key_field, key_value, false) {
        Some(d) => d.clone(),
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Draft not found."}}),
            )
        }
    };

    // Draft-Daten in aktiven Datensatz uebernehmen oder neuen aktiven erzeugen
    let has_active = draft
        .get("HasActiveEntity")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if has_active {
        // Existierenden aktiven Datensatz aktualisieren
        if let Some(active) = find_record_mut(records, key_field, key_value, true) {
            if let (Some(active_obj), Some(draft_obj)) =
                (active.as_object_mut(), draft.as_object())
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
        // Neuer Datensatz: Draft zum aktiven Datensatz befoerdern
        let mut new_active = draft.clone();
        set_draft_flags(&mut new_active, true, false, false);
        records.push(new_active);
    }

    // Draft entfernen
    remove_records(records, key_field, key_value, false);

    let result = find_record(records, key_field, key_value, true).cloned();

    // Kompositions-Kinder aktivieren
    activate_children(store, entity, key_value, entities);

    match result {
        Some(mut r) => {
            inject_odata_context(&mut r, set_name);
            (200, r)
        }
        None => (
            404,
            json!({"error": {"code": "404", "message": "Activated entity not found."}}),
        ),
    }
}

pub fn draft_prepare(
    store: &Store,
    entity: &dyn ODataEntity,
    key_value: &str,
    is_active: bool,
) -> (u16, Value) {
    let records = match store.get(entity.set_name()) {
        Some(r) => r,
        None => {
            return (
                404,
                json!({"error": {"code": "404", "message": "Entity set not found."}}),
            )
        }
    };
    match find_record(records, entity.key_field(), key_value, is_active) {
        Some(r) => {
            let mut result = r.clone();
            inject_odata_context(&mut result, entity.set_name());
            (200, result)
        }
        None => (
            404,
            json!({"error": {"code": "404", "message": "Entity not found for draftPrepare."}}),
        ),
    }
}

// ── Sub-Collection ──────────────────────────────────────────────────

/// Liefert Kind-Eintraege einer Komposition, gefiltert nach Parent-Key + IsActiveEntity.
pub fn read_sub_collection(
    store: &Store,
    parent_entity: &dyn ODataEntity,
    parent_key: &EntityKeyInfo,
    child_entity: &dyn ODataEntity,
    query: &str,
    entities: &[&dyn ODataEntity],
) -> Value {
    let parent_key_field = parent_entity.key_field();
    let child_records: Vec<Value> = store
        .get(child_entity.set_name())
        .map(|records| {
            records
                .iter()
                .filter(|r| {
                    r.get(parent_key_field).and_then(|v| v.as_str())
                        == Some(&parent_key.key_value)
                        && r.get("IsActiveEntity").and_then(|v| v.as_bool())
                            == Some(parent_key.is_active)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let qs = parse_query_string(query);
    query_collection_from(child_entity, &child_records, &qs, entities, store)
}

/// Erzeugt einen neuen Draft-Datensatz per POST auf die Collection.
pub fn create_entity(
    store: &mut Store,
    entity: &dyn ODataEntity,
    body: &str,
) -> (u16, Value) {
    let mut new_record: Value = if body.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(body).unwrap_or_else(|_| json!({}))
    };

    if let Some(obj) = new_record.as_object_mut() {
        // Key generieren, falls nicht vorhanden
        let key_field = entity.key_field();
        if !obj.contains_key(key_field) {
            let existing = store
                .get(entity.set_name())
                .map(|r| r.len())
                .unwrap_or(0);
            obj.insert(
                key_field.to_string(),
                json!(format!("NEW{:04}", existing + 1)),
            );
        }

        // Draft-Felder
        obj.insert("IsActiveEntity".to_string(), json!(false));
        obj.insert("HasActiveEntity".to_string(), json!(false));
        obj.insert("HasDraftEntity".to_string(), json!(false));

        // Default-Werte fuer fehlende Felder
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
    inject_odata_context(&mut result, entity.set_name());
    store
        .entry(entity.set_name().to_string())
        .or_insert_with(Vec::new)
        .push(new_record);

    (201, result)
}

/// Erzeugt ein neues Kind-Element in einer Sub-Collection (Komposition).
pub fn create_sub_item(
    store: &mut Store,
    parent_entity: &dyn ODataEntity,
    parent_key: &EntityKeyInfo,
    child_entity: &dyn ODataEntity,
    body: &str,
) -> (u16, Value) {
    let mut new_record: Value = if body.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(body).unwrap_or_else(|_| json!({}))
    };

    if let Some(obj) = new_record.as_object_mut() {
        // Parent-Key eintragen
        obj.entry(parent_entity.key_field().to_string())
            .or_insert_with(|| json!(parent_key.key_value));

        // Kind-Key generieren, falls nicht vorhanden
        let child_key_field = child_entity.key_field();
        if !obj.contains_key(child_key_field) {
            let existing = store
                .get(child_entity.set_name())
                .map(|r| r.len())
                .unwrap_or(0);
            obj.insert(
                child_key_field.to_string(),
                json!(format!("NEW{:04}", existing + 1)),
            );
        }

        // Draft-Felder
        obj.entry("IsActiveEntity".to_string())
            .or_insert(json!(false));
        obj.entry("HasActiveEntity".to_string())
            .or_insert(json!(false));
        obj.entry("HasDraftEntity".to_string())
            .or_insert(json!(false));

        // Default-Werte fuer fehlende Felder
        if let Some(fields) = child_entity.fields_def() {
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

    let result = new_record.clone();
    store
        .entry(child_entity.set_name().to_string())
        .or_insert_with(Vec::new)
        .push(new_record);

    (201, result)
}
