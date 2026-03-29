use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

// ── Helpers: Owned → &'static (fuer Programm-Lebensdauer) ──────────────

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn leak_opt(s: &Option<String>) -> Option<&'static str> {
    s.as_ref().map(|v| leak_str(v))
}

fn leak_vec<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

fn leak_strs(v: &[String]) -> &'static [&'static str] {
    leak_vec(v.iter().map(|s| leak_str(s)).collect())
}

// ── JSON Config Schema ──────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
pub struct EntityConfig {
    pub set_name: String,
    pub key_field: String,
    pub type_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_set_name: Option<String>,
    pub fields: Vec<FieldConfig>,
    #[serde(default)]
    pub navigation_properties: Vec<NavPropertyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<AnnotationsConfig>,
    /// Kachel-Konfiguration fuer das FLP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile: Option<TileConfig>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TileConfig {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    #[serde(default = "default_edm_string")]
    pub edm_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub immutable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_object: Option<String>,
}

fn default_edm_string() -> String {
    "Edm.String".to_string()
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NavPropertyConfig {
    pub name: String,
    pub target_type: String,
    pub target_set: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_collection: bool,
    /// Verknuepfungsfeld fuer $expand.
    /// 1:1 → Feld auf dieser Entitaet, das den Key der Ziel-Entitaet enthaelt.
    /// 1:n → Feld auf der Ziel-Entitaet, das den eigenen Key referenziert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreign_key: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct AnnotationsConfig {
    #[serde(default)]
    pub selection_fields: Vec<String>,
    #[serde(default)]
    pub line_item: Vec<LineItemConfig>,
    pub header_info: HeaderInfoConfig,
    #[serde(default)]
    pub header_facets: Vec<HeaderFacetConfig>,
    #[serde(default)]
    pub data_points: Vec<DataPointConfig>,
    #[serde(default)]
    pub facet_sections: Vec<FacetSectionConfig>,
    #[serde(default)]
    pub field_groups: Vec<FieldGroupConfig>,
    #[serde(default)]
    pub table_facets: Vec<TableFacetConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct LineItemConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_object: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct HeaderInfoConfig {
    pub type_name: String,
    pub type_name_plural: String,
    pub title_path: String,
    pub description_path: String,
}

#[derive(Deserialize, Serialize)]
pub struct HeaderFacetConfig {
    pub data_point_qualifier: String,
    pub label: String,
}

#[derive(Deserialize, Serialize)]
pub struct DataPointConfig {
    pub qualifier: String,
    pub value_path: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_value: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visualization: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct FacetSectionConfig {
    pub label: String,
    pub id: String,
    pub field_group_qualifier: String,
    pub field_group_label: String,
}

#[derive(Deserialize, Serialize)]
pub struct FieldGroupConfig {
    pub qualifier: String,
    pub fields: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct TableFacetConfig {
    pub label: String,
    pub id: String,
    pub navigation_property: String,
}

// ── Konvertierung Config → static Annotation-Structs ────────────────────

fn convert_field(f: &FieldConfig) -> FieldDef {
    FieldDef {
        name: leak_str(&f.name),
        label: leak_str(&f.label),
        edm_type: leak_str(&f.edm_type),
        max_length: f.max_length,
        precision: f.precision,
        scale: f.scale,
        immutable: f.immutable,
        semantic_object: leak_opt(&f.semantic_object),
    }
}

fn convert_nav_property(n: &NavPropertyConfig) -> NavigationPropertyDef {
    NavigationPropertyDef {
        name: leak_str(&n.name),
        target_type: leak_str(&n.target_type),
        is_collection: n.is_collection,
    }
}

fn convert_line_item(c: &LineItemConfig) -> LineItemField {
    LineItemField {
        name: leak_str(&c.name),
        label: leak_opt(&c.label),
        importance: leak_opt(&c.importance),
        criticality_path: leak_opt(&c.criticality_path),
        navigation_path: leak_opt(&c.navigation_path),
        semantic_object: leak_opt(&c.semantic_object),
    }
}

fn convert_annotations(c: &AnnotationsConfig) -> &'static AnnotationsDef {
    let def = AnnotationsDef {
        selection_fields: leak_strs(&c.selection_fields),
        line_item: leak_vec(c.line_item.iter().map(convert_line_item).collect()),
        header_info: HeaderInfoDef {
            type_name: leak_str(&c.header_info.type_name),
            type_name_plural: leak_str(&c.header_info.type_name_plural),
            title_path: leak_str(&c.header_info.title_path),
            description_path: leak_str(&c.header_info.description_path),
        },
        header_facets: leak_vec(
            c.header_facets
                .iter()
                .map(|h| HeaderFacetDef {
                    data_point_qualifier: leak_str(&h.data_point_qualifier),
                    label: leak_str(&h.label),
                })
                .collect(),
        ),
        data_points: leak_vec(
            c.data_points
                .iter()
                .map(|d| DataPointDef {
                    qualifier: leak_str(&d.qualifier),
                    value_path: leak_str(&d.value_path),
                    title: leak_str(&d.title),
                    max_value: d.max_value,
                    visualization: leak_opt(&d.visualization),
                })
                .collect(),
        ),
        facet_sections: leak_vec(
            c.facet_sections
                .iter()
                .map(|f| FacetSectionDef {
                    label: leak_str(&f.label),
                    id: leak_str(&f.id),
                    field_group_qualifier: leak_str(&f.field_group_qualifier),
                    field_group_label: leak_str(&f.field_group_label),
                })
                .collect(),
        ),
        field_groups: leak_vec(
            c.field_groups
                .iter()
                .map(|g| FieldGroupDef {
                    qualifier: leak_str(&g.qualifier),
                    fields: leak_strs(&g.fields),
                })
                .collect(),
        ),
        table_facets: leak_vec(
            c.table_facets
                .iter()
                .map(|t| TableFacetDef {
                    label: leak_str(&t.label),
                    id: leak_str(&t.id),
                    navigation_property: leak_str(&t.navigation_property),
                })
                .collect(),
        ),
    };
    Box::leak(Box::new(def))
}

// ── GenericEntity ───────────────────────────────────────────────────────

pub struct GenericEntity {
    set_name: &'static str,
    key_field: &'static str,
    type_name: &'static str,
    parent_set_name: Option<&'static str>,
    fields: &'static [FieldDef],
    nav_properties: &'static [NavigationPropertyDef],
    nav_configs: Vec<NavPropertyConfig>,
    annotations: Option<&'static AnnotationsDef>,
    entity_set_xml: String,
    tile: Option<TileConfig>,
}

impl fmt::Debug for GenericEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenericEntity")
            .field("set_name", &self.set_name)
            .field("type_name", &self.type_name)
            .finish()
    }
}

impl GenericEntity {
    pub fn from_config(config: EntityConfig) -> Self {
        let set_name = leak_str(&config.set_name);
        let type_name = leak_str(&config.type_name);

        // EntitySet-XML vorberechnen
        let mut xml = format!(
            "<EntitySet Name=\"{}\" EntityType=\"{}.{}\">",
            set_name, NAMESPACE, type_name
        );
        for nav in &config.navigation_properties {
            xml.push_str(&format!(
                "\n<NavigationPropertyBinding Path=\"{}\" Target=\"{}\"/>",
                nav.name, nav.target_set
            ));
        }
        xml.push_str(&format!(
            "\n<NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"{}\"/>",
            set_name
        ));
        xml.push_str(
            "\n<NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>"
        );
        xml.push_str("\n</EntitySet>");

        GenericEntity {
            set_name,
            key_field: leak_str(&config.key_field),
            type_name,
            parent_set_name: leak_opt(&config.parent_set_name),
            fields: leak_vec(config.fields.iter().map(convert_field).collect()),
            nav_properties: leak_vec(
                config
                    .navigation_properties
                    .iter()
                    .map(convert_nav_property)
                    .collect(),
            ),
            nav_configs: config.navigation_properties,
            annotations: config.annotations.as_ref().map(convert_annotations),
            entity_set_xml: xml,
            tile: config.tile,
        }
    }
}

impl ODataEntity for GenericEntity {
    fn set_name(&self) -> &'static str {
        self.set_name
    }
    fn key_field(&self) -> &'static str {
        self.key_field
    }
    fn type_name(&self) -> &'static str {
        self.type_name
    }
    fn parent_set_name(&self) -> Option<&'static str> {
        self.parent_set_name
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        Some(self.fields)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        self.nav_properties
    }

    fn entity_set(&self) -> String {
        self.entity_set_xml.clone()
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        self.annotations
    }

    fn apps_json_entry(&self) -> Option<(String, Value)> {
        let tile = self.tile.as_ref()?;
        let key = format!("{}-display", self.set_name);
        let mut entry = serde_json::json!({
            "title": tile.title,
            "semanticObject": self.set_name,
            "action": "display"
        });
        if let Some(desc) = &tile.description {
            entry["description"] = Value::String(desc.clone());
        }
        if let Some(icon) = &tile.icon {
            entry["icon"] = Value::String(icon.clone());
        }
        Some((key, entry))
    }

    fn expand_record(
        &self,
        record: &mut Value,
        nav_properties: &[&str],
        entities: &[&dyn ODataEntity],
        data_store: &HashMap<String, Vec<Value>>,
    ) {
        for nav in &self.nav_configs {
            if !nav_properties.contains(&nav.name.as_str()) {
                continue;
            }
            let target = match entities.iter().find(|e| e.set_name() == nav.target_set) {
                Some(t) => t,
                None => continue,
            };
            let data = data_store
                .get(target.set_name())
                .cloned()
                .unwrap_or_else(|| target.mock_data());

            if nav.is_collection {
                // 1:n – foreign_key auf dem Kind verweist auf unseren Key
                let fk = nav
                    .foreign_key
                    .as_deref()
                    .unwrap_or(self.key_field);
                let key_val = record
                    .get(self.key_field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(kv) = key_val {
                    let children: Vec<Value> = data
                        .into_iter()
                        .filter(|r| r.get(fk).and_then(|v| v.as_str()) == Some(&kv))
                        .collect();
                    if let Some(obj) = record.as_object_mut() {
                        obj.insert(nav.name.clone(), Value::Array(children));
                    }
                }
            } else {
                // 1:1 – foreign_key ist das Feld auf diesem Record, das den Ziel-Key enthaelt
                let fk = nav
                    .foreign_key
                    .as_deref()
                    .unwrap_or(target.key_field());
                let fk_val = record
                    .get(fk)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(fkv) = fk_val {
                    let target_key = target.key_field();
                    let found = data
                        .into_iter()
                        .find(|r| r.get(target_key).and_then(|v| v.as_str()) == Some(&fkv));
                    if let Some(obj) = record.as_object_mut() {
                        obj.insert(nav.name.clone(), found.unwrap_or(Value::Null));
                    }
                }
            }
        }
    }
}

// ── Laden aus Verzeichnis ───────────────────────────────────────────────

/// Laedt alle Entity-Konfigurationen aus einem Verzeichnis als rohe Structs.
pub fn load_raw_configs(config_dir: &Path) -> Vec<EntityConfig> {
    let mut configs: Vec<EntityConfig> = Vec::new();

    if !config_dir.is_dir() {
        info!(
            "Kein Entity-Config-Verzeichnis: {} – uebersprungen",
            config_dir.display()
        );
        return configs;
    }

    let mut entries: Vec<_> = match std::fs::read_dir(config_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!(
                "  WARNUNG: Konnte {} nicht lesen: {}",
                config_dir.display(),
                e
            );
            return configs;
        }
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<EntityConfig>(&content) {
                Ok(config) => {
                    info!(
                        "  Generische Entitaet geladen: {} ({})",
                        config.set_name, config.type_name
                    );
                    configs.push(config);
                }
                Err(e) => {
                    eprintln!(
                        "  WARNUNG: Konnte {} nicht parsen: {}",
                        path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "  WARNUNG: Konnte {} nicht lesen: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    configs
}

/// Wandelt rohe EntityConfigs in registrierbare ODataEntity-Instanzen um.
pub fn create_generic_entities(configs: Vec<EntityConfig>) -> Vec<&'static dyn ODataEntity> {
    configs
        .into_iter()
        .map(|config| {
            let entity = GenericEntity::from_config(config);
            let leaked: &'static GenericEntity = Box::leak(Box::new(entity));
            leaked as &'static dyn ODataEntity
        })
        .collect()
}

/// Laedt und erzeugt generische Entitaeten in einem Schritt (Bequemlichkeit).
pub fn load_generic_entities(config_dir: &Path) -> Vec<&'static dyn ODataEntity> {
    create_generic_entities(load_raw_configs(config_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Helper: minimal EntityConfig ────────────────────────────

    fn minimal_config() -> EntityConfig {
        EntityConfig {
            set_name: "TestItems".to_string(),
            key_field: "ItemID".to_string(),
            type_name: "TestItem".to_string(),
            parent_set_name: None,
            fields: vec![
                FieldConfig {
                    name: "ItemID".to_string(),
                    label: "Item Nr.".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    semantic_object: None,
                },
                FieldConfig {
                    name: "Name".to_string(),
                    label: "Name".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(80),
                    precision: None,
                    scale: None,
                    immutable: false,
                    semantic_object: None,
                },
            ],
            navigation_properties: vec![],
            annotations: None,
            tile: None,
        }
    }

    fn full_config() -> EntityConfig {
        EntityConfig {
            set_name: "Orders".to_string(),
            key_field: "OrderID".to_string(),
            type_name: "Order".to_string(),
            parent_set_name: None,
            fields: vec![
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
                FieldConfig {
                    name: "Status".to_string(),
                    label: "Status".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(1),
                    precision: None,
                    scale: None,
                    immutable: false,
                    semantic_object: None,
                },
                FieldConfig {
                    name: "Amount".to_string(),
                    label: "Betrag".to_string(),
                    edm_type: "Edm.Decimal".to_string(),
                    max_length: None,
                    precision: Some(15),
                    scale: Some(2),
                    immutable: false,
                    semantic_object: None,
                },
            ],
            navigation_properties: vec![
                NavPropertyConfig {
                    name: "Items".to_string(),
                    target_type: "OrderItem".to_string(),
                    target_set: "OrderItems".to_string(),
                    is_collection: true,
                    foreign_key: Some("OrderID".to_string()),
                },
            ],
            annotations: Some(AnnotationsConfig {
                selection_fields: vec!["Status".to_string()],
                line_item: vec![
                    LineItemConfig {
                        name: "OrderID".to_string(),
                        label: None,
                        importance: Some("High".to_string()),
                        criticality_path: None,
                        navigation_path: None,
                        semantic_object: None,
                    },
                    LineItemConfig {
                        name: "Status".to_string(),
                        label: None,
                        importance: None,
                        criticality_path: Some("StatusCriticality".to_string()),
                        navigation_path: None,
                        semantic_object: None,
                    },
                ],
                header_info: HeaderInfoConfig {
                    type_name: "Auftrag".to_string(),
                    type_name_plural: "Auftraege".to_string(),
                    title_path: "OrderID".to_string(),
                    description_path: "Status".to_string(),
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
                    fields: vec!["OrderID".to_string(), "Status".to_string(), "Amount".to_string()],
                }],
                table_facets: vec![TableFacetConfig {
                    label: "Positionen".to_string(),
                    id: "ItemsFacet".to_string(),
                    navigation_property: "Items".to_string(),
                }],
            }),
            tile: Some(TileConfig {
                title: "Auftraege".to_string(),
                description: Some("Auftragsübersicht".to_string()),
                icon: Some("sap-icon://sales-order".to_string()),
            }),
        }
    }

    fn child_config() -> EntityConfig {
        EntityConfig {
            set_name: "OrderItems".to_string(),
            key_field: "ItemID".to_string(),
            type_name: "OrderItem".to_string(),
            parent_set_name: Some("Orders".to_string()),
            fields: vec![
                FieldConfig {
                    name: "ItemID".to_string(),
                    label: "Pos-Nr.".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    semantic_object: None,
                },
                FieldConfig {
                    name: "OrderID".to_string(),
                    label: "Auftragsnr.".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    semantic_object: Some("Orders".to_string()),
                },
            ],
            navigation_properties: vec![],
            annotations: Some(AnnotationsConfig {
                selection_fields: vec![],
                line_item: vec![
                    LineItemConfig {
                        name: "ItemID".to_string(),
                        label: None,
                        importance: None,
                        criticality_path: None,
                        navigation_path: None,
                        semantic_object: None,
                    },
                ],
                header_info: HeaderInfoConfig {
                    type_name: "Position".to_string(),
                    type_name_plural: "Positionen".to_string(),
                    title_path: "ItemID".to_string(),
                    description_path: "OrderID".to_string(),
                },
                header_facets: vec![],
                data_points: vec![],
                facet_sections: vec![],
                field_groups: vec![],
                table_facets: vec![],
            }),
            tile: None,
        }
    }

    // ── Serde Roundtrip Tests ───────────────────────────────────

    #[test]
    fn config_serde_roundtrip_minimal() {
        let config = minimal_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EntityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.set_name, "TestItems");
        assert_eq!(parsed.key_field, "ItemID");
        assert_eq!(parsed.fields.len(), 2);
        assert!(parsed.annotations.is_none());
        assert!(parsed.tile.is_none());
        assert!(parsed.parent_set_name.is_none());
    }

    #[test]
    fn config_serde_roundtrip_full() {
        let config = full_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EntityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.set_name, "Orders");
        assert_eq!(parsed.navigation_properties.len(), 1);
        assert_eq!(parsed.navigation_properties[0].name, "Items");
        assert!(parsed.navigation_properties[0].is_collection);
        let ann = parsed.annotations.as_ref().unwrap();
        assert_eq!(ann.selection_fields, vec!["Status"]);
        assert_eq!(ann.line_item.len(), 2);
        assert_eq!(ann.header_info.type_name, "Auftrag");
        assert_eq!(ann.facet_sections.len(), 1);
        assert_eq!(ann.field_groups.len(), 1);
        assert_eq!(ann.table_facets.len(), 1);
        let tile = parsed.tile.as_ref().unwrap();
        assert_eq!(tile.title, "Auftraege");
        assert!(tile.description.is_some());
        assert!(tile.icon.is_some());
    }

    #[test]
    fn config_serde_skip_serializing_defaults() {
        let config = minimal_config();
        let val: Value = serde_json::to_value(&config).unwrap();
        // Optional None fields should not appear
        assert!(val.get("parent_set_name").is_none());
        assert!(val.get("annotations").is_none());
        assert!(val.get("tile").is_none());
        // immutable=false should not appear on field
        let field0 = &val["fields"][0];
        assert!(field0.get("immutable").is_none() ||
            field0.get("immutable") == Some(&json!(true))); // only appears when true
    }

    #[test]
    fn config_serde_child_with_parent() {
        let config = child_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EntityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.parent_set_name.as_deref(), Some("Orders"));
        assert_eq!(parsed.fields[1].semantic_object.as_deref(), Some("Orders"));
    }

    #[test]
    fn config_serde_immutable_roundtrip() {
        let config = minimal_config();
        let val: Value = serde_json::to_value(&config).unwrap();
        // First field: immutable=true → must be present
        assert_eq!(val["fields"][0]["immutable"], json!(true));
        // Second field: immutable=false → must NOT be present
        assert!(val["fields"][1].get("immutable").is_none());
    }

    #[test]
    fn config_serde_decimal_field_roundtrip() {
        let config = full_config();
        let val: Value = serde_json::to_value(&config).unwrap();
        let amount_field = &val["fields"][2];
        assert_eq!(amount_field["edm_type"], "Edm.Decimal");
        assert_eq!(amount_field["precision"], 15);
        assert_eq!(amount_field["scale"], 2);
        assert!(amount_field.get("max_length").is_none());
    }

    #[test]
    fn config_deserialize_from_existing_json() {
        // Parse a real config file from the workspace
        let content = std::fs::read_to_string("config/entities/Customers.json")
            .expect("Customers.json should exist");
        let config: EntityConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.set_name, "Customers");
        assert_eq!(config.key_field, "CustomerID");
        assert!(!config.fields.is_empty());
        assert!(config.annotations.is_some());
        let ann = config.annotations.as_ref().unwrap();
        assert!(!ann.line_item.is_empty());
        assert!(!ann.facet_sections.is_empty());
    }

    #[test]
    fn config_deserialize_serialize_preserves_real_file() {
        // Roundtrip: parse real file → serialize → parse again → compare key fields
        let content = std::fs::read_to_string("config/entities/Contacts.json")
            .expect("Contacts.json should exist");
        let config: EntityConfig = serde_json::from_str(&content).unwrap();
        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let reparsed: EntityConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.set_name, reparsed.set_name);
        assert_eq!(config.key_field, reparsed.key_field);
        assert_eq!(config.fields.len(), reparsed.fields.len());
        assert_eq!(
            config.navigation_properties.len(),
            reparsed.navigation_properties.len()
        );
        let ann1 = config.annotations.as_ref().unwrap();
        let ann2 = reparsed.annotations.as_ref().unwrap();
        assert_eq!(ann1.line_item.len(), ann2.line_item.len());
        assert_eq!(ann1.facet_sections.len(), ann2.facet_sections.len());
        assert_eq!(ann1.field_groups.len(), ann2.field_groups.len());
    }

    // ── GenericEntity / ODataEntity Tests ───────────────────────

    #[test]
    fn generic_entity_basic_properties() {
        let entity = GenericEntity::from_config(minimal_config());
        assert_eq!(entity.set_name(), "TestItems");
        assert_eq!(entity.key_field(), "ItemID");
        assert_eq!(entity.type_name(), "TestItem");
        assert!(entity.parent_set_name().is_none());
        assert_eq!(entity.mock_data().len(), 0);
    }

    #[test]
    fn generic_entity_fields_def() {
        let entity = GenericEntity::from_config(minimal_config());
        let fields = entity.fields_def().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "ItemID");
        assert_eq!(fields[0].label, "Item Nr.");
        assert!(fields[0].immutable);
        assert_eq!(fields[0].max_length, Some(10));
        assert_eq!(fields[1].name, "Name");
        assert!(!fields[1].immutable);
    }

    #[test]
    fn generic_entity_annotations() {
        let entity = GenericEntity::from_config(full_config());
        let ann = entity.annotations_def().unwrap();
        assert_eq!(ann.selection_fields, &["Status"]);
        assert_eq!(ann.line_item.len(), 2);
        assert_eq!(ann.line_item[0].name, "OrderID");
        assert_eq!(ann.line_item[0].importance, Some("High"));
        assert_eq!(ann.header_info.type_name, "Auftrag");
        assert_eq!(ann.header_info.type_name_plural, "Auftraege");
        assert_eq!(ann.facet_sections.len(), 1);
        assert_eq!(ann.facet_sections[0].id, "General");
        assert_eq!(ann.table_facets.len(), 1);
        assert_eq!(ann.table_facets[0].navigation_property, "Items");
    }

    #[test]
    fn generic_entity_navigation_properties() {
        let entity = GenericEntity::from_config(full_config());
        let navs = entity.navigation_properties();
        assert_eq!(navs.len(), 1);
        assert_eq!(navs[0].name, "Items");
        assert_eq!(navs[0].target_type, "OrderItem");
        assert!(navs[0].is_collection);
    }

    #[test]
    fn generic_entity_parent_set_name() {
        let entity = GenericEntity::from_config(child_config());
        assert_eq!(entity.parent_set_name(), Some("Orders"));
    }

    #[test]
    fn generic_entity_entity_set_xml() {
        let entity = GenericEntity::from_config(full_config());
        let xml = entity.entity_set();
        assert!(xml.contains("EntitySet Name=\"Orders\""));
        assert!(xml.contains("EntityType=\"ProductsService.Order\""));
        assert!(xml.contains("Path=\"Items\" Target=\"OrderItems\""));
        assert!(xml.contains("SiblingEntity"));
        assert!(xml.contains("DraftAdministrativeData"));
    }

    #[test]
    fn generic_entity_entity_set_xml_no_nav() {
        let entity = GenericEntity::from_config(minimal_config());
        let xml = entity.entity_set();
        assert!(xml.contains("EntitySet Name=\"TestItems\""));
        // Only SiblingEntity + DraftAdministrativeData bindings
        assert!(xml.contains("SiblingEntity"));
        assert!(xml.contains("DraftAdministrativeData"));
    }

    #[test]
    fn generic_entity_apps_json_with_tile() {
        let entity = GenericEntity::from_config(full_config());
        let (key, entry) = entity.apps_json_entry().unwrap();
        assert_eq!(key, "Orders-display");
        assert_eq!(entry["title"], "Auftraege");
        assert_eq!(entry["semanticObject"], "Orders");
        assert_eq!(entry["action"], "display");
        assert_eq!(entry["description"], "Auftragsübersicht");
        assert_eq!(entry["icon"], "sap-icon://sales-order");
    }

    #[test]
    fn generic_entity_apps_json_without_tile() {
        let entity = GenericEntity::from_config(minimal_config());
        assert!(entity.apps_json_entry().is_none());
    }

    #[test]
    fn generic_entity_expand_1n() {
        let order_entity = GenericEntity::from_config(full_config());
        let child_entity = GenericEntity::from_config(child_config());
        let entities: Vec<&dyn ODataEntity> =
            vec![&order_entity as &dyn ODataEntity, &child_entity as &dyn ODataEntity];

        let mut store: HashMap<String, Vec<Value>> = HashMap::new();
        store.insert("OrderItems".to_string(), vec![
            json!({"ItemID": "I001", "OrderID": "O001"}),
            json!({"ItemID": "I002", "OrderID": "O001"}),
            json!({"ItemID": "I003", "OrderID": "O002"}),
        ]);

        let mut record = json!({"OrderID": "O001", "Status": "A"});
        order_entity.expand_record(&mut record, &["Items"], &entities, &store);

        let items = record["Items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["ItemID"], "I001");
        assert_eq!(items[1]["ItemID"], "I002");
    }

    #[test]
    fn generic_entity_expand_1_1() {
        let contact_config = EntityConfig {
            set_name: "Contacts".to_string(),
            key_field: "ContactID".to_string(),
            type_name: "Contact".to_string(),
            parent_set_name: None,
            fields: vec![
                FieldConfig {
                    name: "ContactID".to_string(),
                    label: "ID".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: None, precision: None, scale: None,
                    immutable: true, semantic_object: None,
                },
                FieldConfig {
                    name: "CustomerID".to_string(),
                    label: "Kunde".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: None, precision: None, scale: None,
                    immutable: false, semantic_object: None,
                },
            ],
            navigation_properties: vec![NavPropertyConfig {
                name: "Customer".to_string(),
                target_type: "Customer".to_string(),
                target_set: "Customers".to_string(),
                is_collection: false,
                foreign_key: Some("CustomerID".to_string()),
            }],
            annotations: None,
            tile: None,
        };
        let customer_config = EntityConfig {
            set_name: "Customers".to_string(),
            key_field: "CustomerID".to_string(),
            type_name: "Customer".to_string(),
            parent_set_name: None,
            fields: vec![FieldConfig {
                name: "CustomerID".to_string(),
                label: "ID".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None, precision: None, scale: None,
                immutable: true, semantic_object: None,
            }],
            navigation_properties: vec![],
            annotations: None,
            tile: None,
        };

        let contact_entity = GenericEntity::from_config(contact_config);
        let customer_entity = GenericEntity::from_config(customer_config);
        let entities: Vec<&dyn ODataEntity> =
            vec![&contact_entity as &dyn ODataEntity, &customer_entity as &dyn ODataEntity];

        let mut store: HashMap<String, Vec<Value>> = HashMap::new();
        store.insert("Customers".to_string(), vec![
            json!({"CustomerID": "C001", "Name": "Acme"}),
            json!({"CustomerID": "C002", "Name": "Global"}),
        ]);

        let mut record = json!({"ContactID": "K001", "CustomerID": "C002"});
        contact_entity.expand_record(&mut record, &["Customer"], &entities, &store);

        assert_eq!(record["Customer"]["CustomerID"], "C002");
        assert_eq!(record["Customer"]["Name"], "Global");
    }

    #[test]
    fn generic_entity_expand_unknown_nav_ignored() {
        let entity = GenericEntity::from_config(full_config());
        let entities: Vec<&dyn ODataEntity> = vec![&entity as &dyn ODataEntity];
        let store: HashMap<String, Vec<Value>> = HashMap::new();

        let mut record = json!({"OrderID": "O001"});
        entity.expand_record(&mut record, &["NonExistent"], &entities, &store);
        // Record unchanged — no panic
        assert!(record.get("NonExistent").is_none());
    }

    #[test]
    fn generic_entity_debug_impl() {
        let entity = GenericEntity::from_config(minimal_config());
        let dbg = format!("{:?}", entity);
        assert!(dbg.contains("TestItems"));
        assert!(dbg.contains("TestItem"));
    }

    // ── load_raw_configs Tests ──────────────────────────────────

    #[test]
    fn load_raw_configs_from_workspace() {
        let config_dir = Path::new("config/entities");
        let configs = load_raw_configs(config_dir);
        assert!(configs.len() >= 2, "expected at least 2 configs, got {}", configs.len());
        let names: Vec<&str> = configs.iter().map(|c| c.set_name.as_str()).collect();
        assert!(names.contains(&"Customers"));
        assert!(names.contains(&"Contacts"));
    }

    #[test]
    fn load_raw_configs_nonexistent_dir() {
        let configs = load_raw_configs(Path::new("/tmp/nonexistent_dir_12345"));
        assert!(configs.is_empty());
    }

    #[test]
    fn create_generic_entities_preserves_count() {
        let configs = vec![minimal_config(), full_config()];
        let entities = create_generic_entities(configs);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].set_name(), "TestItems");
        assert_eq!(entities[1].set_name(), "Orders");
    }
}
