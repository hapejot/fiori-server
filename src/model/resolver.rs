//! Layer 2: Resolver — transforms Layer 1 specs into resolved entities.
//!
//! `resolve(entities, relationships) → Vec<ResolvedEntity>`
//!
//! Steps:
//! 1. Collect all entity names from relationships + explicit specs
//! 2. Build base entities from specs (or auto-create with ID + Name)
//! 3. For each relationship: inject FK property, nav properties, value list,
//!    text_path on BOTH sides
//! 4. For conditional relationships: inject FK with filtered value list
//! 5. Apply smart defaults (selection_fields, facet_sections from form_groups)

use std::collections::HashMap;

use super::defaults;
use super::resolved::*;
use crate::spec::{
    AtomValueList, Condition, EntitySpec, FieldSpec, Relationship, ValueListFilter,
};

/// Resolve a set of entity specs and relationships into fully resolved entities.
pub fn resolve(
    specs: &[EntitySpec],
    relationships: &[Relationship],
) -> Vec<ResolvedEntity> {
    let spec_map: HashMap<&str, &EntitySpec> =
        specs.iter().map(|s| (s.set_name.as_str(), s)).collect();

    let rel_map: HashMap<&str, &Relationship> =
        relationships.iter().map(|r| (r.name.as_str(), r)).collect();

    // 1. Collect all entity set names from relationships + specs
    let mut entity_names: Vec<String> = Vec::new();
    for r in relationships {
        for name in [&r.one.entity, &r.many.entity] {
            if !entity_names.contains(name) {
                entity_names.push(name.clone());
            }
        }
    }
    for s in specs {
        if !entity_names.contains(&s.set_name) {
            entity_names.push(s.set_name.clone());
        }
    }

    // 2. Build base resolved entities
    let mut entities: HashMap<String, ResolvedEntity> = HashMap::new();
    for name in &entity_names {
        let entity = if let Some(spec) = spec_map.get(name.as_str()) {
            base_from_spec(spec)
        } else {
            auto_entity(name)
        };
        entities.insert(name.clone(), entity);
    }

    // 3. Process relationships
    for rel in relationships {
        apply_relationship(rel, &rel_map, &spec_map, &mut entities);
    }

    // 4. Apply smart defaults
    for entity in entities.values_mut() {
        defaults::apply_defaults(entity);
    }

    // Return in stable order (same as entity_names)
    entity_names
        .iter()
        .filter_map(|n| entities.remove(n))
        .collect()
}

/// Build a base ResolvedEntity from an EntitySpec (own fields only, no FK/nav yet).
fn base_from_spec(spec: &EntitySpec) -> ResolvedEntity {
    let has_key = spec.fields.iter().any(|f| f.name() == "ID");
    let mut properties: Vec<ResolvedProperty> = if has_key {
        vec![]
    } else {
        vec![key_property()]
    };
    properties.extend(spec.fields.iter().map(|f| resolve_field(f)));

    let data_points = spec
        .data_points
        .iter()
        .map(|dp| ResolvedDataPoint {
            qualifier: dp.qualifier.to_string(),
            value_path: dp.value_path.to_string(),
            title: dp.title.to_string(),
            max_value: dp.max_value,
            visualization: dp.visualization.map(|v| v.to_string()),
        })
        .collect();

    let header_facets = spec
        .header_facets
        .iter()
        .map(|hf| ResolvedHeaderFacet {
            data_point_qualifier: hf.data_point_qualifier.to_string(),
            label: hf.label.to_string(),
        })
        .collect();

    ResolvedEntity {
        set_name: spec.set_name.clone(),
        type_name: spec.resolved_type_name(),
        type_name_plural: spec.resolved_type_name_plural(),
        key_field: "ID".into(),
        title_field: spec.resolved_title_field(),
        description_field: spec.description_field.clone(),
        parent_set_name: None,
        properties,
        nav_properties: vec![],
        data_points,
        header_facets,
        facet_sections: vec![],
        table_facets: vec![],
        selection_fields: vec![],
        package: spec.package.clone(),
    }
}

/// Auto-create an entity with just ID + Name when introduced by a relationship.
fn auto_entity(set_name: &str) -> ResolvedEntity {
    let type_name = if set_name.ends_with('s') && set_name.len() > 1 {
        set_name[..set_name.len() - 1].to_string()
    } else {
        set_name.to_string()
    };

    ResolvedEntity {
        set_name: set_name.into(),
        type_name_plural: set_name.into(),
        type_name,
        key_field: "ID".into(),
        title_field: "Name".into(),
        description_field: None,
        parent_set_name: None,
        properties: vec![
            key_property(),
            ResolvedProperty {
                name: "Name".into(),
                edm_type: "Edm.String".into(),
                label: "Name".into(),
                max_length: Some(80),
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    searchable: true,
                    show_in_list: true,
                    ..Default::default()
                },
                package: None,
            },
        ],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
    }
}

/// Create the standard key property (ID, Edm.Guid, computed + hidden).
fn key_property() -> ResolvedProperty {
    ResolvedProperty {
        name: "ID".into(),
        edm_type: "Edm.Guid".into(),
        label: "ID".into(),
        max_length: None,
        precision: None,
        scale: None,
        computed: true,
        immutable: false,
        hidden: true,
        text_path: None,
        value_list: None,
        measure: None,
        presentation: ResolvedPresentation::default(),
        package: None,
    }
}

/// Convert a FieldSpec into a ResolvedProperty.
fn resolve_field(field: &FieldSpec) -> ResolvedProperty {
    match field {
        FieldSpec::Atom {
            name,
            label,
            edm_type,
            package,
            max_length,
            precision,
            scale,
            computed,
            immutable,
            value_list,
            presentation,
        } => ResolvedProperty {
            name: name.clone(),
            edm_type: edm_type.clone(),
            label: label.clone(),
            max_length: *max_length,
            precision: *precision,
            scale: *scale,
            computed: *computed,
            immutable: *immutable,
            hidden: edm_type == "Edm.Guid",
            text_path: None,
            value_list: value_list.as_ref().map(resolve_value_list),
            measure: None,
            presentation: resolve_presentation(presentation),
            package: package.clone(),
        },
        FieldSpec::Measure {
            name,
            label,
            package,
            precision,
            scale,
            unit_field,
            kind,
            presentation,
        } => ResolvedProperty {
            name: name.clone(),
            edm_type: "Edm.Decimal".into(),
            label: label.clone(),
            max_length: None,
            precision: *precision,
            scale: *scale,
            computed: false,
            immutable: false,
            hidden: false,
            text_path: None,
            value_list: None,
            measure: Some(ResolvedMeasure {
                unit_field: unit_field.clone(),
                kind: kind.clone(),
            }),
            presentation: resolve_presentation(presentation),
            package: package.clone(),
        },
    }
}

fn resolve_presentation(p: &crate::spec::PresentationOverrides) -> ResolvedPresentation {
    ResolvedPresentation {
        searchable: p.searchable.unwrap_or(false),
        show_in_list: p.show_in_list.unwrap_or(false),
        list_sort_order: p.list_sort_order,
        list_importance: p.list_importance.clone(),
        criticality_path: p.criticality_path.clone(),
        form_group: p.form_group.clone(),
    }
}

fn resolve_value_list(vl: &AtomValueList) -> ResolvedValueList {
    match vl {
        AtomValueList::FieldValueList {
            list_id,
            prefer_dialog,
        } => ResolvedValueList::CodeList {
            list_id: list_id.clone(),
            fixed_values: !prefer_dialog,
        },
        AtomValueList::EntityRef {
            entity_set,
            key_property,
            display_property,
            filters,
            prefer_dialog,
        } => ResolvedValueList::EntityRef {
            collection_path: entity_set.clone(),
            key_property: key_property.clone(),
            display_property: display_property.clone(),
            filters: filters.clone(),
            fixed_values: !prefer_dialog,
        },
    }
}

/// Apply a single relationship to the entity map — injects FK, navs, text_path,
/// value list, table facets on both sides.
fn apply_relationship(
    rel: &Relationship,
    rel_map: &HashMap<&str, &Relationship>,
    _spec_map: &HashMap<&str, &EntitySpec>,
    entities: &mut HashMap<String, ResolvedEntity>,
) {
    let fk_field_name = rel.fk_field_name();
    let one_entity_name = &rel.one.entity;
    let many_entity_name = &rel.many.entity;

    // Determine the one-side type name and title field
    let one_type_name = entities
        .get(one_entity_name.as_str())
        .map(|e| e.type_name.clone())
        .unwrap_or_else(|| one_entity_name.clone());
    let one_title_field = entities
        .get(one_entity_name.as_str())
        .map(|e| e.title_field.clone())
        .unwrap_or_else(|| "Name".into());
    let many_type_name = entities
        .get(many_entity_name.as_str())
        .map(|e| e.type_name.clone())
        .unwrap_or_else(|| many_entity_name.clone());

    // Pre-extract conditional ref display property (before mutable borrow)
    let cond_display_property: Option<String> = rel.condition.as_ref().and_then(|cond| {
        match cond.condition {
            Condition::SubsetOf => {
                rel_map.get(cond.reference.as_str()).map(|parent_rel| {
                    entities
                        .get(parent_rel.many.entity.as_str())
                        .map(|e| e.title_field.clone())
                        .unwrap_or_else(|| "Name".into())
                })
            }
        }
    });

    // --- Many side: gets FK property + 1:1 nav ---
    if let Some(many_entity) = entities.get_mut(many_entity_name.as_str()) {
        // FK property (Guid, computed for compositions, editable for references)
        let fk_computed = rel.owned;
        let fk_label = rel
            .fk_label
            .clone()
            .unwrap_or_else(|| rel.many.nav_name.trim_start_matches('_').to_string());

        // Build value list for FK
        let fk_value_list = if let Some(cond) = &rel.condition {
            // Conditional relationship: filter value list by parent's FK
            match cond.condition {
                Condition::SubsetOf => {
                    if let Some(parent_rel) = rel_map.get(cond.reference.as_str()) {
                        Some(ResolvedValueList::EntityRef {
                            collection_path: parent_rel.many.entity.clone(),
                            key_property: "ID".into(),
                            display_property: cond_display_property,
                            filters: vec![ValueListFilter {
                                local_property: "ID".into(),
                                target_property: parent_rel.fk_field_name(),
                            }],
                            fixed_values: false,
                        })
                    } else {
                        None
                    }
                }
            }
        } else if !fk_computed {
            // Non-owned, non-conditional: unfiltered reference to one-side entity
            Some(ResolvedValueList::EntityRef {
                collection_path: one_entity_name.clone(),
                key_property: "ID".into(),
                display_property: Some(one_title_field.clone()),
                filters: vec![],
                fixed_values: false,
            })
        } else {
            None
        };

        // text_path: "NavName/TitleField" for the FK
        let text_path = if !rel.many_side_hidden() {
            Some(format!("{}/{}", rel.many.nav_name, one_title_field))
        } else {
            None
        };

        // Add FK property if not already present
        if !many_entity.properties.iter().any(|p| p.name == fk_field_name) {
            many_entity.properties.push(ResolvedProperty {
                name: fk_field_name.clone(),
                edm_type: "Edm.Guid".into(),
                label: fk_label,
                max_length: None,
                precision: None,
                scale: None,
                computed: fk_computed,
                immutable: false,
                hidden: true,
                text_path,
                value_list: fk_value_list,
                measure: None,
                presentation: ResolvedPresentation {
                    form_group: rel.fk_form_group.clone(),
                    ..Default::default()
                },
                package: rel.package.clone(),
            });
        }

        // 1:1 nav property on the many side → points to the one entity
        many_entity.nav_properties.push(ResolvedNavProperty {
            name: rel.many.nav_name.clone(),
            target_type: one_type_name.clone(),
            target_set: one_entity_name.clone(),
            is_collection: false,
            foreign_key: Some(fk_field_name.clone()),
            relationship: rel.name.clone(),
            is_composition: false,
        });

        // Set parent_set_name for composition children
        if rel.owned {
            many_entity.parent_set_name = Some(one_entity_name.clone());
        }
    }

    // --- One side: gets 1:N nav + optional TableFacet ---
    if let Some(one_entity) = entities.get_mut(one_entity_name.as_str()) {
        // Collection nav property (only for non-conditional relationships)
        if rel.condition.is_none() {
            one_entity.nav_properties.push(ResolvedNavProperty {
                name: rel.one.nav_name.clone(),
                target_type: many_type_name.clone(),
                target_set: many_entity_name.clone(),
                is_collection: true,
                foreign_key: Some(fk_field_name.clone()),
                relationship: rel.name.clone(),
                is_composition: rel.owned,
            });

            // Auto-generate TableFacet for visible collection navs
            if !rel.one_side_hidden() {
                let label = rel.one.nav_name.clone();
                let id = format!("{}Section", rel.one.nav_name);
                one_entity.table_facets.push(ResolvedTableFacet {
                    label,
                    id,
                    navigation_property: rel.one.nav_name.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Condition, ConditionalRef, Side};

    #[test]
    fn test_auto_entity() {
        let e = auto_entity("Customers");
        assert_eq!(e.type_name, "Customer");
        assert_eq!(e.title_field, "Name");
        assert_eq!(e.properties.len(), 2);
        assert_eq!(e.properties[0].name, "ID");
        assert!(e.properties[0].computed);
        assert!(e.properties[0].hidden);
        assert_eq!(e.properties[1].name, "Name");
    }

    #[test]
    fn test_simple_composition() {
        let specs = vec![
            EntitySpec {
                set_name: "Orders".into(),
                package: None,
                type_name: Some("Order".into()),
                type_name_plural: None,
                title_field: Some("OrderName".into()),
                description_field: None,
                fields: vec![FieldSpec::string("OrderName", "Order Name", 80)],
                data_points: vec![],
                header_facets: vec![],
            },
            EntitySpec {
                set_name: "OrderItems".into(),
                package: None,
                type_name: Some("OrderItem".into()),
                type_name_plural: None,
                title_field: Some("ItemName".into()),
                description_field: None,
                fields: vec![FieldSpec::string("ItemName", "Item Name", 80)],
                data_points: vec![],
                header_facets: vec![],
            },
        ];
        let rels = vec![Relationship {
            name: "Order_Items".into(),
            one: Side::new("Orders", "Items"),
            many: Side::new("OrderItems", "_Order"),
            owned: true,
            fk_field: Some("OrderID".into()),
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: None,
        }];

        let resolved = resolve(&specs, &rels);
        assert_eq!(resolved.len(), 2);

        // Orders gets a collection nav "Items" + TableFacet
        let orders = &resolved[0];
        assert_eq!(orders.set_name, "Orders");
        assert!(orders.nav_properties.iter().any(|n| n.name == "Items" && n.is_collection));
        // No table facet because nav is not _-prefixed but wait — "Items" is visible
        // Actually it should have a table facet
        assert!(orders.table_facets.iter().any(|t| t.navigation_property == "Items"));

        // OrderItems gets FK "OrderID" + nav "_Order"
        let items = &resolved[1];
        assert_eq!(items.set_name, "OrderItems");
        assert!(items.properties.iter().any(|p| p.name == "OrderID" && p.computed));
        assert!(items.nav_properties.iter().any(|n| n.name == "_Order" && !n.is_collection));
        assert_eq!(items.parent_set_name.as_deref(), Some("Orders"));
    }

    #[test]
    fn test_conditional_relationship() {
        let specs = vec![
            EntitySpec {
                set_name: "Configs".into(),
                package: None,
                type_name: Some("Config".into()),
                type_name_plural: None,
                title_field: Some("Name".into()),
                description_field: None,
                fields: vec![FieldSpec::string("Name", "Name", 40)],
                data_points: vec![],
                header_facets: vec![],
            },
            EntitySpec {
                set_name: "Fields".into(),
                package: None,
                type_name: Some("Field".into()),
                type_name_plural: None,
                title_field: Some("FieldName".into()),
                description_field: None,
                fields: vec![FieldSpec::string("FieldName", "Field Name", 40)],
                data_points: vec![],
                header_facets: vec![],
            },
        ];
        let rels = vec![
            Relationship {
                name: "Config_Fields".into(),
                one: Side::new("Configs", "Fields"),
                many: Side::new("Fields", "_Config"),
                owned: true,
                fk_field: Some("ConfigID".into()),
                fk_label: None,
                fk_form_group: None,
                condition: None,
                package: None,
            },
            Relationship {
                name: "TitleField".into(),
                one: Side::new("Fields", "_TitleField"),
                many: Side::new("Configs", "_TitlePath"),
                owned: false,
                fk_field: Some("TitlePath".into()),
                fk_label: Some("Title Field".into()),
                fk_form_group: Some("Header".into()),
                condition: Some(ConditionalRef {
                    condition: Condition::SubsetOf,
                    reference: "Config_Fields".into(),
                }),
                package: None,
            },
        ];

        let resolved = resolve(&specs, &rels);
        let configs = resolved.iter().find(|e| e.set_name == "Configs").unwrap();

        // Configs should have FK "TitlePath" with a filtered value list
        let fk = configs.properties.iter().find(|p| p.name == "TitlePath").unwrap();
        assert!(!fk.computed); // non-owned → editable
        assert!(matches!(fk.value_list, Some(ResolvedValueList::EntityRef { .. })));

        // The value list should have a filter
        if let Some(ResolvedValueList::EntityRef { filters, .. }) = &fk.value_list {
            assert_eq!(filters.len(), 1);
            assert_eq!(filters[0].local_property, "ID");
            assert_eq!(filters[0].target_property, "ConfigID");
        }

        // Configs should have 1:1 nav "_TitlePath" but NO collection nav
        assert!(configs.nav_properties.iter().any(|n| n.name == "_TitlePath" && !n.is_collection));
        // Conditional rels don't add collection nav on one-side
        let fields = resolved.iter().find(|e| e.set_name == "Fields").unwrap();
        assert!(!fields.nav_properties.iter().any(|n| n.name == "_TitleField"));
    }

    #[test]
    fn test_auto_entity_from_relationship() {
        let rels = vec![Relationship {
            name: "Order_Customer".into(),
            one: Side::new("Customers", "Orders"),
            many: Side::new("Orders", "Customer"),
            owned: false,
            fk_field: None,
            fk_label: None,
            fk_form_group: None,
            condition: None,
            package: None,
        }];

        let resolved = resolve(&[], &rels);
        assert_eq!(resolved.len(), 2);

        // Both auto-created with ID + Name
        let customers = resolved.iter().find(|e| e.set_name == "Customers").unwrap();
        assert_eq!(customers.type_name, "Customer");
        assert_eq!(customers.title_field, "Name");

        let orders = resolved.iter().find(|e| e.set_name == "Orders").unwrap();
        // Orders should have FK "CustomerID" + nav "Customer"
        assert!(orders.properties.iter().any(|p| p.name == "CustomerID"));
        assert!(orders.nav_properties.iter().any(|n| n.name == "Customer"));
    }
}
