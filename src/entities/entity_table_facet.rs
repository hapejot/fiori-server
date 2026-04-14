use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityTableFacetEntity;

impl ODataEntity for EntityTableFacetEntity {
    fn set_name(&self) -> &'static str {
        "EntityTableFacets"
    }
    fn key_field(&self) -> &'static str {
        "ID"
    }
    fn type_name(&self) -> &'static str {
        "EntityTableFacet"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ID",                 label: "ID",                 edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "ConfigID",           label: "Config-ID",           edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "SetName",            label: "EntitySet",           edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("TFInfo")},
            FieldDef { name: "FacetLabel",         label: "Label",              edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(1), list_importance: Some("High"), list_criticality_path: None, form_group: Some("TFInfo")},
            FieldDef { name: "FacetId",            label: "Facet-ID",           edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(2), list_importance: None, list_criticality_path: None, form_group: Some("TFInfo")},
            FieldDef { name: "NavigationProperty", label: "Navigation Property", edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(3), list_importance: None, list_criticality_path: None, form_group: Some("TFInfo")},
            FieldDef { name: "SortOrder",          label: "Reihenfolge",        edm_type: "Edm.Int32",  max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(0), list_importance: Some("High"), list_criticality_path: None, form_group: Some("TFInfo")},
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityTableFacets\" EntityType=\"{ns}.EntityTableFacet\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityTableFacets\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Tabellen-Facette",
                type_name_plural: "Tabellen-Facetten",
                title_path: "FacetLabel",
                description_path: "NavigationProperty",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Tabellen-Facet-Konfiguration", id: "TFConfig", field_group_qualifier: "TFInfo", field_group_label: "Informationen" },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_EntityTableFacets-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
