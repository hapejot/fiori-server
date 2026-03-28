use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityFieldEntity;

impl ODataEntity for EntityFieldEntity {
    fn set_name(&self) -> &'static str {
        "EntityFields"
    }
    fn key_field(&self) -> &'static str {
        "FieldID"
    }
    fn type_name(&self) -> &'static str {
        "EntityField"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("EntityConfigs")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "FieldID",              label: "Feld-ID",            edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "SetName",              label: "EntitySet",           edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "FieldName",            label: "Feldname",            edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Label",                label: "Bezeichnung",         edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "EdmType",              label: "Datentyp",            edm_type: "Edm.String",  max_length: Some(30),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "MaxLength",            label: "Max. Laenge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Precision",            label: "Praezision",          edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Scale",                label: "Dezimalstellen",      edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "IsImmutable",          label: "Unveraenderlich",     edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "SemanticObject",       label: "Semantic Object",     edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "SortOrder",            label: "Reihenfolge",         edm_type: "Edm.Int32",   max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "ShowInLineItem",       label: "In Liste",            edm_type: "Edm.Boolean", max_length: None,      precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "LineItemImportance",   label: "Wichtigkeit",         edm_type: "Edm.String",  max_length: Some(10),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "LineItemLabel",        label: "Listen-Label",        edm_type: "Edm.String",  max_length: Some(80),  precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "LineItemCriticalityPath", label: "Kritikalitaets-Pfad", edm_type: "Edm.String", max_length: Some(40), precision: None, scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "LineItemSemanticObject", label: "Listen-Sem.Object", edm_type: "Edm.String",  max_length: Some(40),  precision: None, scale: None, immutable: false, semantic_object: None },
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityFields\" EntityType=\"{ns}.EntityField\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityFields\"/>\n\
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
                LineItemField { name: "FieldName",     label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Label",         label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "EdmType",       label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "MaxLength",     label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "IsImmutable",   label: None, importance: None,         criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "ShowInLineItem", label: None, importance: None,        criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Felddefinition",
                type_name_plural: "Felddefinitionen",
                title_path: "FieldName",
                description_path: "Label",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef { label: "Feldeigenschaften",    id: "FieldProps",   field_group_qualifier: "FieldProps",   field_group_label: "Eigenschaften" },
                FacetSectionDef { label: "Listen-Konfiguration", id: "LineItemProps", field_group_qualifier: "LineItemProps", field_group_label: "LineItem" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "FieldProps",    fields: &["FieldID", "SetName", "FieldName", "Label", "EdmType", "MaxLength", "Precision", "Scale", "IsImmutable", "SemanticObject", "SortOrder"] },
                FieldGroupDef { qualifier: "LineItemProps", fields: &["ShowInLineItem", "LineItemImportance", "LineItemLabel", "LineItemCriticalityPath", "LineItemSemanticObject"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        ("_EntityFields-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
