use std::collections::HashMap;

use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::{value_list_id, ODataEntity};
use crate::NAMESPACE;

#[derive(Debug)]
pub struct FieldValueListEntity;

impl ODataEntity for FieldValueListEntity {
    fn set_name(&self) -> &'static str {
        "FieldValueLists"
    }
    fn type_name(&self) -> &'static str {
        "FieldValueList"
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![
            json!({ "ID": value_list_id("EdmTypes"), "ListName": "EdmTypes", "Description": "OData EDM Datentypen" }),
        ]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ID",          label: "ID",            edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,  references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None},
            FieldDef { name: "ListName",    label: "Listenname",    edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(0), list_importance: Some("High"), list_criticality_path: None, form_group: Some("ListInfo")},
            FieldDef { name: "Description", label: "Beschreibung",  edm_type: "Edm.String", max_length: Some(120), precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None , prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(1), list_importance: None, list_criticality_path: None, form_group: Some("ListInfo")},
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef { name: "Items", target_type: "FieldValueListItem", is_collection: true, foreign_key: Some("ListID") },
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
        let list_id = record.get("ID").and_then(|v| v.as_str()).map(|s| s.to_string());
        if let Some(lid) = list_id {
            if nav_properties.contains(&"Items") {
                let children: Vec<Value> = data_store
                    .get("FieldValueListItems")
                    .map(|records| {
                        records
                            .iter()
                            .filter(|r| r.get("ListID").and_then(|v| v.as_str()) == Some(&lid))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("Items".to_string(), Value::Array(children));
                }
            }
        }
    }


    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"FieldValueLists\" EntityType=\"{ns}.FieldValueList\">\n\
             <NavigationPropertyBinding Path=\"Items\" Target=\"FieldValueListItems\"/>\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"FieldValueLists\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Werteliste",
                type_name_plural: "Wertelisten",
                title_path: "ListName",
                description_path: "Description",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Listen-Details", id: "ListDetails", field_group_qualifier: "ListInfo", field_group_label: "Informationen" },
                FacetSectionDef { label: "Eintraege",      id: "ListItems",   field_group_qualifier: "",         field_group_label: ""             },
            ],
            table_facets: &[
                TableFacetDef { label: "Eintraege", id: "ListItems", navigation_property: "Items" },
            ],
        };
        Some(&DEF)
    }

    fn manifest_routes(&self) -> Vec<Value> {
        vec![
            json!({ "pattern": "FieldValueLists:?query:", "name": "FieldValueListsList", "target": "FieldValueListsList" }),
            json!({ "pattern": "FieldValueLists({key}):?query:", "name": "FieldValueListsObjectPage", "target": ["FieldValueListsList", "FieldValueListsObjectPage"] }),
            json!({ "pattern": "FieldValueLists({key})/Items({key2}):?query:", "name": "FieldValueListItemsObjectPage", "target": ["FieldValueListsList", "FieldValueListsObjectPage", "FieldValueListItemsObjectPage"] }),
        ]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![
            ("FieldValueListsList".to_string(), json!({
                "type": "Component",
                "id": "FieldValueListsList",
                "name": "sap.fe.templates.ListReport",
                "options": { "settings": {
                    "contextPath": "/FieldValueLists",
                    "variantManagement": "Page",
                    "initialLoad": "Enabled",
                    "navigation": { "FieldValueLists": { "detail": { "route": "FieldValueListsObjectPage" } } }
                }},
                "controlAggregation": "beginColumnPages",
                "contextPattern": ""
            })),
            ("FieldValueListsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "FieldValueListsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/FieldValueLists" } },
                "controlAggregation": "midColumnPages",
                "contextPattern": "/FieldValueLists({key})"
            })),
            ("FieldValueListItemsObjectPage".to_string(), json!({
                "type": "Component",
                "id": "FieldValueListItemsObjectPage",
                "name": "sap.fe.templates.ObjectPage",
                "options": { "settings": { "contextPath": "/FieldValueLists/Items" } },
                "controlAggregation": "endColumnPages",
                "contextPattern": "/FieldValueLists({key})/Items({key2})"
            })),
        ]
    }

    fn apps_json_entry(&self) -> Option<(String, Value)> {
        Some((
            "FieldValueLists-display".to_string(),
            json!({
                "title": "Wertelisten",
                "description": "Feste Wertelisten verwalten",
                "icon": "sap-icon://value-help",
                "semanticObject": "FieldValueLists",
                "action": "display"
            }),
        ))
    }
}
