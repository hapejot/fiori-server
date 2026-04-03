use std::sync::LazyLock;

use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::{value_list_id, ODataEntity};
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityFieldEntity;

impl ODataEntity for EntityFieldEntity {
    fn set_name(&self) -> &'static str {
        "EntityFields"
    }
    fn key_field(&self) -> &'static str {
        "FieldID"
    }
    fn type_name(&self) -> &'static str {
        "EntityField"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: LazyLock<Vec<FieldDef>> = LazyLock::new(|| {
            // value_source speichert die UUID der FieldValueList, nicht den Namen
            let edm_types_id: &'static str = Box::leak(value_list_id("EdmTypes").into_boxed_str());
            vec![
            FieldDef { name: "FieldID",              label: "Feld-ID",            edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: true, computed: false,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SetName",              label: "EntitySet",           edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: true, computed: false,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FieldName",            label: "Feldname",            edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "Label",                label: "Bezeichnung",         edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "EdmType",              label: "Datentyp",            edm_type: "Edm.String",  max_length: Some(30),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: Some(edm_types_id) , value_list: None, text_path: None},
            FieldDef { name: "MaxLength",            label: "Max. Laenge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "Precision",            label: "Praezision",          edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "Scale",                label: "Dezimalstellen",      edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "IsImmutable",          label: "Unveraenderlich",     edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "IsComputed",           label: "Berechnet",           edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SemanticObject",       label: "Semantic Object",     edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "ValueSource",          label: "Werteliste",          edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: Some("FieldValueLists"), value_source: None , value_list: Some(&ValueListDef { collection_path: "FieldValueLists", key_property: "ID", display_property: Some("ListName"), fixed_values: false }), text_path: Some("_ValueList/ListName")},
            FieldDef { name: "TextPath",             label: "Textpfad",            edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "DefaultValue",         label: "Standardwert",        edm_type: "Edm.String",  max_length: Some(120), precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SortOrder",            label: "Reihenfolge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "ShowInLineItem",       label: "In Liste",            edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "LineItemImportance",   label: "Wichtigkeit",         edm_type: "Edm.String",  max_length: Some(10),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "LineItemLabel",        label: "Listen-Label",        edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "LineItemCriticalityPath", label: "Kritikalitaets-Pfad", edm_type: "Edm.String", max_length: Some(40), precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "LineItemSemanticObject", label: "Listen-Sem.Object", edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            ]
        });
        Some(FIELDS.as_slice())
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            // 1:1 navigation to FieldValueList for Common.Text on ValueSource
            NavigationPropertyDef { name: "_ValueList", target_type: "FieldValueList", is_collection: false, foreign_key: Some("ValueSource") },
        ];
        NAV
    }

    fn expand_record(&self, record: &mut Value, nav_properties: &[&str], entities: &[&dyn ODataEntity], data_store: &std::collections::HashMap<String, Vec<Value>>) {
        // _ValueList expansion: attach FieldValueList based on ValueSource (UUID)
        if nav_properties.contains(&"_ValueList") {
            if let Some(list_id) = record.get("ValueSource").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                if !list_id.is_empty() {
                    let vl_entity = entities.iter().find(|e| e.set_name() == "FieldValueLists");
                    if let Some(entity) = vl_entity {
                        let data = data_store.get(entity.set_name())
                            .cloned()
                            .unwrap_or_else(|| entity.mock_data());
                        let vl = data.into_iter()
                            .find(|p| p.get("ID").and_then(|v| v.as_str()) == Some(&list_id));
                        if let Some(obj) = record.as_object_mut() {
                            obj.insert("_ValueList".to_string(), vl.unwrap_or(Value::Null));
                        }
                    }
                }
            }
        }
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityFields\" EntityType=\"{ns}.EntityField\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityFields\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             <NavigationPropertyBinding Path=\"_ValueList\" Target=\"FieldValueLists\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "SortOrder",      label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "FieldName",      label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Label",          label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "ShowInLineItem", label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "EdmType",        label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "MaxLength",      label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "IsImmutable",    label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "IsComputed",     label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Felddefinition",
                type_name_plural: "Felddefinitionen",
                title_path: "FieldName",
                description_path: "Label",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Feldeigenschaften",    id: "FieldProps",   field_group_qualifier: "FieldProps",   field_group_label: "Eigenschaften" },
                FacetSectionDef { label: "Listen-Konfiguration", id: "LineItemProps", field_group_qualifier: "LineItemProps", field_group_label: "LineItem" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "FieldProps",    fields: &["FieldID", "SetName", "FieldName", "Label", "EdmType", "MaxLength", "Precision", "Scale", "IsImmutable", "IsComputed", "SemanticObject", "ValueSource", "TextPath", "DefaultValue", "SortOrder"] },
                FieldGroupDef { qualifier: "LineItemProps", fields: &["ShowInLineItem", "LineItemImportance", "LineItemLabel", "LineItemCriticalityPath", "LineItemSemanticObject"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_EntityFields-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
