use std::collections::HashMap;
use std::path::Path;

use log::info;
use serde_json::{json, Value};

use super::generic::{
    AnnotationsConfig, DataPointConfig, EntityConfig, FacetSectionConfig, FieldConfig,
    FieldGroupConfig, HeaderFacetConfig, HeaderInfoConfig, LineItemConfig, NavPropertyConfig,
    TableFacetConfig, TileConfig,
};

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
/// Baut EntityConfig-Structs aus den Meta-Datensaetzen auf und serialisiert diese,
/// sodass die gleichen Typen fuer Ein- und Ausgabe verwendet werden.
///
/// Nicht nachverfolgte Felder (header_facets, data_points,
/// Navigations-Pfad-LineItems) werden aus der Originaldatei beibehalten.
pub fn publish_entity_config(
    config_dir: &Path,
    set_name: &str,
    data_store: &dyn crate::data_store::DataStore,
) -> Result<Value, String> {
    let get_records = |name: &str| -> Vec<Value> { data_store.get_records(name) };

    // ── 1. EntityConfigs-Datensatz finden ────────────────────────
    let all_configs = get_records("EntityConfigs");
    let config_record = all_configs
        .iter()
        .find(|r| {
            r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                && r.get("IsActiveEntity")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
        })
        .ok_or_else(|| format!("Entity-Config '{}' nicht gefunden", set_name))?;

    let str_val = |record: &Value, field: &str| -> String {
        record
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let opt_str = |record: &Value, field: &str| -> Option<String> {
        record
            .get(field)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    };

    // ── 2. Zugehoerige Meta-Datensaetze sammeln ─────────────────
    let filter_active = |records: Vec<Value>| -> Vec<Value> {
        let mut filtered: Vec<Value> = records
            .into_iter()
            .filter(|r| {
                r.get("SetName").and_then(|v| v.as_str()) == Some(set_name)
                    && r.get("IsActiveEntity")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
            })
            .collect();
        filtered.sort_by_key(|f| f.get("SortOrder").and_then(|v| v.as_i64()).unwrap_or(999));
        filtered
    };

    let field_records = filter_active(get_records("EntityFields"));
    let facet_records = filter_active(get_records("EntityFacets"));
    let nav_records = filter_active(get_records("EntityNavigations"));
    let table_facet_records = filter_active(get_records("EntityTableFacets"));

    // ── 3. Originaldatei laden (fuer nicht-nachverfolgte Felder) ─
    let config_path = config_dir.join(format!("{}.json", set_name));
    let original: Option<Value> = if config_path.is_file() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    } else {
        None
    };

    // ── 4. FieldConfig-Structs aufbauen ─────────────────────────
    let fields: Vec<FieldConfig> = field_records
        .iter()
        .map(|f| FieldConfig {
            name: str_val(f, "FieldName"),
            label: str_val(f, "Label"),
            edm_type: str_val(f, "EdmType"),
            max_length: f
                .get("MaxLength")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .filter(|&v| v > 0),
            precision: f
                .get("Precision")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .filter(|&v| v > 0),
            scale: f
                .get("Scale")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .filter(|&v| v > 0),
            immutable: f
                .get("IsImmutable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            semantic_object: opt_str(f, "SemanticObject"),
        })
        .collect();

    // ── 5. LineItem-Eintraege aus Field-Metadaten ───────────────
    let mut line_items: Vec<LineItemConfig> = field_records
        .iter()
        .filter(|f| {
            f.get("ShowInLineItem")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .map(|f| LineItemConfig {
            name: str_val(f, "FieldName"),
            label: opt_str(f, "LineItemLabel"),
            importance: opt_str(f, "LineItemImportance"),
            criticality_path: opt_str(f, "LineItemCriticalityPath"),
            navigation_path: None,
            semantic_object: opt_str(f, "LineItemSemanticObject"),
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
                    if let Ok(li) = serde_json::from_value::<LineItemConfig>(item.clone()) {
                        line_items.push(li);
                    }
                }
            }
        }
    }

    // ── 6. FacetSections + FieldGroups ──────────────────────────
    let facet_sections: Vec<FacetSectionConfig> = facet_records
        .iter()
        .map(|f| FacetSectionConfig {
            label: str_val(f, "SectionLabel"),
            id: str_val(f, "SectionId"),
            field_group_qualifier: str_val(f, "FieldGroupQualifier"),
            field_group_label: str_val(f, "FieldGroupLabel"),
        })
        .collect();

    let field_groups: Vec<FieldGroupConfig> = facet_records
        .iter()
        .map(|f| {
            let fields_str = str_val(f, "FieldGroupFields");
            FieldGroupConfig {
                qualifier: str_val(f, "FieldGroupQualifier"),
                fields: fields_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            }
        })
        .collect();

    // ── 7. NavigationProperties ─────────────────────────────────
    let navigation_properties: Vec<NavPropertyConfig> = nav_records
        .iter()
        .map(|n| NavPropertyConfig {
            name: str_val(n, "NavName"),
            target_type: str_val(n, "TargetType"),
            target_set: str_val(n, "TargetSet"),
            is_collection: n
                .get("IsCollection")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            foreign_key: opt_str(n, "ForeignKey"),
        })
        .collect();

    // ── 8. TableFacets ──────────────────────────────────────────
    let table_facets: Vec<TableFacetConfig> = table_facet_records
        .iter()
        .map(|tf| TableFacetConfig {
            label: str_val(tf, "FacetLabel"),
            id: str_val(tf, "FacetId"),
            navigation_property: str_val(tf, "NavigationProperty"),
        })
        .collect();

    // ── 9. SelectionFields ──────────────────────────────────────
    let sf_str = str_val(config_record, "SelectionFields");
    let selection_fields: Vec<String> = sf_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // ── 10. Nicht-nachverfolgte Felder aus Original ─────────────
    let header_facets: Vec<HeaderFacetConfig> = original
        .as_ref()
        .and_then(|o| o.get("annotations"))
        .and_then(|a| a.get("header_facets"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let data_points: Vec<DataPointConfig> = original
        .as_ref()
        .and_then(|o| o.get("annotations"))
        .and_then(|a| a.get("data_points"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // ── 11. HeaderInfo ──────────────────────────────────────────
    let header_info = HeaderInfoConfig {
        type_name: str_val(config_record, "HeaderTypeName"),
        type_name_plural: str_val(config_record, "HeaderTypeNamePlural"),
        title_path: str_val(config_record, "HeaderTitlePath"),
        description_path: str_val(config_record, "HeaderDescriptionPath"),
    };

    // ── 12. Annotations zusammenbauen ───────────────────────────
    let annotations = AnnotationsConfig {
        selection_fields,
        line_item: line_items,
        header_info,
        header_facets,
        data_points,
        facet_sections,
        field_groups,
        table_facets,
    };

    // ── 13. Tile ────────────────────────────────────────────────
    let tile_title = str_val(config_record, "TileTitle");
    let tile = if !tile_title.is_empty() {
        Some(TileConfig {
            title: tile_title,
            description: opt_str(config_record, "TileDescription"),
            icon: opt_str(config_record, "TileIcon"),
        })
    } else {
        None
    };

    // ── 14. EntityConfig aufbauen und serialisieren ─────────────
    let entity_config = EntityConfig {
        set_name: set_name.to_string(),
        key_field: str_val(config_record, "KeyField"),
        type_name: str_val(config_record, "TypeName"),
        parent_set_name: opt_str(config_record, "ParentSetName"),
        fields,
        navigation_properties,
        annotations: Some(annotations),
        tile,
    };

    std::fs::create_dir_all(config_dir).map_err(|e| format!("Verzeichnis-Fehler: {}", e))?;
    let json_str = serde_json::to_string_pretty(&entity_config)
        .map_err(|e| format!("JSON-Fehler: {}", e))?;
    std::fs::write(&config_path, &json_str)
        .map_err(|e| format!("Schreib-Fehler: {}", e))?;

    info!(
        "  Config publiziert: {} -> {}",
        set_name,
        config_path.display()
    );

    Ok(config_record.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_store::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::RwLock;

    // ── Helper: build a test config ─────────────────────────────

    fn test_config() -> EntityConfig {
        EntityConfig {
            set_name: "Products".to_string(),
            key_field: "ProductID".to_string(),
            type_name: "Product".to_string(),
            parent_set_name: None,
            fields: vec![
                FieldConfig {
                    name: "ProductID".to_string(),
                    label: "Produkt-Nr.".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    semantic_object: None,
                },
                FieldConfig {
                    name: "ProductName".to_string(),
                    label: "Produktname".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(80),
                    precision: None,
                    scale: None,
                    immutable: false,
                    semantic_object: None,
                },
                FieldConfig {
                    name: "Price".to_string(),
                    label: "Preis".to_string(),
                    edm_type: "Edm.Decimal".to_string(),
                    max_length: None,
                    precision: Some(15),
                    scale: Some(2),
                    immutable: false,
                    semantic_object: None,
                },
            ],
            navigation_properties: vec![NavPropertyConfig {
                name: "Supplier".to_string(),
                target_type: "Supplier".to_string(),
                target_set: "Suppliers".to_string(),
                is_collection: false,
                foreign_key: Some("SupplierID".to_string()),
            }],
            annotations: Some(AnnotationsConfig {
                selection_fields: vec!["ProductName".to_string()],
                line_item: vec![
                    LineItemConfig {
                        name: "ProductID".to_string(),
                        label: None,
                        importance: Some("High".to_string()),
                        criticality_path: None,
                        navigation_path: None,
                        semantic_object: None,
                    },
                    LineItemConfig {
                        name: "ProductName".to_string(),
                        label: None,
                        importance: None,
                        criticality_path: None,
                        navigation_path: None,
                        semantic_object: None,
                    },
                ],
                header_info: HeaderInfoConfig {
                    type_name: "Produkt".to_string(),
                    type_name_plural: "Produkte".to_string(),
                    title_path: "ProductName".to_string(),
                    description_path: "ProductID".to_string(),
                },
                header_facets: vec![],
                data_points: vec![],
                facet_sections: vec![FacetSectionConfig {
                    label: "Allgemein".to_string(),
                    id: "General".to_string(),
                    field_group_qualifier: "Main".to_string(),
                    field_group_label: "Hauptdaten".to_string(),
                }],
                field_groups: vec![FieldGroupConfig {
                    qualifier: "Main".to_string(),
                    fields: vec![
                        "ProductID".to_string(),
                        "ProductName".to_string(),
                        "Price".to_string(),
                    ],
                }],
                table_facets: vec![],
            }),
            tile: Some(TileConfig {
                title: "Produkte".to_string(),
                description: Some("Produktkatalog".to_string()),
                icon: Some("sap-icon://product".to_string()),
            }),
        }
    }

    fn config_with_table_facets() -> EntityConfig {
        let mut config = test_config();
        config.set_name = "Orders".to_string();
        config.key_field = "OrderID".to_string();
        config.type_name = "Order".to_string();
        config.fields = vec![
            FieldConfig {
                name: "OrderID".to_string(),
                label: "Auftragsnr.".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: Some(10),
                precision: None,
                scale: None,
                immutable: true,
                semantic_object: None,
            },
        ];
        config.navigation_properties = vec![NavPropertyConfig {
            name: "Items".to_string(),
            target_type: "OrderItem".to_string(),
            target_set: "OrderItems".to_string(),
            is_collection: true,
            foreign_key: Some("OrderID".to_string()),
        }];
        let ann = config.annotations.as_mut().unwrap();
        ann.selection_fields = vec![];
        ann.line_item = vec![LineItemConfig {
            name: "OrderID".to_string(),
            label: None,
            importance: Some("High".to_string()),
            criticality_path: None,
            navigation_path: None,
            semantic_object: None,
        }];
        ann.header_info = HeaderInfoConfig {
            type_name: "Auftrag".to_string(),
            type_name_plural: "Auftraege".to_string(),
            title_path: "OrderID".to_string(),
            description_path: "OrderID".to_string(),
        };
        ann.facet_sections = vec![];
        ann.field_groups = vec![];
        ann.table_facets = vec![TableFacetConfig {
            label: "Positionen".to_string(),
            id: "ItemsFacet".to_string(),
            navigation_property: "Items".to_string(),
        }];
        config.tile = None;
        config
    }

    // ── Simple mock DataStore for publish_entity_config ──────────

    struct MockDataStore {
        store: RwLock<HashMap<String, Vec<Value>>>,
    }

    impl MockDataStore {
        fn from_meta(configs: &[EntityConfig]) -> Self {
            let (config_records, field_records, facet_records, nav_records, table_facet_records) =
                generate_meta_data(configs);
            let mut store = HashMap::new();
            store.insert("EntityConfigs".to_string(), config_records);
            store.insert("EntityFields".to_string(), field_records);
            store.insert("EntityFacets".to_string(), facet_records);
            store.insert("EntityNavigations".to_string(), nav_records);
            store.insert("EntityTableFacets".to_string(), table_facet_records);
            Self {
                store: RwLock::new(store),
            }
        }
    }

    impl DataStore for MockDataStore {
        fn get_collection(&self, _: &str, _: &ODataQuery, _: Option<&ParentKey>) -> Result<Value, StoreError> {
            Ok(json!({"value": []}))
        }
        fn count(&self, _: &str, _: &ODataQuery, _: Option<&ParentKey>) -> usize { 0 }
        fn read_entity(&self, _: &str, _: &EntityKey, _: &ODataQuery) -> Result<Value, StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn create_entity(&self, _: &str, _: &Value, _: Option<&ParentKey>) -> Result<Value, StoreError> {
            Err(StoreError::BadRequest("mock".to_string()))
        }
        fn patch_entity(&self, _: &str, _: &EntityKey, _: &Value) -> Result<Value, StoreError> {
            Err(StoreError::BadRequest("mock".to_string()))
        }
        fn delete_entity(&self, _: &str, _: &EntityKey) -> Result<(), StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn draft_edit(&self, _: &str, _: &EntityKey) -> Result<Value, StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn draft_activate(&self, _: &str, _: &EntityKey) -> Result<Value, StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn draft_prepare(&self, _: &str, _: &EntityKey) -> Result<Value, StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn get_property(&self, _: &str, _: &EntityKey, _: &str) -> Result<Value, StoreError> {
            Err(StoreError::NotFound("mock".to_string()))
        }
        fn get_records(&self, set_name: &str) -> Vec<Value> {
            self.store.read().unwrap().get(set_name).cloned().unwrap_or_default()
        }
        fn commit(&self) {}
    }

    // ── generate_meta_data Tests ────────────────────────────────

    #[test]
    fn meta_generates_config_record() {
        let configs = vec![test_config()];
        let (config_records, _, _, _, _) = generate_meta_data(&configs);
        assert_eq!(config_records.len(), 1);
        let cr = &config_records[0];
        assert_eq!(cr["SetName"], "Products");
        assert_eq!(cr["KeyField"], "ProductID");
        assert_eq!(cr["TypeName"], "Product");
        assert_eq!(cr["HeaderTypeName"], "Produkt");
        assert_eq!(cr["HeaderTypeNamePlural"], "Produkte");
        assert_eq!(cr["HeaderTitlePath"], "ProductName");
        assert_eq!(cr["HeaderDescriptionPath"], "ProductID");
        assert_eq!(cr["SelectionFields"], "ProductName");
        assert_eq!(cr["TileTitle"], "Produkte");
        assert_eq!(cr["TileDescription"], "Produktkatalog");
        assert_eq!(cr["TileIcon"], "sap-icon://product");
    }

    #[test]
    fn meta_generates_field_records() {
        let configs = vec![test_config()];
        let (_, field_records, _, _, _) = generate_meta_data(&configs);
        assert_eq!(field_records.len(), 3);

        // First field: ProductID (immutable, in line_item with High importance)
        let f0 = &field_records[0];
        assert_eq!(f0["FieldID"], "Products_ProductID");
        assert_eq!(f0["SetName"], "Products");
        assert_eq!(f0["FieldName"], "ProductID");
        assert_eq!(f0["Label"], "Produkt-Nr.");
        assert_eq!(f0["EdmType"], "Edm.String");
        assert_eq!(f0["MaxLength"], 10);
        assert_eq!(f0["IsImmutable"], true);
        assert_eq!(f0["SortOrder"], 0);
        assert_eq!(f0["ShowInLineItem"], true);
        assert_eq!(f0["LineItemImportance"], "High");

        // Second field: ProductName (in line_item, no importance)
        let f1 = &field_records[1];
        assert_eq!(f1["FieldName"], "ProductName");
        assert_eq!(f1["ShowInLineItem"], true);
        assert_eq!(f1["LineItemImportance"], "");

        // Third field: Price (NOT in line_item)
        let f2 = &field_records[2];
        assert_eq!(f2["FieldName"], "Price");
        assert_eq!(f2["EdmType"], "Edm.Decimal");
        assert_eq!(f2["Precision"], 15);
        assert_eq!(f2["Scale"], 2);
        assert_eq!(f2["ShowInLineItem"], false);
    }

    #[test]
    fn meta_generates_facet_records() {
        let configs = vec![test_config()];
        let (_, _, facet_records, _, _) = generate_meta_data(&configs);
        assert_eq!(facet_records.len(), 1);
        let fr = &facet_records[0];
        assert_eq!(fr["FacetID"], "Products_General");
        assert_eq!(fr["SetName"], "Products");
        assert_eq!(fr["SectionLabel"], "Allgemein");
        assert_eq!(fr["SectionId"], "General");
        assert_eq!(fr["FieldGroupQualifier"], "Main");
        assert_eq!(fr["FieldGroupLabel"], "Hauptdaten");
        assert_eq!(fr["FieldGroupFields"], "ProductID,ProductName,Price");
    }

    #[test]
    fn meta_generates_nav_records() {
        let configs = vec![test_config()];
        let (_, _, _, nav_records, _) = generate_meta_data(&configs);
        assert_eq!(nav_records.len(), 1);
        let nr = &nav_records[0];
        assert_eq!(nr["NavID"], "Products_Supplier");
        assert_eq!(nr["NavName"], "Supplier");
        assert_eq!(nr["TargetType"], "Supplier");
        assert_eq!(nr["TargetSet"], "Suppliers");
        assert_eq!(nr["IsCollection"], false);
        assert_eq!(nr["ForeignKey"], "SupplierID");
    }

    #[test]
    fn meta_generates_table_facet_records() {
        let configs = vec![config_with_table_facets()];
        let (_, _, _, _, table_facet_records) = generate_meta_data(&configs);
        assert_eq!(table_facet_records.len(), 1);
        let tf = &table_facet_records[0];
        assert_eq!(tf["TableFacetID"], "Orders_ItemsFacet");
        assert_eq!(tf["FacetLabel"], "Positionen");
        assert_eq!(tf["FacetId"], "ItemsFacet");
        assert_eq!(tf["NavigationProperty"], "Items");
    }

    #[test]
    fn meta_no_annotations_produces_empty_meta() {
        let config = EntityConfig {
            set_name: "Simple".to_string(),
            key_field: "ID".to_string(),
            type_name: "Simple".to_string(),
            parent_set_name: None,
            fields: vec![FieldConfig {
                name: "ID".to_string(),
                label: "ID".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None, precision: None, scale: None,
                immutable: false, semantic_object: None,
            }],
            navigation_properties: vec![],
            annotations: None,
            tile: None,
        };
        let (configs, fields, facets, navs, tfs) = generate_meta_data(&[config]);
        assert_eq!(configs.len(), 1);
        assert_eq!(fields.len(), 1);
        assert_eq!(facets.len(), 0);
        assert_eq!(navs.len(), 0);
        assert_eq!(tfs.len(), 0);
        // Header info should be empty strings
        assert_eq!(configs[0]["HeaderTypeName"], "");
        assert_eq!(configs[0]["SelectionFields"], "");
        assert_eq!(configs[0]["TileTitle"], "");
    }

    #[test]
    fn meta_multiple_configs() {
        let configs = vec![test_config(), config_with_table_facets()];
        let (config_records, field_records, facet_records, nav_records, tf_records) =
            generate_meta_data(&configs);
        assert_eq!(config_records.len(), 2);
        assert_eq!(config_records[0]["SetName"], "Products");
        assert_eq!(config_records[1]["SetName"], "Orders");
        // Products: 3 fields + Orders: 1 field = 4
        assert_eq!(field_records.len(), 4);
        // Products: 1 facet + Orders: 0 facets = 1
        assert_eq!(facet_records.len(), 1);
        // Products: 1 nav + Orders: 1 nav = 2
        assert_eq!(nav_records.len(), 2);
        // Products: 0 table_facets + Orders: 1 table_facet = 1
        assert_eq!(tf_records.len(), 1);
    }

    #[test]
    fn meta_parent_set_name_propagated() {
        let config = EntityConfig {
            set_name: "OrderItems".to_string(),
            key_field: "ItemID".to_string(),
            type_name: "OrderItem".to_string(),
            parent_set_name: Some("Orders".to_string()),
            fields: vec![FieldConfig {
                name: "ItemID".to_string(),
                label: "Pos".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None, precision: None, scale: None,
                immutable: false, semantic_object: None,
            }],
            navigation_properties: vec![],
            annotations: None,
            tile: None,
        };
        let (config_records, _, _, _, _) = generate_meta_data(&[config]);
        assert_eq!(config_records[0]["ParentSetName"], "Orders");
    }

    #[test]
    fn meta_semantic_object_on_field() {
        let config = EntityConfig {
            set_name: "Contacts".to_string(),
            key_field: "ContactID".to_string(),
            type_name: "Contact".to_string(),
            parent_set_name: None,
            fields: vec![FieldConfig {
                name: "CustomerID".to_string(),
                label: "Kunde".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None, precision: None, scale: None,
                immutable: false,
                semantic_object: Some("Customers".to_string()),
            }],
            navigation_properties: vec![],
            annotations: None,
            tile: None,
        };
        let (_, field_records, _, _, _) = generate_meta_data(&[config]);
        assert_eq!(field_records[0]["SemanticObject"], "Customers");
    }

    // ── Roundtrip: generate_meta_data → publish_entity_config ───

    #[test]
    fn meta_roundtrip_publish_reconstructs_config() {
        let original = test_config();
        let mock_store = MockDataStore::from_meta(&[original]);

        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_roundtrip");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let result = publish_entity_config(&tmp_dir, "Products", &mock_store);
        assert!(result.is_ok());

        // Read back the written file and deserialize as EntityConfig
        let written = std::fs::read_to_string(tmp_dir.join("Products.json")).unwrap();
        let reconstructed: EntityConfig = serde_json::from_str(&written).unwrap();

        assert_eq!(reconstructed.set_name, "Products");
        assert_eq!(reconstructed.key_field, "ProductID");
        assert_eq!(reconstructed.type_name, "Product");
        assert_eq!(reconstructed.fields.len(), 3);
        assert_eq!(reconstructed.fields[0].name, "ProductID");
        assert_eq!(reconstructed.fields[0].max_length, Some(10));
        assert!(reconstructed.fields[0].immutable);
        assert_eq!(reconstructed.fields[2].name, "Price");
        assert_eq!(reconstructed.fields[2].edm_type, "Edm.Decimal");
        assert_eq!(reconstructed.fields[2].precision, Some(15));
        assert_eq!(reconstructed.fields[2].scale, Some(2));
        assert_eq!(reconstructed.navigation_properties.len(), 1);
        assert_eq!(reconstructed.navigation_properties[0].name, "Supplier");
        assert_eq!(reconstructed.navigation_properties[0].foreign_key,
            Some("SupplierID".to_string()));

        let ann = reconstructed.annotations.unwrap();
        assert_eq!(ann.selection_fields, vec!["ProductName"]);
        assert_eq!(ann.line_item.len(), 2);
        assert_eq!(ann.line_item[0].name, "ProductID");
        assert_eq!(ann.line_item[0].importance, Some("High".to_string()));
        assert_eq!(ann.header_info.type_name, "Produkt");
        assert_eq!(ann.header_info.type_name_plural, "Produkte");
        assert_eq!(ann.facet_sections.len(), 1);
        assert_eq!(ann.facet_sections[0].id, "General");
        assert_eq!(ann.field_groups.len(), 1);
        assert_eq!(ann.field_groups[0].fields, vec!["ProductID", "ProductName", "Price"]);

        let tile = reconstructed.tile.unwrap();
        assert_eq!(tile.title, "Produkte");
        assert_eq!(tile.description, Some("Produktkatalog".to_string()));
        assert_eq!(tile.icon, Some("sap-icon://product".to_string()));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn meta_roundtrip_with_table_facets() {
        let original = config_with_table_facets();
        let mock_store = MockDataStore::from_meta(&[original]);

        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_table_facets");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let result = publish_entity_config(&tmp_dir, "Orders", &mock_store);
        assert!(result.is_ok());

        let written = std::fs::read_to_string(tmp_dir.join("Orders.json")).unwrap();
        let reconstructed: EntityConfig = serde_json::from_str(&written).unwrap();

        assert_eq!(reconstructed.navigation_properties.len(), 1);
        assert!(reconstructed.navigation_properties[0].is_collection);
        let ann = reconstructed.annotations.unwrap();
        assert_eq!(ann.table_facets.len(), 1);
        assert_eq!(ann.table_facets[0].navigation_property, "Items");
        assert_eq!(ann.table_facets[0].label, "Positionen");
        assert!(reconstructed.tile.is_none());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn meta_publish_not_found() {
        let mock_store = MockDataStore::from_meta(&[test_config()]);
        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_not_found");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let result = publish_entity_config(&tmp_dir, "NonExistent", &mock_store);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nicht gefunden"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn meta_publish_preserves_nav_path_line_items() {
        // Write an "original" config with navigation-path line items
        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_nav_li");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        // Simulate existing config file with nav-path line item
        let original_json = json!({
            "set_name": "Products",
            "key_field": "ProductID",
            "type_name": "Product",
            "fields": [],
            "annotations": {
                "selection_fields": [],
                "line_item": [
                    {"name": "ProductID"},
                    {"name": "Supplier/SupplierName", "label": "Lieferant"}
                ],
                "header_info": {
                    "type_name": "Produkt",
                    "type_name_plural": "Produkte",
                    "title_path": "ProductName",
                    "description_path": "ProductID"
                },
                "header_facets": [],
                "data_points": [],
                "facet_sections": [],
                "field_groups": [],
                "table_facets": []
            }
        });
        std::fs::write(
            tmp_dir.join("Products.json"),
            serde_json::to_string_pretty(&original_json).unwrap(),
        ).unwrap();

        let mock_store = MockDataStore::from_meta(&[test_config()]);
        let result = publish_entity_config(&tmp_dir, "Products", &mock_store);
        assert!(result.is_ok());

        let written = std::fs::read_to_string(tmp_dir.join("Products.json")).unwrap();
        let reconstructed: EntityConfig = serde_json::from_str(&written).unwrap();
        let ann = reconstructed.annotations.unwrap();

        // Should have the original field-based line items + the nav-path one
        let nav_li: Vec<&LineItemConfig> = ann.line_item.iter()
            .filter(|li| li.name.contains('/'))
            .collect();
        assert_eq!(nav_li.len(), 1);
        assert_eq!(nav_li[0].name, "Supplier/SupplierName");
        assert_eq!(nav_li[0].label, Some("Lieferant".to_string()));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn meta_publish_preserves_header_facets_from_original() {
        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_hf");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        // Write original config with header_facets and data_points
        let original_json = json!({
            "set_name": "Products",
            "key_field": "ProductID",
            "type_name": "Product",
            "fields": [],
            "annotations": {
                "selection_fields": [],
                "line_item": [],
                "header_info": {
                    "type_name": "Produkt",
                    "type_name_plural": "Produkte",
                    "title_path": "ProductName",
                    "description_path": "ProductID"
                },
                "header_facets": [
                    {"data_point_qualifier": "Price", "label": "Preis"}
                ],
                "data_points": [
                    {"qualifier": "Price", "value_path": "Price", "title": "Preis"}
                ],
                "facet_sections": [],
                "field_groups": [],
                "table_facets": []
            }
        });
        std::fs::write(
            tmp_dir.join("Products.json"),
            serde_json::to_string_pretty(&original_json).unwrap(),
        ).unwrap();

        let mock_store = MockDataStore::from_meta(&[test_config()]);
        let result = publish_entity_config(&tmp_dir, "Products", &mock_store);
        assert!(result.is_ok());

        let written = std::fs::read_to_string(tmp_dir.join("Products.json")).unwrap();
        let reconstructed: EntityConfig = serde_json::from_str(&written).unwrap();
        let ann = reconstructed.annotations.unwrap();

        assert_eq!(ann.header_facets.len(), 1);
        assert_eq!(ann.header_facets[0].data_point_qualifier, "Price");
        assert_eq!(ann.data_points.len(), 1);
        assert_eq!(ann.data_points[0].qualifier, "Price");
        assert_eq!(ann.data_points[0].value_path, "Price");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    // ── write_meta_data Tests ───────────────────────────────────

    #[test]
    fn meta_write_creates_json_files() {
        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_write");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        write_meta_data(&tmp_dir, &[test_config()]);

        assert!(tmp_dir.join("EntityConfigs.json").exists());
        assert!(tmp_dir.join("EntityFields.json").exists());
        assert!(tmp_dir.join("EntityFacets.json").exists());
        assert!(tmp_dir.join("EntityNavigations.json").exists());
        assert!(tmp_dir.join("EntityTableFacets.json").exists());

        // Verify content is valid JSON arrays
        let content = std::fs::read_to_string(tmp_dir.join("EntityConfigs.json")).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["SetName"], "Products");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    // ── Full cycle: config → meta → publish → re-parse ──────────

    #[test]
    fn meta_full_cycle_identity() {
        // This test verifies the core guarantee: config → generate_meta_data → publish → deserialize
        // produces an EntityConfig structurally matching the original.
        let original = test_config();
        let mock_store = MockDataStore::from_meta(&[original]);

        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_cycle");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        publish_entity_config(&tmp_dir, "Products", &mock_store).unwrap();

        let written = std::fs::read_to_string(tmp_dir.join("Products.json")).unwrap();
        let reconstructed: EntityConfig = serde_json::from_str(&written).unwrap();

        // Serialize both and compare the JSON values for structural equality
        let original_val = serde_json::to_value(&test_config()).unwrap();
        let reconstructed_val = serde_json::to_value(&reconstructed).unwrap();

        // Compare all top-level and nested fields
        assert_eq!(original_val["set_name"], reconstructed_val["set_name"]);
        assert_eq!(original_val["key_field"], reconstructed_val["key_field"]);
        assert_eq!(original_val["type_name"], reconstructed_val["type_name"]);
        assert_eq!(original_val["fields"], reconstructed_val["fields"]);
        assert_eq!(original_val["navigation_properties"], reconstructed_val["navigation_properties"]);
        assert_eq!(original_val["tile"], reconstructed_val["tile"]);

        let orig_ann = &original_val["annotations"];
        let recon_ann = &reconstructed_val["annotations"];
        assert_eq!(orig_ann["selection_fields"], recon_ann["selection_fields"]);
        assert_eq!(orig_ann["header_info"], recon_ann["header_info"]);
        assert_eq!(orig_ann["line_item"], recon_ann["line_item"]);
        assert_eq!(orig_ann["facet_sections"], recon_ann["facet_sections"]);
        assert_eq!(orig_ann["field_groups"], recon_ann["field_groups"]);
        assert_eq!(orig_ann["table_facets"], recon_ann["table_facets"]);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    // ── Real workspace config roundtrip ─────────────────────────

    #[test]
    fn meta_roundtrip_real_workspace_configs() {
        use super::super::generic::load_raw_configs;

        let config_dir = Path::new("config/entities");
        let raw_configs = load_raw_configs(config_dir);
        assert!(!raw_configs.is_empty(), "Expected configs from workspace");

        let mock_store = MockDataStore::from_meta(&raw_configs);

        let tmp_dir = std::env::temp_dir().join("fiori_test_meta_real");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        for config in &raw_configs {
            let result = publish_entity_config(&tmp_dir, &config.set_name, &mock_store);
            assert!(result.is_ok(), "Failed to publish {}: {:?}", config.set_name, result.err());

            // Verify the written file can be re-parsed
            let path = tmp_dir.join(format!("{}.json", config.set_name));
            let content = std::fs::read_to_string(&path).unwrap();
            let reparsed: EntityConfig = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to reparse {}: {}", config.set_name, e));
            assert_eq!(reparsed.set_name, config.set_name);
            assert_eq!(reparsed.key_field, config.key_field);
            assert_eq!(reparsed.type_name, config.type_name);
            assert_eq!(reparsed.fields.len(), config.fields.len());
            assert_eq!(reparsed.navigation_properties.len(), config.navigation_properties.len());
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
