use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityNavigationEntity;

impl ODataEntity for EntityNavigationEntity {
    fn set_name(&self) -> &'static str {
        "EntityNavigations"
    }
    fn key_field(&self) -> &'static str {
        "NavID"
    }
    fn type_name(&self) -> &'static str {
        "EntityNavigation"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "NavID",         label: "Nav-ID",          edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "SetName",       label: "EntitySet",       edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "NavName",       label: "Name",            edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "TargetType",    label: "Ziel-Typ",        edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "TargetSet",     label: "Ziel-EntitySet",  edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "IsCollection",  label: "Ist Kollektion",  edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "ForeignKey",    label: "Fremdschluessel", edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "SortOrder",     label: "Reihenfolge",     edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityNavigations\" EntityType=\"{ns}.EntityNavigation\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityNavigations\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "SortOrder",     label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "NavName",       label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "TargetSet",     label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "IsCollection",  label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "ForeignKey",    label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Navigation Property",
                type_name_plural: "Navigation Properties",
                title_path: "NavName",
                description_path: "TargetSet",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Navigation-Konfiguration", id: "NavConfig", field_group_qualifier: "NavInfo", field_group_label: "Informationen" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "NavInfo", fields: &["NavID", "SetName", "NavName", "TargetType", "TargetSet", "IsCollection", "ForeignKey", "SortOrder"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_EntityNavigations-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
