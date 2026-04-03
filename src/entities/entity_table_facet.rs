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
        "TableFacetID"
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
            FieldDef { name: "TableFacetID",       label: "TableFacet-ID",      edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: true, computed: false,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SetName",            label: "EntitySet",           edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: true, computed: false,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FacetLabel",         label: "Label",              edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FacetId",            label: "Facet-ID",           edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "NavigationProperty", label: "Navigation Property", edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SortOrder",          label: "Reihenfolge",        edm_type: "Edm.Int32",  max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
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
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "SortOrder",          label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "FacetLabel",         label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "FacetId",            label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "NavigationProperty", label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
            ],
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
            field_groups: &[
                FieldGroupDef { qualifier: "TFInfo", fields: &["TableFacetID", "SetName", "FacetLabel", "FacetId", "NavigationProperty", "SortOrder"] },
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
