use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::spec::{self, EntitySpec};
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityFacetEntity;

impl ODataEntity for EntityFacetEntity {
    fn set_name(&self) -> &'static str {
        "EntityFacets"
    }
    fn type_name(&self) -> &'static str {
        "EntityFacet"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(spec::meta_package::entity_facets())
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
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
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            },
            FieldDef {
                name: "ConfigID",
                label: "Config-ID",
                edm_type: "Edm.Guid",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
                computed: true,
                references_entity: Some("EntityConfig"),
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            },
            FieldDef {
                name: "SectionLabel",
                label: "Abschnitt-Label",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(1),
                list_importance: Some("High"),
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
            FieldDef {
                name: "SectionId",
                label: "Abschnitt-ID",
                edm_type: "Edm.String",
                max_length: Some(40),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(2),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
            FieldDef {
                name: "FieldGroupQualifier",
                label: "FieldGroup-Qualifier",
                edm_type: "Edm.String",
                max_length: Some(40),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(3),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
            FieldDef {
                name: "FieldGroupLabel",
                label: "FieldGroup-Label",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
            FieldDef {
                name: "FieldGroupFields",
                label: "Felder (kommasep.)",
                edm_type: "Edm.String",
                max_length: Some(500),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(4),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
            FieldDef {
                name: "SortOrder",
                label: "Reihenfolge",
                edm_type: "Edm.Int32",
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(0),
                list_importance: Some("High"),
                list_criticality_path: None,
                form_group: Some("FacetInfo"),
            },
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
            header_info: HeaderInfoDef {
                type_name: "Facette",
                type_name_plural: "Facetten",
                title_path: "SectionLabel",
                description_path: "SectionId",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[FacetSectionDef {
                label: "Facet-Konfiguration",
                id: "FacetConfig",
                field_group_qualifier: "FacetInfo",
                field_group_label: "Informationen",
            }],
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
