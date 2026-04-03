use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::{value_list_id, ODataEntity};
use crate::NAMESPACE;

#[derive(Debug)]
pub struct FieldValueListItemEntity;

impl ODataEntity for FieldValueListItemEntity {
    fn set_name(&self) -> &'static str {
        "FieldValueListItems"
    }
    fn key_field(&self) -> &'static str {
        "ID"
    }
    fn type_name(&self) -> &'static str {
        "FieldValueListItem"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("FieldValueLists")
    }

    fn mock_data(&self) -> Vec<Value> {
        let edm_id = value_list_id("EdmTypes");
        vec![
            json!({ "ID": value_list_id("EdmTypes_Edm.String"),         "ListID": edm_id, "Code": "Edm.String",         "Description": "Zeichenkette",      "SortOrder": 0 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Int32"),          "ListID": edm_id, "Code": "Edm.Int32",           "Description": "Ganzzahl",           "SortOrder": 1 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Int64"),          "ListID": edm_id, "Code": "Edm.Int64",           "Description": "Lange Ganzzahl",     "SortOrder": 2 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Decimal"),        "ListID": edm_id, "Code": "Edm.Decimal",         "Description": "Dezimalzahl",        "SortOrder": 3 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Boolean"),        "ListID": edm_id, "Code": "Edm.Boolean",         "Description": "Wahrheitswert",      "SortOrder": 4 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.DateTimeOffset"), "ListID": edm_id, "Code": "Edm.DateTimeOffset",  "Description": "Datum und Uhrzeit",  "SortOrder": 5 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Date"),           "ListID": edm_id, "Code": "Edm.Date",            "Description": "Datum",              "SortOrder": 6 }),
            json!({ "ID": value_list_id("EdmTypes_Edm.Guid"),           "ListID": edm_id, "Code": "Edm.Guid",            "Description": "GUID",               "SortOrder": 7 }),
        ]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ID",          label: "ID",              edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: true,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "ListID",      label: "Listen-ID",       edm_type: "Edm.Guid",   max_length: None,      precision: None, scale: None, immutable: true, computed: false,  semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "Code",        label: "Code",            edm_type: "Edm.String", max_length: Some(40),  precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "Description", label: "Beschreibung",    edm_type: "Edm.String", max_length: Some(120), precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
            FieldDef { name: "SortOrder",   label: "Reihenfolge",     edm_type: "Edm.Int32",  max_length: None,      precision: None, scale: None, immutable: false, computed: false, semantic_object: None, value_source: None , value_list: None, text_path: None},
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"FieldValueListItems\" EntityType=\"{ns}.FieldValueListItem\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"FieldValueListItems\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "SortOrder",   label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Code",        label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Description", label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Wertlisten-Eintrag",
                type_name_plural: "Wertlisten-Eintraege",
                title_path: "Code",
                description_path: "Description",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Eintrag-Details", id: "ItemDetails", field_group_qualifier: "ItemInfo", field_group_label: "Informationen" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "ItemInfo", fields: &["Code", "Description", "SortOrder"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_FieldValueListItems-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
