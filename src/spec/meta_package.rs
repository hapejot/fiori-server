//! # Meta Package
//!
//! The configuration layer expressed in its own terms — proving the spec model
//! is self-hosting. These entities configure generic entities at runtime:
//!
//! ```text
//! EntityConfigs (root)
//! ├── Fields      → EntityFields       (composition)
//! │   condition: HeaderTitlePath picks one from Fields
//! │   condition: HeaderDescriptionPath picks one from Fields
//! ├── Facets      → EntityFacets       (composition)
//! ├── Navigations → EntityNavigations  (composition)
//! └── TableFacets → EntityTableFacets  (composition)
//!
//! FieldValueLists (root, has Launchpad tile)
//! └── Items → FieldValueListItems      (composition)
//! ```

use super::{Condition, ConditionalRef, EntitySpec, FieldSpec, Relationship, Side};
use crate::entity::value_list_id;
use crate::spec::AtomValueList;

const PKG: &str = "meta";

fn pkg() -> Option<String> {
    Some(PKG.into())
}

pub fn meta_package() -> (Vec<EntitySpec>, Vec<Relationship>) {
    let rels = meta_relationships();
    let entities = meta_entities();
    (entities, rels)
}

/// Returns just the meta-package relationships (for AppStateBuilder registration).
pub fn meta_relationships() -> Vec<Relationship> {
    vec![
        // Compositions: EntityConfigs owns its child tables
        Relationship {
            name: "EntityConfig_Fields".into(),
            one: Side::new("EntityConfigs", "Fields"),
            many: Side::new("EntityFields", "_Config"),
            owned: true,
            fk_field: Some("ConfigID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: pkg(),
        },
        Relationship {
            name: "EntityConfig_Facets".into(),
            one: Side::new("EntityConfigs", "Facets"),
            many: Side::new("EntityFacets", "_Config"),
            owned: true,
            fk_field: Some("ConfigID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: pkg(),
        },
        Relationship {
            name: "EntityConfig_Navigations".into(),
            one: Side::new("EntityConfigs", "Navigations"),
            many: Side::new("EntityNavigations", "_Config"),
            owned: true,
            fk_field: Some("ConfigID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: pkg(),
        },
        Relationship {
            name: "EntityConfig_TableFacets".into(),
            one: Side::new("EntityConfigs", "TableFacets"),
            many: Side::new("EntityTableFacets", "_Config"),
            owned: true,
            fk_field: Some("ConfigID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: pkg(),
        },
        // Composition: FieldValueLists owns its items
        Relationship {
            name: "FieldValueList_Items".into(),
            one: Side::new("FieldValueLists", "Items"),
            many: Side::new("FieldValueListItems", "_List"),
            owned: true,
            fk_field: Some("ListID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: pkg(),
        },
        // Conditional: HeaderTitlePath picks one from Fields
        Relationship {
            name: "HeaderTitle".into(),
            one: Side::new("EntityFields", "_HeaderTitleField"),
            many: Side::new("EntityConfigs", "_HeaderTitlePath"),
            owned: false,
            fk_field: Some("HeaderTitlePath".into()),
            fk_label: Some("Header Title Field".into()),
            fk_form_group: Some("Header".into()),
            condition: Some(ConditionalRef {
                condition: Condition::SubsetOf,
                reference: "EntityConfig_Fields".into(),
            }),
            package: pkg(),
        },
        // Conditional: HeaderDescriptionPath picks one from Fields
        Relationship {
            name: "HeaderDescription".into(),
            one: Side::new("EntityFields", "_HeaderDescField"),
            many: Side::new("EntityConfigs", "_HeaderDescPath"),
            owned: false,
            fk_field: Some("HeaderDescriptionPath".into()),
            fk_label: Some("Header Description Field".into()),
            fk_form_group: Some("Header".into()),
            condition: Some(ConditionalRef {
                condition: Condition::SubsetOf,
                reference: "EntityConfig_Fields".into(),
            }),
            package: pkg(),
        },
        // Reference: EntityFields → FieldValueLists
        Relationship {
            name: "EntityField_ValueList".into(),
            one: Side::new("FieldValueLists", "_ReferencingFields"),
            many: Side::new("EntityFields", "_ValueList"),
            owned: false,
            fk_field: Some("ValueSource".into()),
            fk_label: Some("Value Source".into()),
            fk_form_group: Some("FieldProps".into()),
            condition: None,
            package: pkg(),
        },
    ]
}

// ─── Entities ────────────────────────────────────────────────────────────────

pub fn meta_entities() -> Vec<EntitySpec> {
    vec![
        entity_configs(),
        entity_fields(),
        entity_facets(),
        entity_navigations(),
        entity_table_facets(),
        field_value_lists(),
        field_value_list_items(),
    ]
}

pub fn entity_configs() -> EntitySpec {
    EntitySpec {
        set_name: "EntityConfigs".into(),
        package: pkg(),
        type_name: Some("EntityConfig".into()),
        type_name_plural: None,
        title_field: Some("SetName".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("SetName", "Set Name", 40)
                .form_group("Basic")
                .searchable()
                .show_in_list(),
            FieldSpec::string("TypeName", "Type Name", 40)
                .computed()
                .form_group("Basic"),
            FieldSpec::string("Title", "Title", 80).form_group("Tile"),
            FieldSpec::string("TileDescription", "Tile Description", 120).form_group("Tile"),
            FieldSpec::string("TileIcon", "Tile Icon", 80).form_group("Tile"),
            FieldSpec::string("HeaderTypeName", "Header Type Name", 40).form_group("Header"),
            FieldSpec::string("HeaderTypeNamePlural", "Header Type Name Plural", 40)
                .form_group("Header"),
            // HeaderTitlePath and HeaderDescriptionPath are FK fields auto-derived
            // from the conditional relationships "HeaderTitle" and "HeaderDescription".
            FieldSpec::string("SelectionFields", "Selection Fields", 200).form_group("Basic"),
            FieldSpec::string("DefaultValues", "Default Values", 500),
            FieldSpec::string("HeaderFacets", "Header Facets", 2000),
            FieldSpec::string("DataPoints", "Data Points", 2000),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn entity_fields() -> EntitySpec {
    EntitySpec {
        set_name: "EntityFields".into(),
        package: pkg(),
        type_name: Some("EntityField".into()),
        type_name_plural: None,
        title_field: Some("FieldName".into()),
        description_field: None,
        fields: vec![
            // ConfigID is a composition FK — auto-generated by EntityConfig_Fields relationship.
            FieldSpec::string("SetName", "Set Name", 40)
                .computed()
                .form_group("FieldProps"),
            FieldSpec::string("FieldName", "Field Name", 40)
                .form_group("FieldProps")
                .show_in_list(),
            FieldSpec::string("Label", "Label", 80).form_group("FieldProps"),
            FieldSpec::string("EdmType", "EDM Type", 30)
                .form_group("FieldProps")
                .with_value_list(AtomValueList::FieldValueList {
                    list_id: value_list_id("EdmTypes"),
                    prefer_dialog: false,
                }),
            FieldSpec::int("MaxLength", "Max Length").form_group("FieldProps"),
            FieldSpec::int("Precision", "Precision").form_group("FieldProps"),
            FieldSpec::int("Scale", "Scale").form_group("FieldProps"),
            FieldSpec::bool_field("IsImmutable", "Immutable").form_group("FieldProps"),
            FieldSpec::bool_field("IsComputed", "Computed").form_group("FieldProps"),
            FieldSpec::string("ReferencesEntity", "References Entity", 40)
                .form_group("FieldProps"),
            FieldSpec::bool_field("PreferDialog", "Prefer Dialog").form_group("FieldProps"),
            // ValueSource is a FK — auto-generated by the EntityField_ValueList relationship.
            FieldSpec::string("TextPath", "Text Path", 80).form_group("FieldProps"),
            FieldSpec::string("DefaultValue", "Default Value", 120).form_group("FieldProps"),
            FieldSpec::int("SortOrder", "Sort Order").form_group("FieldProps"),
            FieldSpec::bool_field("ShowInLineItem", "Show in Line Item")
                .form_group("LineItemProps"),
            FieldSpec::string("LineItemImportance", "Line Item Importance", 10)
                .form_group("LineItemProps"),
            FieldSpec::string("LineItemLabel", "Line Item Label", 80)
                .form_group("LineItemProps"),
            FieldSpec::string("LineItemCriticalityPath", "Criticality Path", 40)
                .form_group("LineItemProps"),
            FieldSpec::string("LineItemSemanticObject", "Semantic Object", 40)
                .form_group("LineItemProps"),
            FieldSpec::bool_field("Searchable", "Searchable").form_group("FieldProps"),
            FieldSpec::string("FormGroup", "Form Group", 40)
                .form_group("FieldProps")
                .with_value_list(AtomValueList::from_siblings(
                    "EntityFacets",
                    "ConfigID",
                    "FieldGroupQualifier",
                    Some("FieldGroupLabel"),
                )),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn entity_facets() -> EntitySpec {
    EntitySpec {
        set_name: "EntityFacets".into(),
        package: pkg(),
        type_name: Some("EntityFacet".into()),
        type_name_plural: None,
        title_field: Some("SectionLabel".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("SectionLabel", "Section Label", 80).form_group("FacetInfo"),
            FieldSpec::string("SectionId", "Section ID", 40).form_group("FacetInfo"),
            FieldSpec::string("FieldGroupQualifier", "Field Group Qualifier", 40)
                .form_group("FacetInfo"),
            FieldSpec::string("FieldGroupLabel", "Field Group Label", 80)
                .form_group("FacetInfo"),
            FieldSpec::string("FieldGroupFields", "Field Group Fields", 500)
                .form_group("FacetInfo"),
            FieldSpec::int("SortOrder", "Sort Order").form_group("FacetInfo"),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn entity_navigations() -> EntitySpec {
    EntitySpec {
        set_name: "EntityNavigations".into(),
        package: pkg(),
        type_name: Some("EntityNavigation".into()),
        type_name_plural: None,
        title_field: Some("NavName".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("SetName", "Set Name", 40)
                .computed()
                .form_group("NavInfo"),
            FieldSpec::string("NavName", "Navigation Name", 40).form_group("NavInfo"),
            FieldSpec::string("TargetType", "Target Type", 40).form_group("NavInfo"),
            FieldSpec::string("TargetSet", "Target Set", 40).form_group("NavInfo"),
            FieldSpec::bool_field("IsCollection", "Is Collection").form_group("NavInfo"),
            FieldSpec::string("ForeignKey", "Foreign Key", 40).form_group("NavInfo"),
            FieldSpec::int("SortOrder", "Sort Order").form_group("NavInfo"),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn entity_table_facets() -> EntitySpec {
    EntitySpec {
        set_name: "EntityTableFacets".into(),
        package: pkg(),
        type_name: Some("EntityTableFacet".into()),
        type_name_plural: None,
        title_field: Some("FacetLabel".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("SetName", "Set Name", 40)
                .computed()
                .form_group("TFInfo"),
            FieldSpec::string("FacetLabel", "Facet Label", 80).form_group("TFInfo"),
            FieldSpec::string("FacetId", "Facet ID", 40).form_group("TFInfo"),
            FieldSpec::string("NavigationProperty", "Navigation Property", 40)
                .form_group("TFInfo"),
            FieldSpec::int("SortOrder", "Sort Order").form_group("TFInfo"),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn field_value_lists() -> EntitySpec {
    EntitySpec {
        set_name: "FieldValueLists".into(),
        package: pkg(),
        type_name: Some("FieldValueList".into()),
        type_name_plural: None,
        title_field: Some("ListName".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("ListName", "List Name", 40).form_group("ListInfo"),
            FieldSpec::string("Description", "Description", 120).form_group("ListInfo"),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

pub fn field_value_list_items() -> EntitySpec {
    EntitySpec {
        set_name: "FieldValueListItems".into(),
        package: pkg(),
        type_name: Some("FieldValueListItem".into()),
        type_name_plural: None,
        title_field: Some("Code".into()),
        description_field: None,
        fields: vec![
            // ListID is a composition FK — auto-generated by FieldValueList_Items relationship.
            FieldSpec::string("Code", "Code", 40).form_group("ItemInfo"),
            FieldSpec::string("Description", "Description", 120).form_group("ItemInfo"),
            FieldSpec::int("SortOrder", "Sort Order").form_group("ItemInfo"),
        ],
        data_points: vec![],
        header_facets: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn test_meta_package_resolves() {
        let (specs, rels) = meta_package();
        assert_eq!(specs.len(), 7);
        assert_eq!(rels.len(), 8);

        let resolved = model::resolve(&specs, &rels);
        assert_eq!(resolved.len(), 7);

        // EntityConfigs: root entity
        let configs = resolved.iter().find(|e| e.set_name == "EntityConfigs").unwrap();
        assert!(configs.parent_set_name.is_none());
        assert_eq!(configs.title_field, "SetName");
        let config_navs: Vec<&str> = configs.nav_properties.iter().map(|n| n.name.as_str()).collect();
        assert!(config_navs.contains(&"Fields"));
        assert!(config_navs.contains(&"Facets"));
        assert!(config_navs.contains(&"Navigations"));
        assert!(config_navs.contains(&"TableFacets"));
        assert!(config_navs.contains(&"_HeaderTitlePath"));
        assert!(config_navs.contains(&"_HeaderDescPath"));
        let config_props: Vec<&str> = configs.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(config_props.contains(&"HeaderTitlePath"));
        assert!(config_props.contains(&"HeaderDescriptionPath"));
        assert_eq!(configs.table_facets.len(), 4);

        // EntityFields: child of EntityConfigs
        let fields = resolved.iter().find(|e| e.set_name == "EntityFields").unwrap();
        assert_eq!(fields.parent_set_name.as_deref(), Some("EntityConfigs"));
        let field_props: Vec<&str> = fields.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(field_props.contains(&"ConfigID"));
        assert!(field_props.contains(&"ValueSource"));
        let vs = fields.properties.iter().find(|p| p.name == "ValueSource").unwrap();
        assert!(vs.value_list.is_some());

        // FieldValueLists: root with Items composition
        let lists = resolved.iter().find(|e| e.set_name == "FieldValueLists").unwrap();
        assert!(lists.parent_set_name.is_none());
        assert!(lists.nav_properties.iter().any(|n| n.name == "Items"));

        // FieldValueListItems: child of FieldValueLists
        let items = resolved.iter().find(|e| e.set_name == "FieldValueListItems").unwrap();
        assert_eq!(items.parent_set_name.as_deref(), Some("FieldValueLists"));
        assert!(items.properties.iter().any(|p| p.name == "ListID"));
    }

    #[test]
    fn test_meta_package_xml_generation() {
        let (specs, rels) = meta_package();
        let resolved = model::resolve(&specs, &rels);

        for entity in &resolved {
            let et_xml = crate::odata::entity_type::generate_entity_type(entity);
            assert!(et_xml.contains(&format!("EntityType Name=\"{}\"", entity.type_name)));

            let es_xml = crate::odata::entity_type::generate_entity_set(entity);
            assert!(es_xml.contains(&format!("EntitySet Name=\"{}\"", entity.set_name)));

            let ann_xml = crate::odata::xml_types::anns_to_xml(
                &crate::odata::annotations_gen::generate_annotations(entity),
            );
            assert!(ann_xml.contains("UI.SelectionFields"));
            assert!(ann_xml.contains("UI.LineItem"));
            assert!(ann_xml.contains("UI.HeaderInfo"));
        }
    }
}
