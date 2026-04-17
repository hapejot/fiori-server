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
    fn type_name(&self) -> &'static str {
        "EntityNavigation"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ID",            label: "ID",              edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "ConfigID",      label: "Config-ID",        edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "SetName",       label: "EntitySet",       edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: true, computed: true,   references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "NavName",       label: "Name",            edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(1), list_importance: Some("High"), list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "TargetType",    label: "Ziel-Typ",        edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "TargetSet",     label: "Ziel-EntitySet",  edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(2), list_importance: None, list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "IsCollection",  label: "Ist Kollektion",  edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(3), list_importance: None, list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "ForeignKey",    label: "Fremdschluessel", edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(4), list_importance: None, list_criticality_path: None, form_group: Some("NavInfo")},
            FieldDef { name: "SortOrder",     label: "Reihenfolge",     edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(0), list_importance: Some("High"), list_criticality_path: None, form_group: Some("NavInfo")},
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
