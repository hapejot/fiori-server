use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityFacetEntity;

impl ODataEntity for EntityFacetEntity {
    fn set_name(&self) -> &'static str {
        "EntityFacets"
    }
    fn key_field(&self) -> &'static str {
        "FacetID"
    }
    fn type_name(&self) -> &'static str {
        "EntityFacet"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "FacetID",              label: "Facet-ID",            edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SetName",              label: "EntitySet",            edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SectionLabel",         label: "Abschnitt-Label",      edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SectionId",            label: "Abschnitt-ID",         edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FieldGroupQualifier",  label: "FieldGroup-Qualifier", edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FieldGroupLabel",      label: "FieldGroup-Label",     edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "FieldGroupFields",     label: "Felder (kommasep.)",   edm_type: "Edm.String", max_length: Some(500), precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SortOrder",            label: "Reihenfolge",          edm_type: "Edm.Int32",  max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityFacets\" EntityType=\"{ns}.EntityFacet\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityFacets\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "SortOrder",            label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "SectionLabel",         label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "SectionId",            label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "FieldGroupQualifier",  label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "FieldGroupFields",     label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Facette",
                type_name_plural: "Facetten",
                title_path: "SectionLabel",
                description_path: "SectionId",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Facet-Konfiguration", id: "FacetConfig", field_group_qualifier: "FacetInfo", field_group_label: "Informationen" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "FacetInfo", fields: &["FacetID", "SetName", "SectionLabel", "SectionId", "FieldGroupQualifier", "FieldGroupLabel", "FieldGroupFields", "SortOrder"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_EntityFacets-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
