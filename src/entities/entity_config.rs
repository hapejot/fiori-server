use std::collections::HashMap;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityConfigEntity;

impl ODataEntity for EntityConfigEntity {
    fn set_name(&self) -> &'static str {
        "EntityConfigs"
    }
    fn key_field(&self) -> &'static str {
        "ID"
    }
    fn type_name(&self) -> &'static str {
        "EntityConfig"
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
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
                name: "SetName",
                label: "{@i18n>entityset}",
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
                searchable: true,
                show_in_list: true,
                list_sort_order: Some(0),
                list_importance: Some("High"),
                list_criticality_path: None,
                form_group: Some("Basic"),
            },
            FieldDef {
                name: "TypeName",
                label: "{@i18n>entitytype}",
                edm_type: "Edm.String",
                max_length: Some(40),
                precision: None,
                scale: None,
                immutable: false,
                computed: true,
                references_entity: None,
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: true,
                show_in_list: true,
                list_sort_order: Some(1),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Basic"),
            },
            // FieldDef {
            //     name: "ParentSetName",
            //     label: "{@i18n>parententityset}",
            //     edm_type: "Edm.String",
            //     max_length: Some(40),
            //     precision: None,
            //     scale: None,
            //     immutable: false,
            //     computed: false,
            //     references_entity: None,
            //     value_source: None,
            //     prefer_dialog: false,
            //     text_path: None,
            //     searchable: false,
            //     show_in_list: false,
            //     list_sort_order: None,
            //     list_importance: None,
            //     list_criticality_path: None,
            //     form_group: Some("Basic"),
            // },
            FieldDef {
                name: "Title",
                label: "{@i18n>title}",
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
                list_sort_order: Some(3),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Tile"),
            },
            FieldDef {
                name: "TileDescription",
                label: "{@i18n>tileDescription}",
                edm_type: "Edm.String",
                max_length: Some(120),
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
                form_group: Some("Tile"),
            },
            FieldDef {
                name: "TileIcon",
                label: "{@i18n>tileIcon}",
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
                list_sort_order: Some(4),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Tile"),
            },
            FieldDef {
                name: "HeaderTypeName",
                label: "{@i18n>headerTypeName}",
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
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Header"),
            },
            FieldDef {
                name: "HeaderTypeNamePlural",
                label: "{@i18n>headerTypeNamePlural}",
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
                list_sort_order: Some(5),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Header"),
            },
            FieldDef {
                name: "HeaderTitlePath",
                label: "{@i18n>headerTitlePath}",
                edm_type: "Edm.Guid",
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: Some("EntityFields"),
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Header"),
            },
            FieldDef {
                name: "HeaderDescriptionPath",
                label: "{@i18n>headerDescriptionPath}",
                edm_type: "Edm.Guid",
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
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Header"),
            },
            FieldDef {
                name: "SelectionFields",
                label: "{@i18n>selectionFields}",
                edm_type: "Edm.String",
                max_length: Some(200),
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
                form_group: Some("Basic"),
            },
            FieldDef {
                name: "DefaultValues",
                label: "{@i18n>defaultValues}",
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
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            },
            FieldDef {
                name: "HeaderFacets",
                label: "{@i18n>headerFacets}",
                edm_type: "Edm.String",
                max_length: Some(2000),
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
                form_group: None,
            },
            FieldDef {
                name: "DataPoints",
                label: "{@i18n>dataPoints}",
                edm_type: "Edm.String",
                max_length: Some(2000),
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
                form_group: None,
            },
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef {
                name: "Fields",
                target_type: "EntityField",
                is_collection: true,
                foreign_key: Some("ConfigID"),
            },
            NavigationPropertyDef {
                name: "Facets",
                target_type: "EntityFacet",
                is_collection: true,
                foreign_key: Some("ConfigID"),
            },
            NavigationPropertyDef {
                name: "Navigations",
                target_type: "EntityNavigation",
                is_collection: true,
                foreign_key: Some("ConfigID"),
            },
            NavigationPropertyDef {
                name: "TableFacets",
                target_type: "EntityTableFacet",
                is_collection: true,
                foreign_key: Some("ConfigID"),
            },
        ];
        NAV
    }

    fn compute_fields(&self, record: &mut Value) {
        if let Some(obj) = record.as_object_mut() {
            // TypeName = HeaderTypeName + "Type"
            if let Some(htn) = obj.get("HeaderTypeName").and_then(|v| v.as_str()) {
                if !htn.is_empty() {
                    obj.insert("TypeName".to_string(), json!(format!("{}Type", htn)));
                }
            }
        }
    }

    fn auto_create_children(&self, parent_record: &mut Value) -> Vec<(String, Value)> {
        let config_id = parent_record
            .get("ID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if config_id.is_empty() {
            return vec![];
        }
        let name_field_id = Uuid::new_v4().to_string();
        // Set HeaderTitlePath to the auto-created Name field
        if let Some(obj) = parent_record.as_object_mut() {
            obj.insert("HeaderTitlePath".to_string(), json!(&name_field_id));
        }
        vec![(
            "EntityFields".to_string(),
            json!({
                "ID": name_field_id,
                "ConfigID": config_id,
                "FieldName": "Name",
                "Label": "Name",
                "EdmType": "Edm.String",
                "MaxLength": 80,
                "IsImmutable": false,
                "IsComputed": false,
                "SortOrder": 0,
                "ShowInLineItem": true,
                "Searchable": true,
                "FormGroup": "General",
            }),
        )]
    }

    fn expand_record(
        &self,
        record: &mut Value,
        nav_properties: &[&str],
        _entities: &[&dyn ODataEntity],
        data_store: &HashMap<String, Vec<Value>>,
    ) {
        let parent_id = record
            .get("ID")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(pid) = parent_id {
            for &(nav, set) in &[
                ("Fields", "EntityFields"),
                ("Facets", "EntityFacets"),
                ("Navigations", "EntityNavigations"),
                ("TableFacets", "EntityTableFacets"),
            ] {
                if nav_properties.contains(&nav) {
                    let children: Vec<Value> = data_store
                        .get(set)
                        .map(|records| {
                            records
                                .iter()
                                .filter(|r| {
                                    r.get("ConfigID").and_then(|v| v.as_str()) == Some(&pid)
                                })
                                .cloned()
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(obj) = record.as_object_mut() {
                        obj.insert(nav.to_string(), Value::Array(children));
                    }
                }
            }
        }
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityConfigs\" EntityType=\"{ns}.EntityConfig\">\n\
             <NavigationPropertyBinding Path=\"Fields\" Target=\"EntityFields\"/>\n\
             <NavigationPropertyBinding Path=\"Facets\" Target=\"EntityFacets\"/>\n\
             <NavigationPropertyBinding Path=\"Navigations\" Target=\"EntityNavigations\"/>\n\
             <NavigationPropertyBinding Path=\"TableFacets\" Target=\"EntityTableFacets\"/>\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityConfigs\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Entity-Konfiguration",
                type_name_plural: "Entity-Konfigurationen",
                title_path: "SetName",
                description_path: "TypeName",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef {
                    label: "Grunddaten",
                    id: "BasicInfo",
                    field_group_qualifier: "Basic",
                    field_group_label: "Grundkonfiguration",
                },
                FacetSectionDef {
                    label: "Kachel",
                    id: "TileInfo",
                    field_group_qualifier: "Tile",
                    field_group_label: "FLP-Kachel",
                },
                FacetSectionDef {
                    label: "Kopfzeile",
                    id: "HeaderInfo",
                    field_group_qualifier: "Header",
                    field_group_label: "Object-Page-Header",
                },
            ],
            table_facets: &[
                TableFacetDef {
                    label: "Felder",
                    id: "FieldsSection",
                    navigation_property: "Fields",
                },
                TableFacetDef {
                    label: "Facetten",
                    id: "FacetsSection",
                    navigation_property: "Facets",
                },
                TableFacetDef {
                    label: "Navigation Props.",
                    id: "NavigationsSection",
                    navigation_property: "Navigations",
                },
                TableFacetDef {
                    label: "Tabellen-Facetten",
                    id: "TableFacetsSection",
                    navigation_property: "TableFacets",
                },
            ],
        };
        Some(&DEF)
    }

    fn manifest_routes(&self) -> Vec<Value> {
        vec![
            json!({ "pattern": "EntityConfigs:?query:", "name": "EntityConfigsList", "target": "EntityConfigsList" }),
            json!({ "pattern": "EntityConfigs({key}):?query:", "name": "EntityConfigsObjectPage", "target": ["EntityConfigsList", "EntityConfigsObjectPage"] }),
            json!({ "pattern": "EntityConfigs({key})/Fields({key2}):?query:", "name": "EntityFieldsObjectPage", "target": ["EntityConfigsList", "EntityConfigsObjectPage", "EntityFieldsObjectPage"] }),
            json!({ "pattern": "EntityConfigs({key})/Facets({key2}):?query:", "name": "EntityFacetsObjectPage", "target": ["EntityConfigsList", "EntityConfigsObjectPage", "EntityFacetsObjectPage"] }),
            json!({ "pattern": "EntityConfigs({key})/Navigations({key2}):?query:", "name": "EntityNavigationsObjectPage", "target": ["EntityConfigsList", "EntityConfigsObjectPage", "EntityNavigationsObjectPage"] }),
            json!({ "pattern": "EntityConfigs({key})/TableFacets({key2}):?query:", "name": "EntityTableFacetsObjectPage", "target": ["EntityConfigsList", "EntityConfigsObjectPage", "EntityTableFacetsObjectPage"] }),
        ]
    }

    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![
            (
                "EntityConfigsList".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityConfigsList",
                    "name": "sap.fe.templates.ListReport",
                    "options": { "settings": {
                        "contextPath": "/EntityConfigs",
                        "variantManagement": "Page",
                        "initialLoad": "Enabled",
                        "navigation": { "EntityConfigs": { "detail": { "route": "EntityConfigsObjectPage" } } }
                    }},
                    "controlAggregation": "beginColumnPages",
                    "contextPattern": ""
                }),
            ),
            (
                "EntityConfigsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityConfigsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": { "settings": {
                        "contextPath": "/EntityConfigs",
                        "navigation": {
                            "Fields": { "detail": { "route": "EntityFieldsObjectPage" } },
                            "Facets": { "detail": { "route": "EntityFacetsObjectPage" } },
                            "Navigations": { "detail": { "route": "EntityNavigationsObjectPage" } },
                            "TableFacets": { "detail": { "route": "EntityTableFacetsObjectPage" } }
                        }
                    } },
                    "controlAggregation": "midColumnPages",
                    "contextPattern": "/EntityConfigs({key})"
                }),
            ),
            (
                "EntityFieldsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityFieldsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": { "settings": { "contextPath": "/EntityConfigs/Fields" } },
                    "controlAggregation": "endColumnPages",
                    "contextPattern": "/EntityConfigs({key})/Fields({key2})"
                }),
            ),
            (
                "EntityFacetsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityFacetsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": { "settings": { "contextPath": "/EntityConfigs/Facets" } },
                    "controlAggregation": "endColumnPages",
                    "contextPattern": "/EntityConfigs({key})/Facets({key2})"
                }),
            ),
            (
                "EntityNavigationsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityNavigationsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": { "settings": { "contextPath": "/EntityConfigs/Navigations" } },
                    "controlAggregation": "endColumnPages",
                    "contextPattern": "/EntityConfigs({key})/Navigations({key2})"
                }),
            ),
            (
                "EntityTableFacetsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "EntityTableFacetsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": { "settings": { "contextPath": "/EntityConfigs/TableFacets" } },
                    "controlAggregation": "endColumnPages",
                    "contextPattern": "/EntityConfigs({key})/TableFacets({key2})"
                }),
            ),
        ]
    }

    fn apps_json_entry(&self) -> Option<(String, Value)> {
        Some((
            "EntityConfigs-display".to_string(),
            json!({
                "title": "Entity-Konfigurationen",
                "description": "Generische Entitaeten verwalten",
                "icon": "sap-icon://settings",
                "semanticObject": "EntityConfigs",
                "action": "display"
            }),
        ))
    }

    fn custom_actions_xml(&self) -> String {
        let fqn = format!("{}.{}", NAMESPACE, self.type_name());
        format!(
            "<Action Name=\"publishConfig\" IsBound=\"true\" EntitySetPath=\"in\">\
             <Parameter Name=\"in\" Type=\"{fqn}\"/>\
             <ReturnType Type=\"{fqn}\"/>\
             </Action>"
        )
    }

    fn extra_annotations_xml(&self) -> String {
        let ns = NAMESPACE;
        let ty = self.type_name();
        // ValueList annotation template for HeaderTitlePath / HeaderDescriptionPath
        let vl_ann = |prop: &str| {
            format!(
                "<Annotations Target=\"{ns}.{ty}/{prop}\">\
                 <Annotation Term=\"Common.ValueList\">\
                 <Record Type=\"Common.ValueListType\">\
                 <PropertyValue Property=\"CollectionPath\" String=\"EntityFields\"/>\
                 <PropertyValue Property=\"Parameters\">\
                 <Collection>\
                 <Record Type=\"Common.ValueListParameterOut\">\
                 <PropertyValue Property=\"LocalDataProperty\" PropertyPath=\"{prop}\"/>\
                 <PropertyValue Property=\"ValueListProperty\" String=\"ID\"/>\
                 </Record>\
                 <Record Type=\"Common.ValueListParameterDisplayOnly\">\
                 <PropertyValue Property=\"ValueListProperty\" String=\"FieldName\"/>\
                 </Record>\
                 <Record Type=\"Common.ValueListParameterIn\">\
                 <PropertyValue Property=\"LocalDataProperty\" PropertyPath=\"ID\"/>\
                 <PropertyValue Property=\"ValueListProperty\" String=\"ConfigID\"/>\
                 </Record>\
                 </Collection>\
                 </PropertyValue>\
                 </Record>\
                 </Annotation>\
                 </Annotations>"
            )
        };
        format!(
            "<Annotations Target=\"{ns}.{ty}\">\
             <Annotation Term=\"UI.Identification\">\
             <Collection>\
             <Record Type=\"UI.DataFieldForAction\">\
             <PropertyValue Property=\"Action\" String=\"{ns}.publishConfig\"/>\
             <PropertyValue Property=\"Label\" String=\"Konfiguration publizieren\"/>\
             </Record>\
             </Collection>\
             </Annotation>\
             </Annotations>\
             {}\
             {}",
            vl_ann("HeaderTitlePath"),
            vl_ann("HeaderDescriptionPath"),
        )
    }
}
