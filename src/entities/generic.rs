use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use log::info;
use serde::Deserialize;
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

#[derive(Deserialize)]
pub struct EntityConfig {
    pub set_name: String,
    pub key_field: String,
    pub type_name: String,
    #[serde(default)]
    pub parent_set_name: Option<String>,
    pub fields: Vec<FieldConfig>,
    #[serde(default)]
    pub navigation_properties: Vec<NavPropertyConfig>,
    #[serde(default)]
    pub annotations: Option<AnnotationsConfig>,
    /// Kachel-Konfiguration fuer das FLP.
    #[serde(default)]
    pub tile: Option<TileConfig>,
}

#[derive(Deserialize, Clone)]
pub struct TileConfig {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    #[serde(default = "default_edm_string")]
    pub edm_type: String,
    #[serde(default)]
    pub max_length: Option<u32>,
    #[serde(default)]
    pub precision: Option<u32>,
    #[serde(default)]
    pub scale: Option<u32>,
    #[serde(default)]
    pub immutable: bool,
    #[serde(default)]
    pub semantic_object: Option<String>,
}

fn default_edm_string() -> String {
    "Edm.String".to_string()
}

#[derive(Deserialize, Clone)]
pub struct NavPropertyConfig {
    pub name: String,
    pub target_type: String,
    pub target_set: String,
    #[serde(default)]
    pub is_collection: bool,
    /// Verknuepfungsfeld fuer $expand.
    /// 1:1 → Feld auf dieser Entitaet, das den Key der Ziel-Entitaet enthaelt.
    /// 1:n → Feld auf der Ziel-Entitaet, das den eigenen Key referenziert.
    #[serde(default)]
    pub foreign_key: Option<String>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct LineItemConfig {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub importance: Option<String>,
    #[serde(default)]
    pub criticality_path: Option<String>,
    #[serde(default)]
    pub navigation_path: Option<String>,
    #[serde(default)]
    pub semantic_object: Option<String>,
}

#[derive(Deserialize)]
pub struct HeaderInfoConfig {
    pub type_name: String,
    pub type_name_plural: String,
    pub title_path: String,
    pub description_path: String,
}

#[derive(Deserialize)]
pub struct HeaderFacetConfig {
    pub data_point_qualifier: String,
    pub label: String,
}

#[derive(Deserialize)]
pub struct DataPointConfig {
    pub qualifier: String,
    pub value_path: String,
    pub title: String,
    #[serde(default)]
    pub max_value: Option<u32>,
    #[serde(default)]
    pub visualization: Option<String>,
}

#[derive(Deserialize)]
pub struct FacetSectionConfig {
    pub label: String,
    pub id: String,
    pub field_group_qualifier: String,
    pub field_group_label: String,
}

#[derive(Deserialize)]
pub struct FieldGroupConfig {
    pub qualifier: String,
    pub fields: Vec<String>,
}

#[derive(Deserialize)]
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
