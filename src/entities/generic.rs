use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::spec::{
    AtomValueList, EntitySpec, FieldSpec, PresentationOverrides, Relationship, Side,
};
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

// ── JSON Config Schema ──────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
pub struct EntityConfig {
    pub set_name: String,
    pub type_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_set_name: Option<String>,
    pub fields: Vec<FieldConfig>,
    #[serde(default)]
    pub navigation_properties: Vec<NavPropertyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<AnnotationsConfig>,
    /// Standardwerte fuer neue Entitaeten (z.B. {"Currency": "EUR", "Status": "A"}).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_values: Option<Value>,
    /// Kachel-Konfiguration fuer das FLP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile: Option<TileConfig>,
    /// Benannte Wertelisten fuer Felder mit festen Werten.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub value_lists: Vec<FieldValueListConfig>,
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub computed: bool,
    /// FK-Referenz auf ein anderes EntitySet (z.B. "Customers").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub references_entity: Option<String>,
    /// Name einer Werteliste (UUID) fuer Fixed-Value-Dropdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_source: Option<String>,
    /// true → Suchdialog statt Dropdown.
    #[serde(default, skip_serializing_if = "is_false")]
    pub prefer_dialog: bool,
    /// Expliziter Textpfad fuer Common.Text (z.B. "Product/ProductName").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_path: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub searchable: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub show_in_list: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_sort_order: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_importance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_criticality_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form_group: Option<String>,
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

/// Definiert eine benannte Werteliste mit festen Eintraegen.
#[derive(Deserialize, Serialize, Clone)]
pub struct FieldValueListConfig {
    pub list_name: String,
    #[serde(default)]
    pub description: String,
    pub entries: Vec<FieldValueListEntry>,
}

/// Ein einzelner Eintrag in einer Werteliste (Code + Beschreibung).
#[derive(Deserialize, Serialize, Clone)]
pub struct FieldValueListEntry {
    pub code: String,
    pub description: String,
}

// ── Konvertierung Config → static Annotation-Structs ────────────────────

fn convert_field(f: &FieldConfig) -> FieldDef {
    // Explicit text_path takes priority; fall back to auto-generated _text for value_source fields.
    let text_path = if let Some(ref tp) = f.text_path {
        Some(leak_str(tp))
    } else if f.value_source.as_ref().map_or(false, |s| !s.is_empty()) {
        Some(leak_str(&format!("_{}_text", f.name)))
    } else {
        None
    };
    FieldDef {
        name: leak_str(&f.name),
        label: leak_str(&f.label),
        edm_type: leak_str(&f.edm_type),
        max_length: f.max_length,
        precision: f.precision,
        scale: f.scale,
        immutable: f.immutable,
        computed: f.computed,
        references_entity: leak_opt(&f.references_entity),
        value_source: leak_opt(&f.value_source),
        prefer_dialog: f.prefer_dialog,
        text_path,
        searchable: f.searchable,
        show_in_list: f.show_in_list,
        list_sort_order: f.list_sort_order,
        list_importance: leak_opt(&f.list_importance),
        list_criticality_path: leak_opt(&f.list_criticality_path),
        form_group: leak_opt(&f.form_group),
    }
}

fn convert_nav_property(n: &NavPropertyConfig) -> NavigationPropertyDef {
    NavigationPropertyDef {
        name: leak_str(&n.name),
        target_type: leak_str(&n.target_type),
        is_collection: n.is_collection,
        foreign_key: n.foreign_key.as_deref().map(|s| leak_str(&s.to_string())),
    }
}

fn convert_annotations(c: &AnnotationsConfig) -> &'static AnnotationsDef {
    let def = AnnotationsDef {
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

// ── EntityConfig → EntitySpec + Relationships ───────────────────────────

/// Build an EntitySpec from EntityConfig fields (excluding FK fields from references_entity).
fn config_to_entity_spec(
    set_name: &str,
    annotations: &Option<AnnotationsConfig>,
    field_configs: &[FieldConfig],
) -> EntitySpec {
    let ann = annotations.as_ref();

    let fields: Vec<FieldSpec> = field_configs
        .iter()
        .filter(|f| f.references_entity.as_ref().map_or(true, |s| s.is_empty()))
        .map(|f| {
            let value_list = f
                .value_source
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(|vs| AtomValueList::FieldValueList {
                    list_id: vs.clone(),
                    prefer_dialog: f.prefer_dialog,
                });

            FieldSpec::Atom {
                name: f.name.clone(),
                label: f.label.clone(),
                edm_type: f.edm_type.clone(),
                package: None,
                max_length: f.max_length,
                precision: f.precision,
                scale: f.scale,
                computed: f.computed,
                immutable: f.immutable,
                value_list,
                presentation: PresentationOverrides {
                    searchable: if f.searchable { Some(true) } else { None },
                    show_in_list: if f.show_in_list { Some(true) } else { None },
                    list_sort_order: f.list_sort_order,
                    list_importance: f.list_importance.clone(),
                    criticality_path: f.list_criticality_path.clone(),
                    form_group: f.form_group.clone(),
                },
            }
        })
        .collect();

    let data_points = ann.map_or_else(Vec::new, |a| {
        a.data_points
            .iter()
            .map(|dp| DataPointDef {
                qualifier: leak_str(&dp.qualifier),
                value_path: leak_str(&dp.value_path),
                title: leak_str(&dp.title),
                max_value: dp.max_value,
                visualization: dp.visualization.as_deref().map(leak_str),
            })
            .collect()
    });

    let header_facets = ann.map_or_else(Vec::new, |a| {
        a.header_facets
            .iter()
            .map(|hf| HeaderFacetDef {
                data_point_qualifier: leak_str(&hf.data_point_qualifier),
                label: leak_str(&hf.label),
            })
            .collect()
    });

    EntitySpec {
        set_name: set_name.to_string(),
        package: None,
        type_name: ann.map(|a| a.header_info.type_name.clone()),
        type_name_plural: ann.map(|a| a.header_info.type_name_plural.clone()),
        title_field: ann.map(|a| a.header_info.title_path.clone()),
        description_field: ann.map(|a| a.header_info.description_path.clone()),
        fields,
        data_points,
        header_facets,
    }
}

/// Extract relationships from an EntityConfig.
///
/// - 1:N navigation properties → composition (if target has parent_set_name) or association
/// - Fields with references_entity → 1:1 association
fn config_to_relationships(
    config: &EntityConfig,
    parent_sets: &HashMap<String, String>,
) -> Vec<Relationship> {
    let mut rels = vec![];

    // 1:N navigation properties → composition or association
    for nav in &config.navigation_properties {
        if !nav.is_collection {
            continue;
        }

        let is_composition = parent_sets
            .get(&nav.target_set)
            .map_or(false, |parent| parent == &config.set_name);

        // Derive hidden many-side nav name from FK field
        let many_nav_name = if let Some(ref fk) = nav.foreign_key {
            if fk.ends_with("ID") && fk.len() > 2 {
                format!("_{}", &fk[..fk.len() - 2])
            } else {
                format!("_{}", config.set_name)
            }
        } else {
            format!("_{}", config.set_name)
        };

        rels.push(Relationship {
            name: format!("{}_{}", config.set_name, nav.target_set),
            one: Side::new(&config.set_name, &nav.name),
            many: Side::new(&nav.target_set, &many_nav_name),
            owned: is_composition,
            fk_field: nav.foreign_key.clone(),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: None,
        });
    }

    // Fields with references_entity → association
    for f in &config.fields {
        let ref_entity = match &f.references_entity {
            Some(re) if !re.is_empty() => re,
            _ => continue,
        };

        // Derive nav name: "CustomerID" → "Customer"
        let nav_name = if f.name.ends_with("ID") && f.name.len() > 2 {
            f.name[..f.name.len() - 2].to_string()
        } else if ref_entity.ends_with('s') {
            ref_entity[..ref_entity.len() - 1].to_string()
        } else {
            ref_entity.clone()
        };

        rels.push(Relationship {
            name: format!("{}_{}", config.set_name, nav_name),
            one: Side::new(ref_entity, &format!("_{}", config.set_name)),
            many: Side::new(&config.set_name, &nav_name),
            owned: false,
            fk_field: Some(f.name.clone()),
            fk_label: Some(f.label.clone()),
            fk_form_group: f.form_group.clone(),
            condition: None,
            package: None,
        });
    }

    rels
}

// ── GenericEntity ───────────────────────────────────────────────────────

pub struct GenericEntity {
    set_name: &'static str,
    type_name: &'static str,
    parent_set_name: Option<&'static str>,
    fields: &'static [FieldDef],
    nav_properties: &'static [NavigationPropertyDef],
    nav_configs: Vec<NavPropertyConfig>,
    annotations: Option<&'static AnnotationsDef>,
    entity_set_xml: String,
    tile: Option<TileConfig>,
    default_vals: Option<Value>,
    spec: EntitySpec,
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
    pub fn from_config(mut config: EntityConfig, title_paths: &HashMap<String, String>) -> Self {
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

        // Auto-derive text_path for FK fields with references_entity.
        // Convention: NavName is derived from FK field name minus "ID" suffix, or uses target set name (singular).
        // text_path = "{NavName}/{target_title_path}"
        let mut field_configs = config.fields;
        for field in field_configs.iter_mut() {
            let ref_entity = match &field.references_entity {
                Some(re) if !re.is_empty() => re.clone(),
                _ => continue,
            };
            if field.text_path.is_some() {
                continue; // explicit override takes priority
            }
            let target_title = match title_paths.get(&ref_entity) {
                Some(tp) => tp.clone(),
                None => continue,
            };
            // Derive nav name from FK field name: "CustomerID" → "Customer"
            let nav_name = if field.name.ends_with("ID") && field.name.len() > 2 {
                field.name[..field.name.len() - 2].to_string()
            } else {
                // Fallback: singular of target set (strip trailing 's')
                let s = &ref_entity;
                if s.ends_with('s') {
                    s[..s.len() - 1].to_string()
                } else {
                    s.clone()
                }
            };
            field.text_path = Some(format!("{}/{}", nav_name, target_title));
        }

        // Apply FieldGroup mapping from annotations config to FieldConfig.form_group.
        // EntityFacets defines qualifier → field list, but FieldConfig.form_group (from EntityFields)
        // is typically empty. Backfill it so build_annotations() can derive UI.FieldGroup.
        if let Some(ref ann) = config.annotations {
            for fg in &ann.field_groups {
                for field_name in &fg.fields {
                    if let Some(fc) = field_configs.iter_mut().find(|f| &f.name == field_name) {
                        if fc.form_group.is_none() {
                            fc.form_group = Some(fg.qualifier.clone());
                        }
                    }
                }
            }
        }

        // Build EntitySpec from the (form_group-backfilled) fields, before FK processing.
        let spec = config_to_entity_spec(&config.set_name, &config.annotations, &field_configs);

        // Convention: key is always ID (Edm.Guid, auto-generated, hidden in UI).
        // Ensure the ID field exists at position 0 with the correct type.
        let mut fields: Vec<FieldDef> = field_configs.iter().map(convert_field).collect();
        if let Some(pos) = fields.iter().position(|f| f.name == "ID") {
            fields[pos].edm_type = "Edm.Guid";
            fields[pos].immutable = true;
            fields[pos].computed = true;
            fields[pos].max_length = None;
        } else {
            fields.insert(
                0,
                FieldDef {
                    name: "ID",
                    label: "ID",
                    edm_type: "Edm.Guid",
                    max_length: None,
                    precision: None,
                    scale: None,
                    immutable: true,
                    computed: true,
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
            );
        }

        // Build nav props: start from config (1:N), then auto-add 1:1 from references_entity.
        let mut nav_props: Vec<NavigationPropertyDef> = config
            .navigation_properties
            .iter()
            .map(convert_nav_property)
            .collect();

        // Auto-generate 1:1 NavigationPropertyDef from fields with references_entity.
        // Convention: nav name derived from FK field name ("CustomerID" → "Customer").
        for f in &fields {
            let ref_entity = match f.references_entity {
                Some(re) if !re.is_empty() => re,
                _ => continue,
            };
            // Derive nav name
            let nav_name = if f.name.ends_with("ID") && f.name.len() > 2 {
                &f.name[..f.name.len() - 2]
            } else {
                let s = ref_entity;
                if s.ends_with('s') {
                    &s[..s.len() - 1]
                } else {
                    s
                }
            };
            // Derive target type name (singular of entity set)
            let target_type = if ref_entity.ends_with('s') {
                &ref_entity[..ref_entity.len() - 1]
            } else {
                ref_entity
            };
            // Only add if not already present (from EntityNavigations)
            if !nav_props.iter().any(|np| np.name == nav_name) {
                nav_props.push(NavigationPropertyDef {
                    name: leak_str(nav_name),
                    target_type: leak_str(target_type),
                    is_collection: false,
                    foreign_key: Some(f.name),
                });
                // Also register in nav_configs so expand_record can resolve 1:1 navs at runtime
                config.navigation_properties.push(NavPropertyConfig {
                    name: nav_name.to_string(),
                    target_type: target_type.to_string(),
                    target_set: ref_entity.to_string(),
                    is_collection: false,
                    foreign_key: Some(f.name.to_string()),
                });
                // Add NavigationPropertyBinding to EntitySet XML
                xml = xml.replace(
                    "\n<NavigationPropertyBinding Path=\"SiblingEntity\"",
                    &format!(
                        "\n<NavigationPropertyBinding Path=\"{}\" Target=\"{}\"/>\n<NavigationPropertyBinding Path=\"SiblingEntity\"",
                        nav_name, ref_entity
                    ),
                );
            }
        }

        // For fields with value_source, add a hidden computed _text field
        // so Common.Text can resolve the description from FieldValueListItems.
        let text_fields: Vec<FieldDef> = fields
            .iter()
            .filter(|f| f.text_path.is_some() && f.value_source.is_some())
            .map(|f| FieldDef {
                name: f.text_path.unwrap(),
                label: f.text_path.unwrap(),
                edm_type: "Edm.String",
                max_length: Some(120),
                precision: None,
                scale: None,
                immutable: false,
                computed: true,
                references_entity: None,
                prefer_dialog: false,
                value_source: None,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            })
            .collect();
        fields.extend(text_fields);

        GenericEntity {
            set_name,
            type_name,
            parent_set_name: leak_opt(&config.parent_set_name),
            fields: leak_vec(fields),
            nav_properties: leak_vec(nav_props),
            nav_configs: config.navigation_properties,
            annotations: config.annotations.as_ref().map(convert_annotations),
            entity_set_xml: xml,
            tile: config.tile,
            default_vals: config.default_values,
            spec,
        }
    }
}

impl ODataEntity for GenericEntity {
    fn set_name(&self) -> &'static str {
        self.set_name
    }
    fn type_name(&self) -> &'static str {
        self.type_name
    }
    fn parent_set_name(&self) -> Option<&'static str> {
        self.parent_set_name
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

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(self.spec.clone())
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
                let fk = nav.foreign_key.as_deref().unwrap_or("ID");
                let key_val = record
                    .get("ID")
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
                let fk = nav.foreign_key.as_deref().unwrap_or(target.key_field());
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

    fn default_values(&self) -> Option<Value> {
        self.default_vals.clone()
    }

    // Child entities (compositions) have no own tiles, routes or targets.
    fn manifest_inbound(&self) -> (String, Value) {
        if self.parent_set_name.is_some() {
            return (format!("_{}-stub", self.set_name), Value::Null);
        }
        // default trait implementation
        let key = format!("{}-display", self.set_name);
        let entry = serde_json::json!({
            "semanticObject": self.set_name,
            "action": "display",
            "signature": { "parameters": {}, "additionalParameters": "allowed" }
        });
        (key, entry)
    }

    fn manifest_routes(&self) -> Vec<Value> {
        if self.parent_set_name.is_some() {
            return vec![];
        }
        // Build routes: standard 2-level + optional 3rd level for each table facet
        let set = self.set_name;
        let mut routes = vec![
            serde_json::json!({
                "pattern": format!("{}:?query:", set),
                "name": format!("{}List", set),
                "target": format!("{}List", set)
            }),
            serde_json::json!({
                "pattern": format!("{}({{key}}):?query:", set),
                "name": format!("{}ObjectPage", set),
                "target": [format!("{}List", set), format!("{}ObjectPage", set)]
            }),
        ];
        // 3rd level: for each table facet referencing a child nav property.
        // Use scoped names ({parent}_{child}ObjectPage) to avoid colliding
        // with the child entity's own standalone ObjectPage target.
        for tf in self.annotations.iter().flat_map(|a| a.table_facets.iter()) {
            let nav = tf.navigation_property;
            // find the nav config to get the target_set
            if let Some(nc) = self.nav_configs.iter().find(|n| n.name == nav) {
                let child_set = &nc.target_set;
                let scoped = format!("{}_{}", set, child_set);
                routes.push(serde_json::json!({
                    "pattern": format!("{}({{key}})/{}({{key2}}):?query:", set, nav),
                    "name": format!("{}ObjectPage", scoped),
                    "target": [
                        format!("{}List", set),
                        format!("{}ObjectPage", set),
                        format!("{}ObjectPage", scoped)
                    ]
                }));
            }
        }
        routes
    }

    fn manifest_targets(&self) -> Vec<(String, Value)> {
        if self.parent_set_name.is_some() {
            return vec![];
        }
        let set = self.set_name;

        // Build navigation block for ObjectPage: table facets that link to child entities.
        // Use scoped route names to match the scoped 3rd-level routes.
        let mut nav_entries = serde_json::Map::new();
        for tf in self.annotations.iter().flat_map(|a| a.table_facets.iter()) {
            let nav = tf.navigation_property;
            if let Some(nc) = self.nav_configs.iter().find(|n| n.name == nav) {
                let scoped = format!("{}_{}", set, nc.target_set);
                nav_entries.insert(
                    nav.to_string(),
                    serde_json::json!({
                        "detail": { "route": format!("{}ObjectPage", scoped) }
                    }),
                );
            }
        }

        let mut obj_page_settings = serde_json::json!({
            "contextPath": format!("/{}", set)
        });
        if !nav_entries.is_empty() {
            obj_page_settings["navigation"] = Value::Object(nav_entries);
        }

        obj_page_settings["content"] = serde_json::json!(
{
                "header": {
                  "visible": true,
                  "anchorBarVisible": true,
                  "actions": {
                    "action1": {
                      "press": "products.demo.ext.controller.Handler.action1",
                    //   "visible": "{= %{status_code} !== 'submitted' && %{IsActiveEntity}}",
                      "enabled": true,
                      "text": "Action #1",
                      "position": {
                        "placement": "Before",
                        "anchor": "EditAction"
                      }
                    },
                    "action2": {
                      "press": "products.demo.ext.controller.Handler.action2",
                      "text": "Action #2",
                      "visible": true,
                      "enabled": true
                    }
                  }
                },
                "body": {
                  "sections": {
                    "panel1": {
                      "template": "products.demo.ext.fragment.Panel1",
                      "position": {
                        "placement": "After",
                        "anchor": "Main"
                      },
                      "title": "Panel #1"
                    },
                    "panel2": {
                      "template": "products.demo.ext.fragment.Panel2",
                      "position": {
                        "placement": "After",
                        "anchor": "Main"
                      },
                      "title": "Panel #2"
                    }
                  }
                }
            });
        /* "content": */

        let mut targets = vec![
            (
                format!("{}List", set),
                serde_json::json!({
                    "type": "Component",
                    "id": format!("{}List", set),
                    "name": "sap.fe.templates.ListReport",
                    "options": {
                        "settings": {
                            "contextPath": format!("/{}", set),
                            "variantManagement": "Page",
                            "initialLoad": "Enabled",
                            "navigation": {
                                (set): {
                                    "detail": {
                                        "route": format!("{}ObjectPage", set)
                                    }
                                }
                            }
                        }
                    },
                    "controlAggregation": "beginColumnPages",
                    "contextPattern": ""
                }),
            ),
            (
                format!("{}ObjectPage", set),
                serde_json::json!({
                    "type": "Component",
                    "id": format!("{}ObjectPage", set),
                    "name": "sap.fe.templates.ObjectPage",
                    "options": {
                        "settings": obj_page_settings
                    },
                    "controlAggregation": "midColumnPages",
                    "contextPattern": format!("/{}({{key}})", set)
                }),
            ),
        ];

        // 3rd-level child ObjectPages (scoped names to avoid collisions)
        for tf in self.annotations.iter().flat_map(|a| a.table_facets.iter()) {
            let nav = tf.navigation_property;
            if let Some(nc) = self.nav_configs.iter().find(|n| n.name == nav) {
                let scoped = format!("{}_{}", set, nc.target_set);
                targets.push((
                    format!("{}ObjectPage", scoped),
                    serde_json::json!({
                        "type": "Component",
                        "id": format!("{}ObjectPage", scoped),
                        "name": "sap.fe.templates.ObjectPage",
                        "options": {
                            "settings": {
                                "contextPath": format!("/{}/{}", set, nav)
                            }
                        },
                        "controlAggregation": "endColumnPages",
                        "contextPattern": format!("/{}({{key}})/{}({{key2}})", set, nav)
                    }),
                ));
            }
        }

        targets
    }
}

/// Wandelt rohe EntityConfigs in registrierbare ODataEntity-Instanzen um.
/// Returns the entity instances and any relationships extracted from the configs.
pub fn create_generic_entities(
    configs: Vec<EntityConfig>,
) -> (Vec<&'static dyn ODataEntity>, Vec<Relationship>) {
    // Build lookups for auto-deriving text_path and value_list on FK fields.
    let title_paths: HashMap<String, String> = configs
        .iter()
        .filter_map(|c| {
            c.annotations
                .as_ref()
                .map(|a| (c.set_name.clone(), a.header_info.title_path.clone()))
        })
        .collect();

    // Build parent_set lookup for determining compositions.
    let parent_sets: HashMap<String, String> = configs
        .iter()
        .filter_map(|c| {
            c.parent_set_name
                .as_ref()
                .map(|p| (c.set_name.clone(), p.clone()))
        })
        .collect();

    // Extract relationships from all configs before consuming them.
    let relationships: Vec<Relationship> = configs
        .iter()
        .flat_map(|c| config_to_relationships(c, &parent_sets))
        .collect();

    let entities: Vec<&'static dyn ODataEntity> = configs
        .into_iter()
        .map(|config| {
            let entity = GenericEntity::from_config(config, &title_paths);
            let leaked: &'static GenericEntity = Box::leak(Box::new(entity));
            leaked as &'static dyn ODataEntity
        })
        .collect();

    (entities, relationships)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn no_titles() -> HashMap<String, String> {
        HashMap::new()
    }

    // ── Helper: minimal EntityConfig ────────────────────────────

    /// Simple EntityConfig named TestItems
    fn minimal_config() -> EntityConfig {
        EntityConfig {
            set_name: "TestItems".to_string(),
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
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
                FieldConfig {
                    name: "Name".to_string(),
                    label: "Name".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(80),
                    precision: None,
                    scale: None,
                    immutable: false,
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
            ],
            navigation_properties: vec![],
            annotations: None,
            default_values: None,
            tile: None,
            value_lists: vec![],
        }
    }

    /// Complex EntityConfig named Orders mit Navigation, Annotations und Tile
    fn full_config() -> EntityConfig {
        EntityConfig {
            set_name: "Orders".to_string(),
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
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
                FieldConfig {
                    name: "Status".to_string(),
                    label: "Status".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(1),
                    precision: None,
                    scale: None,
                    immutable: false,
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
                FieldConfig {
                    name: "Amount".to_string(),
                    label: "Betrag".to_string(),
                    edm_type: "Edm.Decimal".to_string(),
                    max_length: None,
                    precision: Some(15),
                    scale: Some(2),
                    immutable: false,
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
            ],
            navigation_properties: vec![NavPropertyConfig {
                name: "Items".to_string(),
                target_type: "OrderItem".to_string(),
                target_set: "OrderItems".to_string(),
                is_collection: true,
                foreign_key: Some("OrderID".to_string()),
            }],
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
                    fields: vec![
                        "OrderID".to_string(),
                        "Status".to_string(),
                        "Amount".to_string(),
                    ],
                }],
                table_facets: vec![TableFacetConfig {
                    label: "Positionen".to_string(),
                    id: "ItemsFacet".to_string(),
                    navigation_property: "Items".to_string(),
                }],
            }),
            default_values: None,
            tile: Some(TileConfig {
                title: "Auftraege".to_string(),
                description: Some("Auftragsübersicht".to_string()),
                icon: Some("sap-icon://sales-order".to_string()),
            }),
            value_lists: vec![],
        }
    }

    fn child_config() -> EntityConfig {
        EntityConfig {
            set_name: "OrderItems".to_string(),
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
                    references_entity: None,
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
                FieldConfig {
                    name: "OrderID".to_string(),
                    label: "Auftragsnr.".to_string(),
                    edm_type: "Edm.String".to_string(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    immutable: true,
                    references_entity: Some("Orders".to_string()),
                    prefer_dialog: false,
                    value_source: None,
                    computed: false,
                    text_path: None,
                    searchable: false,
                    show_in_list: false,
                    list_sort_order: None,
                    list_importance: None,
                    list_criticality_path: None,
                    form_group: None,
                },
            ],
            navigation_properties: vec![],
            annotations: Some(AnnotationsConfig {
                selection_fields: vec![],
                line_item: vec![LineItemConfig {
                    name: "ItemID".to_string(),
                    label: None,
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                    semantic_object: None,
                }],
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
            default_values: None,
            tile: None,
            value_lists: vec![],
        }
    }

    // ── Serde Roundtrip Tests ───────────────────────────────────

    #[test]
    fn config_serde_roundtrip_minimal() {
        let config = minimal_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EntityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.set_name, "TestItems");
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
        assert!(field0.get("immutable").is_none() || field0.get("immutable") == Some(&json!(true)));
        // only appears when true
    }

    #[test]
    fn config_serde_child_with_parent() {
        let config = child_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EntityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.parent_set_name.as_deref(), Some("Orders"));
        assert_eq!(
            parsed.fields[1].references_entity.as_deref(),
            Some("Orders")
        );
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
    fn config_deserialize_from_json() {
        // Parse config JSON with all fields populated
        let json = r#"{
            "set_name": "Customers",
            "key_field": "CustomerID",
            "type_name": "Customer",
            "tile": { "title": "Kunden", "description": "Kundenübersicht", "icon": "sap-icon://customer" },
            "fields": [
                { "name": "CustomerID", "label": "ID", "edm_type": "Edm.String", "max_length": 10, "immutable": true },
                { "name": "CustomerName", "label": "Name", "edm_type": "Edm.String", "max_length": 80 }
            ],
            "annotations": {
                "selection_fields": ["CustomerName"],
                "line_item": [{ "name": "CustomerID", "importance": "High" }, { "name": "CustomerName" }],
                "header_info": { "type_name": "Kunde", "type_name_plural": "Kunden", "title_path": "CustomerName", "description_path": "CustomerID" },
                "header_facets": [],
                "data_points": [],
                "facet_sections": [{ "label": "Details", "id": "Details", "field_group_qualifier": "Main", "field_group_label": "Stamm" }],
                "field_groups": [{ "qualifier": "Main", "fields": ["CustomerID", "CustomerName"] }],
                "table_facets": []
            }
        }"#;
        let config: EntityConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.set_name, "Customers");
        assert_eq!(config.fields.len(), 2);
        assert!(config.annotations.is_some());
        let ann = config.annotations.as_ref().unwrap();
        assert!(!ann.line_item.is_empty());
        assert!(!ann.facet_sections.is_empty());
    }

    #[test]
    fn config_deserialize_serialize_roundtrip() {
        // Roundtrip: parse → serialize → parse again → compare key fields
        let json = r#"{
            "set_name": "Contacts",
            "key_field": "ContactID",
            "type_name": "Contact",
            "fields": [
                { "name": "ContactID", "label": "ID", "edm_type": "Edm.String", "max_length": 10, "immutable": true },
                { "name": "CustomerID", "label": "Kunde", "edm_type": "Edm.String", "references_entity": "Customers" }
            ],
            "navigation_properties": [
                { "name": "Customer", "target_type": "Customer", "target_set": "Customers", "is_collection": false, "foreign_key": "CustomerID" }
            ],
            "annotations": {
                "selection_fields": ["CustomerID"],
                "line_item": [{ "name": "ContactID" }, { "name": "CustomerID" }],
                "header_info": { "type_name": "Kontakt", "type_name_plural": "Kontakte", "title_path": "ContactID", "description_path": "" },
                "header_facets": [],
                "data_points": [],
                "facet_sections": [{ "label": "Daten", "id": "Data", "field_group_qualifier": "Main", "field_group_label": "Main" }],
                "field_groups": [{ "qualifier": "Main", "fields": ["ContactID", "CustomerID"] }],
                "table_facets": []
            }
        }"#;
        let config: EntityConfig = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let reparsed: EntityConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.set_name, reparsed.set_name);
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
        let entity = GenericEntity::from_config(minimal_config(), &no_titles());
        assert_eq!(entity.set_name(), "TestItems");
        assert_eq!(entity.key_field(), "ID");
        assert_eq!(entity.type_name(), "TestItem");
        assert!(entity.parent_set_name().is_none());
        assert_eq!(entity.mock_data().len(), 0);
    }

    #[test]
    fn generic_entity_fields_def() {
        let entity = GenericEntity::from_config(minimal_config(), &no_titles());
        let fields = entity.fields_def().unwrap();
        // ID auto-inserted + ItemID + Name = 3
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name, "ID");
        assert_eq!(fields[0].edm_type, "Edm.Guid");
        assert!(fields[0].computed);
        assert_eq!(fields[1].name, "ItemID");
        assert_eq!(fields[1].label, "Item Nr.");
        assert!(fields[1].immutable);
        assert_eq!(fields[1].max_length, Some(10));
        assert_eq!(fields[2].name, "Name");
        assert!(!fields[2].immutable);
    }

    #[test]
    fn generic_entity_annotations() {
        let entity = GenericEntity::from_config(full_config(), &no_titles());
        let ann = entity.annotations_def().unwrap();
        assert_eq!(ann.header_info.type_name, "Auftrag");
        assert_eq!(ann.header_info.type_name_plural, "Auftraege");
        assert_eq!(ann.facet_sections.len(), 1);
        assert_eq!(ann.facet_sections[0].id, "General");
        assert_eq!(ann.table_facets.len(), 1);
        assert_eq!(ann.table_facets[0].navigation_property, "Items");
    }

    #[test]
    fn generic_entity_navigation_properties() {
        let entity = GenericEntity::from_config(full_config(), &no_titles());
        let navs = entity.navigation_properties();
        assert_eq!(navs.len(), 1);
        assert_eq!(navs[0].name, "Items");
        assert_eq!(navs[0].target_type, "OrderItem");
        assert!(navs[0].is_collection);
    }

    #[test]
    fn generic_entity_parent_set_name() {
        let entity = GenericEntity::from_config(child_config(), &no_titles());
        assert_eq!(entity.parent_set_name(), Some("Orders"));
    }

    #[test]
    fn generic_entity_entity_set_xml() {
        let entity = GenericEntity::from_config(full_config(), &no_titles());
        let xml = entity.entity_set();
        assert!(xml.contains("EntitySet Name=\"Orders\""));
        assert!(xml.contains("EntityType=\"Service.Order\""));
        assert!(xml.contains("Path=\"Items\" Target=\"OrderItems\""));
        assert!(xml.contains("SiblingEntity"));
        assert!(xml.contains("DraftAdministrativeData"));
    }

    #[test]
    fn generic_entity_entity_set_xml_no_nav() {
        let entity = GenericEntity::from_config(minimal_config(), &no_titles());
        let xml = entity.entity_set();
        assert!(xml.contains("EntitySet Name=\"TestItems\""));
        // Only SiblingEntity + DraftAdministrativeData bindings
        assert!(xml.contains("SiblingEntity"));
        assert!(xml.contains("DraftAdministrativeData"));
    }

    #[test]
    fn generic_entity_apps_json_with_tile() {
        let entity = GenericEntity::from_config(full_config(), &no_titles());
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
        let entity = GenericEntity::from_config(minimal_config(), &no_titles());
        assert!(entity.apps_json_entry().is_none());
    }

    #[test]
    fn generic_entity_expand_1n() {
        let order_entity = GenericEntity::from_config(full_config(), &no_titles());
        let child_entity = GenericEntity::from_config(child_config(), &no_titles());
        let entities: Vec<&dyn ODataEntity> = vec![
            &order_entity as &dyn ODataEntity,
            &child_entity as &dyn ODataEntity,
        ];

        let mut store: HashMap<String, Vec<Value>> = HashMap::new();
        store.insert(
            "OrderItems".to_string(),
            vec![
                json!({"ItemID": "I001", "OrderID": "O001"}),
                json!({"ItemID": "I002", "OrderID": "O001"}),
                json!({"ItemID": "I003", "OrderID": "O002"}),
            ],
        );

        let mut record = json!({"ID": "O001", "Status": "A"});
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
            type_name: "Contact".to_string(),
            parent_set_name: None,
            fields: vec![FieldConfig {
                name: "CustomerID".to_string(),
                label: "Kunde".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                references_entity: None,
                prefer_dialog: false,
                value_source: None,
                computed: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            }],
            navigation_properties: vec![NavPropertyConfig {
                name: "Customer".to_string(),
                target_type: "Customer".to_string(),
                target_set: "Customers".to_string(),
                is_collection: false,
                foreign_key: Some("CustomerID".to_string()),
            }],
            annotations: None,
            default_values: None,
            tile: None,
            value_lists: vec![],
        };
        let customer_config = EntityConfig {
            set_name: "Customers".to_string(),
            type_name: "Customer".to_string(),
            parent_set_name: None,
            fields: vec![FieldConfig {
                name: "CustomerName".to_string(),
                label: "Name".to_string(),
                edm_type: "Edm.String".to_string(),
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                references_entity: None,
                prefer_dialog: false,
                value_source: None,
                computed: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            }],
            navigation_properties: vec![],
            annotations: None,
            default_values: None,
            tile: None,
            value_lists: vec![],
        };

        let contact_entity = GenericEntity::from_config(contact_config, &no_titles());
        let customer_entity = GenericEntity::from_config(customer_config, &no_titles());
        let entities: Vec<&dyn ODataEntity> = vec![
            &contact_entity as &dyn ODataEntity,
            &customer_entity as &dyn ODataEntity,
        ];

        let mut store: HashMap<String, Vec<Value>> = HashMap::new();
        store.insert(
            "Customers".to_string(),
            vec![
                json!({"ID": "C001", "CustomerName": "Acme"}),
                json!({"ID": "C002", "CustomerName": "Global"}),
            ],
        );

        let mut record = json!({"ID": "K001", "CustomerID": "C002"});
        contact_entity.expand_record(&mut record, &["Customer"], &entities, &store);

        assert_eq!(record["Customer"]["ID"], "C002");
        assert_eq!(record["Customer"]["CustomerName"], "Global");
    }

    #[test]
    fn generic_entity_expand_unknown_nav_ignored() {
        let entity = GenericEntity::from_config(full_config(), &no_titles());
        let entities: Vec<&dyn ODataEntity> = vec![&entity as &dyn ODataEntity];
        let store: HashMap<String, Vec<Value>> = HashMap::new();

        let mut record = json!({"OrderID": "O001"});
        entity.expand_record(&mut record, &["NonExistent"], &entities, &store);
        // Record unchanged — no panic
        assert!(record.get("NonExistent").is_none());
    }

    #[test]
    fn generic_entity_debug_impl() {
        let entity = GenericEntity::from_config(minimal_config(), &no_titles());
        let dbg = format!("{:?}", entity);
        assert!(dbg.contains("TestItems"));
        assert!(dbg.contains("TestItem"));
    }

    // ── create_generic_entities Tests ─────────────────────────

    #[test]
    fn create_generic_entities_preserves_count() {
        let configs = vec![minimal_config(), full_config()];
        let (entities, relationships) = create_generic_entities(configs);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].set_name(), "TestItems");
        assert_eq!(entities[1].set_name(), "Orders");
        // full_config has one 1:N nav (Items) → one composition relationship
        assert!(
            relationships
                .iter()
                .any(|r| r.name == "Orders_OrderItems"),
            "missing Orders_OrderItems rel: {relationships:?}"
        );
    }

    // ── EntitySpec + Relationship extraction ─────────────────────

    #[test]
    fn config_to_entity_spec_excludes_fk_fields() {
        let config = EntityConfig {
            set_name: "Tickets".into(),
            type_name: "Ticket".into(),
            parent_set_name: None,
            fields: vec![
                FieldConfig {
                    name: "Title".into(), label: "Title".into(),
                    edm_type: "Edm.String".into(), max_length: Some(80),
                    precision: None, scale: None, immutable: false, computed: false,
                    references_entity: None, prefer_dialog: false, value_source: None,
                    text_path: None, searchable: true, show_in_list: true,
                    list_sort_order: Some(1), list_importance: None,
                    list_criticality_path: None, form_group: Some("General".into()),
                },
                FieldConfig {
                    name: "CustomerID".into(), label: "Customer".into(),
                    edm_type: "Edm.Guid".into(), max_length: None,
                    precision: None, scale: None, immutable: false, computed: false,
                    references_entity: Some("Customers".into()), prefer_dialog: false,
                    value_source: None, text_path: None, searchable: false,
                    show_in_list: false, list_sort_order: None, list_importance: None,
                    list_criticality_path: None, form_group: Some("General".into()),
                },
            ],
            navigation_properties: vec![],
            annotations: Some(AnnotationsConfig {
                selection_fields: vec!["Title".into()],
                line_item: vec![],
                header_info: HeaderInfoConfig {
                    type_name: "Ticket".into(),
                    type_name_plural: "Tickets".into(),
                    title_path: "Title".into(),
                    description_path: "Title".into(),
                },
                header_facets: vec![], data_points: vec![],
                facet_sections: vec![], field_groups: vec![], table_facets: vec![],
            }),
            default_values: None, tile: None, value_lists: vec![],
        };

        let spec = super::config_to_entity_spec(&config.set_name, &config.annotations, &config.fields);

        // CustomerID (FK) should be excluded
        let field_names: Vec<&str> = spec.fields.iter().map(|f| f.name()).collect();
        assert!(field_names.contains(&"Title"), "missing Title");
        assert!(!field_names.contains(&"CustomerID"), "FK field should be excluded");
        assert_eq!(spec.title_field, Some("Title".into()));
    }

    #[test]
    fn config_to_relationships_extracts_all() {
        let parent_sets = HashMap::from([("OrderItems".into(), "Orders".into())]);

        let config = EntityConfig {
            set_name: "Orders".into(),
            type_name: "Order".into(),
            parent_set_name: None,
            fields: vec![
                FieldConfig {
                    name: "CustomerID".into(), label: "Customer".into(),
                    edm_type: "Edm.Guid".into(), max_length: None,
                    precision: None, scale: None, immutable: false, computed: false,
                    references_entity: Some("Customers".into()), prefer_dialog: false,
                    value_source: None, text_path: None, searchable: false,
                    show_in_list: false, list_sort_order: None, list_importance: None,
                    list_criticality_path: None, form_group: None,
                },
            ],
            navigation_properties: vec![NavPropertyConfig {
                name: "Items".into(),
                target_type: "OrderItem".into(),
                target_set: "OrderItems".into(),
                is_collection: true,
                foreign_key: Some("OrderID".into()),
            }],
            annotations: None,
            default_values: None, tile: None, value_lists: vec![],
        };

        let rels = super::config_to_relationships(&config, &parent_sets);
        assert_eq!(rels.len(), 2);

        // Composition: Orders → OrderItems
        let comp = rels.iter().find(|r| r.name == "Orders_OrderItems").unwrap();
        assert!(comp.owned);
        assert_eq!(comp.one.entity, "Orders");
        assert_eq!(comp.one.nav_name, "Items");
        assert_eq!(comp.many.entity, "OrderItems");
        assert_eq!(comp.fk_field, Some("OrderID".into()));

        // Association: Orders.CustomerID → Customers
        let assoc = rels.iter().find(|r| r.name == "Orders_Customer").unwrap();
        assert!(!assoc.owned);
        assert_eq!(assoc.one.entity, "Customers");
        assert_eq!(assoc.many.entity, "Orders");
        assert_eq!(assoc.many.nav_name, "Customer");
        assert_eq!(assoc.fk_field, Some("CustomerID".into()));
    }

    #[test]
    fn generic_entity_spec_resolves_through_pipeline() {
        use crate::model;

        let configs = vec![
            EntityConfig {
                set_name: "Tasks".into(),
                type_name: "Task".into(),
                parent_set_name: None,
                fields: vec![
                    FieldConfig {
                        name: "TaskName".into(), label: "Task Name".into(),
                        edm_type: "Edm.String".into(), max_length: Some(100),
                        precision: None, scale: None, immutable: false, computed: false,
                        references_entity: None, prefer_dialog: false, value_source: None,
                        text_path: None, searchable: true, show_in_list: true,
                        list_sort_order: Some(1), list_importance: None,
                        list_criticality_path: None, form_group: Some("General".into()),
                    },
                    FieldConfig {
                        name: "AssigneeID".into(), label: "Assignee".into(),
                        edm_type: "Edm.Guid".into(), max_length: None,
                        precision: None, scale: None, immutable: false, computed: false,
                        references_entity: Some("Users".into()), prefer_dialog: false,
                        value_source: None, text_path: None, searchable: false,
                        show_in_list: false, list_sort_order: None, list_importance: None,
                        list_criticality_path: None, form_group: Some("General".into()),
                    },
                ],
                navigation_properties: vec![NavPropertyConfig {
                    name: "SubTasks".into(),
                    target_type: "SubTask".into(),
                    target_set: "SubTasks".into(),
                    is_collection: true,
                    foreign_key: Some("TaskID".into()),
                }],
                annotations: Some(AnnotationsConfig {
                    selection_fields: vec!["TaskName".into()],
                    line_item: vec![],
                    header_info: HeaderInfoConfig {
                        type_name: "Task".into(),
                        type_name_plural: "Tasks".into(),
                        title_path: "TaskName".into(),
                        description_path: "TaskName".into(),
                    },
                    header_facets: vec![], data_points: vec![],
                    facet_sections: vec![], field_groups: vec![],
                    table_facets: vec![TableFacetConfig {
                        label: "Sub Tasks".into(),
                        id: "SubTasksSection".into(),
                        navigation_property: "SubTasks".into(),
                    }],
                }),
                default_values: None, tile: None, value_lists: vec![],
            },
            EntityConfig {
                set_name: "SubTasks".into(),
                type_name: "SubTask".into(),
                parent_set_name: Some("Tasks".into()),
                fields: vec![FieldConfig {
                    name: "SubName".into(), label: "Sub Task Name".into(),
                    edm_type: "Edm.String".into(), max_length: Some(80),
                    precision: None, scale: None, immutable: false, computed: false,
                    references_entity: None, prefer_dialog: false, value_source: None,
                    text_path: None, searchable: false, show_in_list: true,
                    list_sort_order: None, list_importance: None,
                    list_criticality_path: None, form_group: None,
                }],
                navigation_properties: vec![],
                annotations: Some(AnnotationsConfig {
                    selection_fields: vec![],
                    line_item: vec![],
                    header_info: HeaderInfoConfig {
                        type_name: "Sub Task".into(),
                        type_name_plural: "Sub Tasks".into(),
                        title_path: "SubName".into(),
                        description_path: "SubName".into(),
                    },
                    header_facets: vec![], data_points: vec![],
                    facet_sections: vec![], field_groups: vec![], table_facets: vec![],
                }),
                default_values: None, tile: None, value_lists: vec![],
            },
        ];

        let (entities, relationships) = create_generic_entities(configs);
        assert_eq!(entities.len(), 2);

        // Collect specs from entities
        let specs: Vec<_> = entities.iter().filter_map(|e| e.entity_spec()).collect();
        assert_eq!(specs.len(), 2);

        // Resolve through the model pipeline
        let resolved = model::resolve(&specs, &relationships);

        // Tasks: should have TaskName field, SubTasks nav, Assignee nav, and auto-created Users
        let tasks = resolved.iter().find(|e| e.set_name == "Tasks").unwrap();
        assert!(tasks.properties.iter().any(|p| p.name == "TaskName"));
        // FK AssigneeID injected by resolver from the relationship
        assert!(tasks.properties.iter().any(|p| p.name == "AssigneeID"),
            "missing AssigneeID FK: {:?}", tasks.properties.iter().map(|p| &p.name).collect::<Vec<_>>());
        // Composition nav
        assert!(tasks.nav_properties.iter().any(|n| n.name == "SubTasks" && n.is_collection));
        // 1:1 nav to Users
        assert!(tasks.nav_properties.iter().any(|n| n.name == "Assignee" && !n.is_collection));

        // SubTasks: child of Tasks
        let subtasks = resolved.iter().find(|e| e.set_name == "SubTasks").unwrap();
        assert_eq!(subtasks.parent_set_name.as_deref(), Some("Tasks"));
        assert!(subtasks.properties.iter().any(|p| p.name == "TaskID"),
            "missing TaskID FK: {:?}", subtasks.properties.iter().map(|p| &p.name).collect::<Vec<_>>());

        // Users: auto-created from relationship
        let users = resolved.iter().find(|e| e.set_name == "Users").unwrap();
        assert_eq!(users.title_field, "Name");
    }
}
