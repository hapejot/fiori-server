use std::collections::HashMap;
use std::path::Path;

use log::info;
use serde_json::{json, Value};

use super::generic::EntityConfig;

/// Erzeugt Meta-Entity-Daten aus den geladenen EntityConfig-Structs.
/// Gibt (EntityConfigs, EntityFields, EntityFacets, EntityNavigations, EntityTableFacets) zurueck.
pub fn generate_meta_data(
    configs: &[EntityConfig],
) -> (Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>) {
    let mut config_records = Vec::new();
    let mut field_records = Vec::new();
    let mut facet_records = Vec::new();
    let mut nav_records = Vec::new();
    let mut table_facet_records = Vec::new();

    for config in configs {
        let tile = config.tile.as_ref();
        let ann = config.annotations.as_ref();
        let hi = ann.map(|a| &a.header_info);

        // ── EntityConfigs-Datensatz ─────────────────────────────────
        config_records.push(json!({
            "SetName":              config.set_name,
            "KeyField":             config.key_field,
            "TypeName":             config.type_name,
            "ParentSetName":        config.parent_set_name.as_deref().unwrap_or(""),
            "TileTitle":            tile.map(|t| t.title.as_str()).unwrap_or(""),
            "TileDescription":      tile.and_then(|t| t.description.as_deref()).unwrap_or(""),
            "TileIcon":             tile.and_then(|t| t.icon.as_deref()).unwrap_or(""),
            "HeaderTypeName":       hi.map(|h| h.type_name.as_str()).unwrap_or(""),
            "HeaderTypeNamePlural": hi.map(|h| h.type_name_plural.as_str()).unwrap_or(""),
            "HeaderTitlePath":      hi.map(|h| h.title_path.as_str()).unwrap_or(""),
            "HeaderDescriptionPath":hi.map(|h| h.description_path.as_str()).unwrap_or(""),
            "SelectionFields":      ann.map(|a| a.selection_fields.join(",")).unwrap_or_default(),
        }));

        // ── EntityFields-Datensaetze ────────────────────────────────
        // LineItem-Info auf Feld-Ebene zuordnen (nur "echte" Felder, keine Nav-Pfade)
        let line_items: HashMap<&str, _> = ann
            .map(|a| {
                a.line_item
                    .iter()
                    .filter(|li| !li.name.contains('/'))
                    .map(|li| (li.name.as_str(), li))
                    .collect()
            })
            .unwrap_or_default();

        for (idx, field) in config.fields.iter().enumerate() {
            let li = line_items.get(field.name.as_str());
            field_records.push(json!({
                "FieldID":                format!("{}_{}", config.set_name, field.name),
                "SetName":                config.set_name,
                "FieldName":              field.name,
                "Label":                  field.label,
                "EdmType":                field.edm_type,
                "MaxLength":              field.max_length,
                "Precision":              field.precision,
                "Scale":                  field.scale,
                "IsImmutable":            field.immutable,
                "SemanticObject":         field.semantic_object.as_deref().unwrap_or(""),
                "SortOrder":              idx as u32,
                "ShowInLineItem":         li.is_some(),
                "LineItemImportance":     li.and_then(|l| l.importance.as_deref()).unwrap_or(""),
                "LineItemLabel":          li.and_then(|l| l.label.as_deref()).unwrap_or(""),
                "LineItemCriticalityPath":li.and_then(|l| l.criticality_path.as_deref()).unwrap_or(""),
                "LineItemSemanticObject": li.and_then(|l| l.semantic_object.as_deref()).unwrap_or(""),
            }));
        }

        // ── EntityFacets-Datensaetze ────────────────────────────────
        if let Some(ann) = ann {
            let fg_map: HashMap<&str, _> = ann
                .field_groups
                .iter()
                .map(|fg| (fg.qualifier.as_str(), fg))
                .collect();

            for (idx, section) in ann.facet_sections.iter().enumerate() {
                let fg = fg_map.get(section.field_group_qualifier.as_str());
                facet_records.push(json!({
                    "FacetID":              format!("{}_{}", config.set_name, section.id),
                    "SetName":              config.set_name,
                    "SectionLabel":         section.label,
                    "SectionId":            section.id,
                    "FieldGroupQualifier":  section.field_group_qualifier,
                    "FieldGroupLabel":      section.field_group_label,
                    "FieldGroupFields":     fg.map(|f| f.fields.join(",")).unwrap_or_default(),
                    "SortOrder":            idx as u32,
                }));
            }

            // ── EntityTableFacets-Datensaetze ───────────────────────
            for (idx, tf) in ann.table_facets.iter().enumerate() {
                table_facet_records.push(json!({
                    "TableFacetID":       format!("{}_{}", config.set_name, tf.id),
                    "SetName":            config.set_name,
                    "FacetLabel":         tf.label,
                    "FacetId":            tf.id,
                    "NavigationProperty": tf.navigation_property,
                    "SortOrder":          idx as u32,
                }));
            }
        }

        // ── EntityNavigations-Datensaetze ───────────────────────────
        for (idx, nav) in config.navigation_properties.iter().enumerate() {
            nav_records.push(json!({
                "NavID":          format!("{}_{}", config.set_name, nav.name),
                "SetName":        config.set_name,
                "NavName":        nav.name,
                "TargetType":     nav.target_type,
                "TargetSet":      nav.target_set,
                "IsCollection":   nav.is_collection,
                "ForeignKey":     nav.foreign_key.as_deref().unwrap_or(""),
                "SortOrder":      idx as u32,
            }));
        }
    }

    (config_records, field_records, facet_records, nav_records, table_facet_records)
}

/// Schreibt Meta-Entity-Daten als JSON-Dateien ins Data-Verzeichnis.
pub fn write_meta_data(data_dir: &Path, configs: &[EntityConfig]) {
    let (configs_data, fields_data, facets_data, nav_data, table_facet_data) =
        generate_meta_data(configs);

    std::fs::create_dir_all(data_dir).ok();

    let write_json = |name: &str, data: &[Value]| {
        let path = data_dir.join(format!("{}.json", name));
        match serde_json::to_string_pretty(data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("  WARNUNG: Konnte {} nicht schreiben: {}", path.display(), e);
                } else {
                    info!("  Meta-Daten: {} ({} Eintraege)", name, data.len());
                }
            }
            Err(e) => eprintln!("  WARNUNG: JSON-Fehler fuer {}: {}", name, e),
        }
    };

    write_json("EntityConfigs", &configs_data);
    write_json("EntityFields", &fields_data);
    write_json("EntityFacets", &facets_data);
    write_json("EntityNavigations", &nav_data);
    write_json("EntityTableFacets", &table_facet_data);
}

/// Publiziert Meta-Entity-Datensaetze zurueck in eine Config-JSON-Datei.
///
/// Nicht nachverfolgte Felder (navigation_properties, header_facets, data_points,
/// table_facets, Navigations-Pfad-LineItems) werden aus der Originaldatei beibehalten.
pub fn publish_entity_config(
    store: &HashMap<String, Vec<Value>>,
    config_dir: &Path,
    set_name: &str,
) -> Result<Value, String> {
    // 1. EntityConfigs-Datensatz finden
    let config_record = store
        .get("EntityConfigs")
        .and_then(|records| {
            records.iter().find(|r| {
                r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                    && r.get("IsActiveEntity")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
            })
        })
        .ok_or_else(|| format!("Entity-Config '{}' nicht gefunden", set_name))?;

    // 2. Zugehoerige EntityFields sammeln
    let mut fields: Vec<&Value> = store
        .get("EntityFields")
        .map(|records| {
            records
                .iter()
                .filter(|r| {
                    r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                        && r.get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                })
                .collect()
        })
        .unwrap_or_default();
    fields.sort_by_key(|f| f.get("SortOrder").and_then(|v| v.as_i64()).unwrap_or(999));

    // 3. Zugehoerige EntityFacets sammeln
    let mut facets: Vec<&Value> = store
        .get("EntityFacets")
        .map(|records| {
            records
                .iter()
                .filter(|r| {
                    r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                        && r.get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                })
                .collect()
        })
        .unwrap_or_default();
    facets.sort_by_key(|f| f.get("SortOrder").and_then(|v| v.as_i64()).unwrap_or(999));

    // 4. Zugehoerige EntityNavigations sammeln
    let mut navigations: Vec<&Value> = store
        .get("EntityNavigations")
        .map(|records| {
            records
                .iter()
                .filter(|r| {
                    r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                        && r.get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                })
                .collect()
        })
        .unwrap_or_default();
    navigations.sort_by_key(|f| f.get("SortOrder").and_then(|v| v.as_i64()).unwrap_or(999));

    // 5. Zugehoerige EntityTableFacets sammeln
    let mut table_facets: Vec<&Value> = store
        .get("EntityTableFacets")
        .map(|records| {
            records
                .iter()
                .filter(|r| {
                    r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                        && r.get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                })
                .collect()
        })
        .unwrap_or_default();
    table_facets.sort_by_key(|f| f.get("SortOrder").and_then(|v| v.as_i64()).unwrap_or(999));

    // 6. Originaldatei laden (fuer nicht-nachverfolgte Felder)
    let config_path = config_dir.join(format!("{}.json", set_name));
    let original: Option<Value> = if config_path.is_file() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    } else {
        None
    };

    let str_val = |record: &Value, field: &str| -> String {
        record
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    // ── Fields-Array aufbauen ─────────────────────────────────────
    let config_fields: Vec<Value> = fields
        .iter()
        .map(|f| {
            let mut field = json!({
                "name":    str_val(f, "FieldName"),
                "label":   str_val(f, "Label"),
                "edm_type": str_val(f, "EdmType"),
            });
            if let Some(ml) = f.get("MaxLength").and_then(|v| v.as_i64()) {
                if ml > 0 {
                    field["max_length"] = json!(ml);
                }
            }
            if let Some(p) = f.get("Precision").and_then(|v| v.as_i64()) {
                if p > 0 {
                    field["precision"] = json!(p);
                }
            }
            if let Some(s) = f.get("Scale").and_then(|v| v.as_i64()) {
                if s > 0 {
                    field["scale"] = json!(s);
                }
            }
            if f.get("IsImmutable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                field["immutable"] = json!(true);
            }
            let so = str_val(f, "SemanticObject");
            if !so.is_empty() {
                field["semantic_object"] = json!(so);
            }
            field
        })
        .collect();

    // ── LineItem-Array aufbauen (inkl. Nav-Pfad-Eintraege aus Original) ──
    let mut config_line_items: Vec<Value> = fields
        .iter()
        .filter(|f| {
            f.get("ShowInLineItem")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .map(|f| {
            let mut li = json!({ "name": str_val(f, "FieldName") });
            let imp = str_val(f, "LineItemImportance");
            if !imp.is_empty() {
                li["importance"] = json!(imp);
            }
            let lab = str_val(f, "LineItemLabel");
            if !lab.is_empty() {
                li["label"] = json!(lab);
            }
            let cp = str_val(f, "LineItemCriticalityPath");
            if !cp.is_empty() {
                li["criticality_path"] = json!(cp);
            }
            let so = str_val(f, "LineItemSemanticObject");
            if !so.is_empty() {
                li["semantic_object"] = json!(so);
            }
            li
        })
        .collect();

    // Navigations-Pfad-LineItems aus Original beibehalten (z.B. "Customer/CustomerName")
    if let Some(orig_li) = original
        .as_ref()
        .and_then(|o| o.get("annotations"))
        .and_then(|a| a.get("line_item"))
        .and_then(|v| v.as_array())
    {
        for item in orig_li {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                if name.contains('/') {
                    config_line_items.push(item.clone());
                }
            }
        }
    }

    // ── FacetSections + FieldGroups aufbauen ─────────────────────
    let config_facet_sections: Vec<Value> = facets
        .iter()
        .map(|f| {
            json!({
                "label":                str_val(f, "SectionLabel"),
                "id":                   str_val(f, "SectionId"),
                "field_group_qualifier": str_val(f, "FieldGroupQualifier"),
                "field_group_label":    str_val(f, "FieldGroupLabel"),
            })
        })
        .collect();

    let config_field_groups: Vec<Value> = facets
        .iter()
        .map(|f| {
            let fields_str = str_val(f, "FieldGroupFields");
            let field_list: Vec<String> = fields_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            json!({
                "qualifier": str_val(f, "FieldGroupQualifier"),
                "fields": field_list,
            })
        })
        .collect();

    // SelectionFields: kommaseparierter String → Array
    let sf_str = str_val(config_record, "SelectionFields");
    let selection_fields: Vec<String> = sf_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Nicht-nachverfolgte Felder aus Original beibehalten
    let orig_header_facets = original
        .as_ref()
        .and_then(|o| o.get("annotations"))
        .and_then(|a| a.get("header_facets"))
        .cloned()
        .unwrap_or(json!([]));
    let orig_data_points = original
        .as_ref()
        .and_then(|o| o.get("annotations"))
        .and_then(|a| a.get("data_points"))
        .cloned()
        .unwrap_or(json!([]));

    // ── NavigationProperties-Array aus Meta-Daten aufbauen ───────
    let config_nav_props: Vec<Value> = navigations
        .iter()
        .map(|n| {
            let mut nav = json!({
                "name":        str_val(n, "NavName"),
                "target_type": str_val(n, "TargetType"),
                "target_set":  str_val(n, "TargetSet"),
                "is_collection": n.get("IsCollection").and_then(|v| v.as_bool()).unwrap_or(false),
            });
            let fk = str_val(n, "ForeignKey");
            if !fk.is_empty() {
                nav["foreign_key"] = json!(fk);
            }
            nav
        })
        .collect();

    // ── TableFacets-Array aus Meta-Daten aufbauen ───────────────
    let config_table_facets: Vec<Value> = table_facets
        .iter()
        .map(|tf| {
            json!({
                "label":               str_val(tf, "FacetLabel"),
                "id":                  str_val(tf, "FacetId"),
                "navigation_property": str_val(tf, "NavigationProperty"),
            })
        })
        .collect();

    // ── Config-JSON zusammenbauen ────────────────────────────────
    let parent_sn = str_val(config_record, "ParentSetName");
    let tile_title = str_val(config_record, "TileTitle");

    let mut result = json!({
        "set_name":   set_name,
        "key_field":  str_val(config_record, "KeyField"),
        "type_name":  str_val(config_record, "TypeName"),
        "fields":     config_fields,
        "navigation_properties": config_nav_props,
        "annotations": {
            "selection_fields": selection_fields,
            "line_item":        config_line_items,
            "header_info": {
                "type_name":        str_val(config_record, "HeaderTypeName"),
                "type_name_plural": str_val(config_record, "HeaderTypeNamePlural"),
                "title_path":       str_val(config_record, "HeaderTitlePath"),
                "description_path": str_val(config_record, "HeaderDescriptionPath"),
            },
            "header_facets":    orig_header_facets,
            "data_points":      orig_data_points,
            "facet_sections":   config_facet_sections,
            "field_groups":     config_field_groups,
            "table_facets":     config_table_facets,
        }
    });

    if !parent_sn.is_empty() {
        result["parent_set_name"] = json!(parent_sn);
    }

    if !tile_title.is_empty() {
        let mut tile = json!({ "title": tile_title });
        let desc = str_val(config_record, "TileDescription");
        if !desc.is_empty() {
            tile["description"] = json!(desc);
        }
        let icon = str_val(config_record, "TileIcon");
        if !icon.is_empty() {
            tile["icon"] = json!(icon);
        }
        result["tile"] = tile;
    }

    // Datei schreiben
    std::fs::create_dir_all(config_dir).map_err(|e| format!("Verzeichnis-Fehler: {}", e))?;
    let json_str =
        serde_json::to_string_pretty(&result).map_err(|e| format!("JSON-Fehler: {}", e))?;
    std::fs::write(&config_path, &json_str)
        .map_err(|e| format!("Schreib-Fehler: {}", e))?;

    info!(
        "  Config publiziert: {} -> {}",
        set_name,
        config_path.display()
    );

    Ok(config_record.clone())
}
