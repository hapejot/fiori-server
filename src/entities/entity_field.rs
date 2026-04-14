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
        "ID"
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
            FieldDef { name: "ID",                   label: "ID",                 edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "ConfigID",             label: "Config-ID",           edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "SetName",              label: "EntitySet",           edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "FieldName",            label: "Feldname",            edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(1), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "Label",                label: "Bezeichnung",         edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(2), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "EdmType",              label: "Datentyp",            edm_type: "Edm.String",  max_length: Some(30),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: Some(edm_types_id) , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(4), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "MaxLength",            label: "Max. Laenge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(5), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "Precision",            label: "Praezision",          edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "Scale",                label: "Dezimalstellen",      edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "IsImmutable",          label: "Unveraenderlich",     edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(6), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "IsComputed",           label: "Berechnet",           edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(7), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "ReferencesEntity",    label: "Referenz-Entity",     edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "PreferDialog",         label: "Suchdialog",           edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "ValueSource",          label: "Werteliste",          edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: Some("FieldValueLists"), value_source: None , prefer_dialog: false, text_path: Some("_ValueList/ListName"), searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "TextPath",             label: "Textpfad",            edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(8), list_importance: Some("Low"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "DefaultValue",         label: "Standardwert",        edm_type: "Edm.String",  max_length: Some(120), precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "SortOrder",            label: "Reihenfolge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(0), list_importance: Some("High"), list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "ShowInLineItem",       label: "In Liste",            edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(3), list_importance: Some("High"), list_criticality_path: None, form_group: Some("LineItemProps")},
            FieldDef { name: "LineItemImportance",   label: "Wichtigkeit",         edm_type: "Edm.String",  max_length: Some(10),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("LineItemProps")},
            FieldDef { name: "LineItemLabel",        label: "Listen-Label",        edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("LineItemProps")},
            FieldDef { name: "LineItemCriticalityPath", label: "Kritikalitaets-Pfad", edm_type: "Edm.String", max_length: Some(40), precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("LineItemProps")},
            FieldDef { name: "LineItemSemanticObject", label: "Listen-Sem.Object", edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("LineItemProps")},
            FieldDef { name: "Searchable",             label: "Suchfeld",            edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
            FieldDef { name: "FormGroup",              label: "Formulargruppe",      edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("FieldProps")},
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
