use std::collections::HashMap;

use serde_json::{json, Value};

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
        "SetName"
    }
    fn type_name(&self) -> &'static str {
        "EntityConfig"
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "SetName",              label: "EntitySet",          edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "KeyField",             label: "Schluesselfeld",     edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "TypeName",             label: "Entity-Typ",         edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "ParentSetName",        label: "Eltern-EntitySet",   edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "TileTitle",            label: "Kachel-Titel",       edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "TileDescription",      label: "Kachel-Beschreibung",edm_type: "Edm.String", max_length: Some(120), precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "TileIcon",             label: "Kachel-Icon",        edm_type: "Edm.String", max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "HeaderTypeName",       label: "Typ-Name",           edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "HeaderTypeNamePlural",  label: "Typ-Name Plural",   edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "HeaderTitlePath",      label: "Titel-Pfad",         edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "HeaderDescriptionPath", label: "Beschreibungs-Pfad",edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SelectionFields",      label: "Suchfelder",         edm_type: "Edm.String", max_length: Some(200), precision: None, scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef { name: "Fields",       target_type: "EntityField",       is_collection: true, foreign_key: None },
            NavigationPropertyDef { name: "Facets",       target_type: "EntityFacet",       is_collection: true, foreign_key: None },
            NavigationPropertyDef { name: "Navigations",  target_type: "EntityNavigation",  is_collection: true, foreign_key: None },
            NavigationPropertyDef { name: "TableFacets",  target_type: "EntityTableFacet",  is_collection: true, foreign_key: None },
        ];
        NAV
    }

    fn expand_record(
        &self,
        record: &mut Value,
        nav_properties: &[&str],
        _entities: &[&dyn ODataEntity],
        data_store: &HashMap<String, Vec<Value>>,
    ) {
        let set_name = record.get("SetName").and_then(|v| v.as_str()).map(|s| s.to_string());
        if let Some(sn) = set_name {
            if nav_properties.contains(&"Fields") {
                let children: Vec<Value> = data_store
                    .get("EntityFields")
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| r.get("SetName").and_then(|v| v.as_str()) == Some(&sn))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("Fields".to_string(), Value::Array(children));
                }
            }
            if nav_properties.contains(&"Facets") {
                let children: Vec<Value> = data_store
                    .get("EntityFacets")
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| r.get("SetName").and_then(|v| v.as_str()) == Some(&sn))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("Facets".to_string(), Value::Array(children));
                }
            }
            if nav_properties.contains(&"Navigations") {
                let children: Vec<Value> = data_store
                    .get("EntityNavigations")
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| r.get("SetName").and_then(|v| v.as_str()) == Some(&sn))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("Navigations".to_string(), Value::Array(children));
                }
            }
            if nav_properties.contains(&"TableFacets") {
                let children: Vec<Value> = data_store
                    .get("EntityTableFacets")
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| r.get("SetName").and_then(|v| v.as_str()) == Some(&sn))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("TableFacets".to_string(), Value::Array(children));
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
            selection_fields: &["SetName", "TypeName"],
            line_item: &[
                LineItemField { name: "SetName",              label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "TypeName",             label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "KeyField",             label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "TileTitle",            label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "TileIcon",             label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "HeaderTypeNamePlural", label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Entity-Konfiguration",
                type_name_plural: "Entity-Konfigurationen",
                title_path: "SetName",
                description_path: "TypeName",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Grunddaten",       id: "BasicInfo",  field_group_qualifier: "Basic",  field_group_label: "Grundkonfiguration" },
                FacetSectionDef { label: "Kachel",           id: "TileInfo",   field_group_qualifier: "Tile",   field_group_label: "FLP-Kachel" },
                FacetSectionDef { label: "Kopfzeile",        id: "HeaderInfo", field_group_qualifier: "Header", field_group_label: "Object-Page-Header" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "Basic",  fields: &["SetName", "KeyField", "TypeName", "ParentSetName", "SelectionFields"] },
                FieldGroupDef { qualifier: "Tile",   fields: &["TileTitle", "TileDescription", "TileIcon"] },
                FieldGroupDef { qualifier: "Header", fields: &["HeaderTypeName", "HeaderTypeNamePlural", "HeaderTitlePath", "HeaderDescriptionPath"] },
            ],
            table_facets: &[
                TableFacetDef { label: "Felder",             id: "FieldsSection",       navigation_property: "Fields" },
                TableFacetDef { label: "Facetten",           id: "FacetsSection",       navigation_property: "Facets" },
                TableFacetDef { label: "Navigation Props.",  id: "NavigationsSection",  navigation_property: "Navigations" },
                TableFacetDef { label: "Tabellen-Facetten",  id: "TableFacetsSection",  navigation_property: "TableFacets" },
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
            ("EntityConfigsList".to_string(), json!({
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
            })),
            ("EntityConfigsObjectPage".to_string(), json!({
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
            })),
            ("EntityFieldsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "EntityFieldsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/EntityConfigs/Fields" } },
                "controlAggregation": "endColumnPages",
                "contextPattern": "/EntityConfigs({key})/Fields({key2})"
            })),
            ("EntityFacetsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "EntityFacetsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/EntityConfigs/Facets" } },
                "controlAggregation": "endColumnPages",
                "contextPattern": "/EntityConfigs({key})/Facets({key2})"
            })),
            ("EntityNavigationsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "EntityNavigationsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/EntityConfigs/Navigations" } },
                "controlAggregation": "endColumnPages",
                "contextPattern": "/EntityConfigs({key})/Navigations({key2})"
            })),
            ("EntityTableFacetsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "EntityTableFacetsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/EntityConfigs/TableFacets" } },
                "controlAggregation": "endColumnPages",
                "contextPattern": "/EntityConfigs({key})/TableFacets({key2})"
            })),
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
             </Annotations>",
            ns = NAMESPACE,
            ty = self.type_name()
        )
    }
}
