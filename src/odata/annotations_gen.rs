//! Layer 3: OData annotation generation from ResolvedEntity.
//!
//! Produces structured `Anns` blocks that serialize to EDMX annotation XML.
//! Two main entry points:
//! - `generate_ui_annotations()` → UI.SelectionFields, UI.LineItem, UI.HeaderInfo,
//!   UI.HeaderFacets, UI.DataPoint, UI.Facets, UI.FieldGroup, SemanticObject
//! - `generate_capability_annotations()` → UpdateRestrictions, InsertRestrictions,
//!   DraftRoot/DraftNode, Common.Label, UI.Hidden, Core.Computed/Immutable,
//!   Common.Text, Common.ValueList, Measures

use crate::model::resolved::*;
use crate::odata::vocab::*;
use crate::odata::xml_types::*;
use crate::spec::MeasureKind;
use crate::NAMESPACE;

/// Generate all annotations for a ResolvedEntity.
pub fn generate_annotations(e: &ResolvedEntity) -> Vec<Anns> {
    let mut blocks = generate_ui_annotations(e);
    blocks.extend(generate_capability_annotations(e));
    blocks
}

// ── UI Annotations ─────────────────────────────────────────────

/// Generate UI annotations (Target: `Service.{TypeName}`).
///
/// SelectionFields, LineItem, HeaderInfo, HeaderFacets, DataPoints,
/// Facets (sections + table facets), FieldGroups, SemanticObject mappings.
pub fn generate_ui_annotations(e: &ResolvedEntity) -> Vec<Anns> {
    let mut blocks = Vec::new();
    let target = format!("{NAMESPACE}.{}", e.type_name);
    let mut anns = Vec::new();

    // ── SelectionFields ──
    anns.push(SelectionFields(e.selection_fields.clone()).to_ann());

    // ── LineItem ──
    let mut line_item_props: Vec<&ResolvedProperty> = e
        .properties
        .iter()
        .filter(|p| p.presentation.show_in_list)
        .collect();
    line_item_props.sort_by_key(|p| p.presentation.list_sort_order.unwrap_or(u32::MAX));

    let fields: Vec<DataFieldVariant> = line_item_props
        .iter()
        .map(|p| build_data_field(p, e))
        .collect();
    anns.push(LineItem { fields }.to_ann());

    // ── HeaderInfo ──
    anns.push(
        HeaderInfo {
            type_name: e.type_name.clone(),
            type_name_plural: e.type_name_plural.clone(),
            title: DataFieldVariant::DataField {
                value: e.title_field.clone(),
                criticality: None,
                importance: None,
            },
            description: e.description_field.as_ref().map(|d| {
                DataFieldVariant::DataField {
                    value: d.clone(),
                    criticality: None,
                    importance: None,
                }
            }),
        }
        .to_ann(),
    );

    // ── HeaderFacets ──
    anns.push(
        HeaderFacets(
            e.header_facets
                .iter()
                .map(|hf| FacetVariant::ReferenceFacet {
                    id: hf.data_point_qualifier.clone(),
                    label: hf.label.clone(),
                    target: format!("@UI.DataPoint#{}", hf.data_point_qualifier),
                })
                .collect(),
        )
        .to_ann(),
    );

    // ── DataPoints ──
    for dp in &e.data_points {
        anns.push(
            DataPoint {
                qualifier: dp.qualifier.clone(),
                value: dp.value_path.clone(),
                title: dp.title.clone(),
                max_value: dp.max_value,
                visualization: dp
                    .visualization
                    .as_deref()
                    .and_then(VisualizationType::from_str),
                criticality: None,
            }
            .to_ann(),
        );
    }

    // ── Facets ──
    let mut facet_variants: Vec<FacetVariant> = Vec::new();
    for sec in &e.facet_sections {
        facet_variants.push(FacetVariant::ReferenceFacet {
            id: sec.id.clone(),
            label: sec.label.clone(),
            target: format!("@UI.FieldGroup#{}", sec.field_group_qualifier),
        });
    }
    for tf in &e.table_facets {
        facet_variants.push(FacetVariant::ReferenceFacet {
            id: tf.id.clone(),
            label: tf.label.clone(),
            target: format!("{}/@UI.LineItem", tf.navigation_property),
        });
    }
    anns.push(Facets(facet_variants).to_ann());

    // ── FieldGroups ──
    let mut seen_qualifiers: Vec<String> = Vec::new();
    for p in &e.properties {
        if let Some(q) = &p.presentation.form_group {
            if !seen_qualifiers.contains(q) {
                seen_qualifiers.push(q.clone());
            }
        }
    }
    for qualifier in &seen_qualifiers {
        let data: Vec<DataFieldVariant> = e
            .properties
            .iter()
            .filter(|p| p.presentation.form_group.as_deref() == Some(qualifier))
            .map(|p| build_data_field(p, e))
            .collect();
        anns.push(FieldGroup {
            qualifier: qualifier.clone(),
            data,
        }.to_ann());
    }

    blocks.push(Anns {
        target,
        annotations: anns,
    });

    // ── Property-level SemanticObject + SemanticObjectMapping ──
    for p in &e.properties {
        if let Some(so) = resolve_semantic_object(p, e) {
            let sa = SemanticObjectAnnotation {
                semantic_object: so,
                mapping: vec![SemanticObjectMapping {
                    local_property: p.name.clone(),
                    semantic_object_property: "ID".into(),
                }],
            };
            blocks.push(Anns {
                target: format!("{NAMESPACE}.{}/{}", e.type_name, p.name),
                annotations: sa.to_anns(),
            });
        }
    }

    blocks
}

// ── Capability Annotations ─────────────────────────────────────

/// Generate capability annotations for a ResolvedEntity.
///
/// EntitySet-level: UpdateRestrictions, InsertRestrictions, DraftRoot/DraftNode.
/// Property-level: Common.Label, UI.Hidden, Core.Computed, Core.Immutable,
/// Common.Text, Common.ValueList, Measures.
pub fn generate_capability_annotations(e: &ResolvedEntity) -> Vec<Anns> {
    let mut blocks = Vec::new();
    let is_draft_root = e.parent_set_name.is_none();

    // ── EntitySet-level ──
    let mut set_anns = Vec::new();

    // UpdateRestrictions
    set_anns.push(UpdateRestrictions { updatable: true }.to_ann());

    // InsertRestrictions — non-insertable = computed fields + draft flags
    let mut non_insertable: Vec<String> = e
        .properties
        .iter()
        .filter(|p| p.computed)
        .map(|p| p.name.clone())
        .collect();
    non_insertable.extend(
        ["IsActiveEntity", "HasActiveEntity", "HasDraftEntity"]
            .iter()
            .map(|s| (*s).into()),
    );
    set_anns.push(
        InsertRestrictions {
            non_insertable_properties: non_insertable,
        }
        .to_ann(),
    );

    // DraftRoot or DraftNode
    if is_draft_root {
        set_anns.push(
            DraftRoot {
                activation_action: format!("{NAMESPACE}.draftActivate"),
                edit_action: format!("{NAMESPACE}.draftEdit"),
                preparation_action: format!("{NAMESPACE}.draftPrepare"),
            }
            .to_ann(),
        );
    } else {
        set_anns.push(
            DraftNode {
                preparation_action: format!("{NAMESPACE}.draftPrepare"),
            }
            .to_ann(),
        );
    }

    blocks.push(Anns {
        target: format!("{NAMESPACE}.EntityContainer/{}", e.set_name),
        annotations: set_anns,
    });

    // ── Per-property annotations ──
    for p in &e.properties {
        let mut prop_anns = vec![ScalarAnnotation::label(&p.label).to_ann()];

        // UI.Hidden
        if p.hidden {
            prop_anns.push(ScalarAnnotation::hidden().to_ann());
        }

        // Core.Computed / Core.Immutable
        if p.computed {
            prop_anns.push(ScalarAnnotation::computed().to_ann());
        } else if p.immutable {
            prop_anns.push(ScalarAnnotation::immutable().to_ann());
        }

        // Common.Text on key field
        if p.name == e.key_field && e.title_field != e.key_field {
            prop_anns.push(
                TextAnnotation {
                    path: e.title_field.clone(),
                    arrangement: TextArrangementType::TextOnly,
                }
                .to_ann(),
            );
        }

        // Common.Text from text_path
        if let Some(tp) = &p.text_path {
            prop_anns.push(
                TextAnnotation {
                    path: tp.clone(),
                    arrangement: TextArrangementType::TextOnly,
                }
                .to_ann(),
            );
        }

        // Measures
        if let Some(m) = &p.measure {
            let ma = match m.kind {
                MeasureKind::Currency => MeasureAnnotation::ISOCurrency(m.unit_field.clone()),
                MeasureKind::Unit => MeasureAnnotation::Unit(m.unit_field.clone()),
            };
            prop_anns.push(ma.to_ann());
        }

        // Common.ValueList
        if let Some(vl) = &p.value_list {
            prop_anns.extend(resolved_value_list_to_vocab(vl).to_anns(&p.name));
        }

        blocks.push(Anns {
            target: format!("{NAMESPACE}.{}/{}", e.type_name, p.name),
            annotations: prop_anns,
        });
    }

    // Draft flag properties — Core.Computed
    for draft_prop in ["IsActiveEntity", "HasActiveEntity", "HasDraftEntity"] {
        blocks.push(Anns {
            target: format!("{NAMESPACE}.{}/{draft_prop}", e.type_name),
            annotations: vec![ScalarAnnotation::computed().to_ann()],
        });
    }

    blocks
}

// ── Helpers ────────────────────────────────────────────────────

/// Build a `DataFieldVariant` from a resolved property.
///
/// If the property has an EntityRef value list → DataFieldWithIntentBasedNavigation,
/// otherwise → DataField (with optional criticality + importance).
fn build_data_field(p: &ResolvedProperty, e: &ResolvedEntity) -> DataFieldVariant {
    let semantic_object = resolve_semantic_object(p, e);
    if let Some(so) = semantic_object {
        DataFieldVariant::DataFieldWithIntentBasedNavigation {
            value: p.name.clone(),
            semantic_object: so,
            action: "display".into(),
            mapping: vec![SemanticObjectMapping {
                local_property: p.name.clone(),
                semantic_object_property: "ID".into(),
            }],
            importance: p
                .presentation
                .list_importance
                .as_deref()
                .and_then(ImportanceType::from_str),
        }
    } else {
        DataFieldVariant::DataField {
            value: p.name.clone(),
            criticality: p
                .presentation
                .criticality_path
                .as_ref()
                .map(|c| CriticalitySource::Path(c.clone())),
            importance: p
                .presentation
                .list_importance
                .as_deref()
                .and_then(ImportanceType::from_str),
        }
    }
}

/// Convert a `ResolvedValueList` (model layer) to a `ValueListAnnotation` (vocab layer).
fn resolved_value_list_to_vocab(vl: &ResolvedValueList) -> ValueListAnnotation {
    match vl {
        ResolvedValueList::CodeList {
            list_id,
            fixed_values,
        } => ValueListAnnotation::CodeList {
            list_id: list_id.clone(),
            fixed_values: *fixed_values,
        },
        ResolvedValueList::EntityRef {
            collection_path,
            key_property,
            display_property,
            filters,
            fixed_values,
        } => ValueListAnnotation::EntityRef {
            collection_path: collection_path.clone(),
            key_property: key_property.clone(),
            display_property: display_property.clone(),
            filters: filters
                .iter()
                .map(|f| crate::odata::vocab::ValueListFilter {
                    local_property: f.local_property.clone(),
                    target_property: f.target_property.clone(),
                })
                .collect(),
            fixed_values: *fixed_values,
        },
    }
}

/// Determine the semantic object for a property (FK referencing another entity).
fn resolve_semantic_object(p: &ResolvedProperty, _e: &ResolvedEntity) -> Option<String> {
    match &p.value_list {
        Some(ResolvedValueList::EntityRef {
            collection_path, ..
        }) => Some(collection_path.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entity() -> ResolvedEntity {
        ResolvedEntity {
            set_name: "Orders".into(),
            type_name: "Order".into(),
            type_name_plural: "Orders".into(),
            key_field: "ID".into(),
            title_field: "OrderName".into(),
            description_field: Some("Status".into()),
            parent_set_name: None,
            properties: vec![
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
                },
                ResolvedProperty {
                    name: "OrderName".into(),
                    edm_type: "Edm.String".into(),
                    label: "Order Name".into(),
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
                        list_sort_order: Some(1),
                        ..Default::default()
                    },
                    package: None,
                },
                ResolvedProperty {
                    name: "Status".into(),
                    edm_type: "Edm.String".into(),
                    label: "Status".into(),
                    max_length: Some(10),
                    precision: None,
                    scale: None,
                    computed: false,
                    immutable: false,
                    hidden: false,
                    text_path: Some("_Status_text".into()),
                    value_list: Some(ResolvedValueList::CodeList {
                        list_id: "abc-123".into(),
                        fixed_values: true,
                    }),
                    measure: None,
                    presentation: ResolvedPresentation {
                        show_in_list: true,
                        list_sort_order: Some(2),
                        form_group: Some("General".into()),
                        ..Default::default()
                    },
                    package: None,
                },
                ResolvedProperty {
                    name: "CustomerID".into(),
                    edm_type: "Edm.Guid".into(),
                    label: "Customer".into(),
                    max_length: None,
                    precision: None,
                    scale: None,
                    computed: false,
                    immutable: false,
                    hidden: true,
                    text_path: Some("Customer/CustomerName".into()),
                    value_list: Some(ResolvedValueList::EntityRef {
                        collection_path: "Customers".into(),
                        key_property: "ID".into(),
                        display_property: Some("CustomerName".into()),
                        filters: vec![],
                        fixed_values: false,
                    }),
                    measure: None,
                    presentation: ResolvedPresentation {
                        show_in_list: true,
                        list_sort_order: Some(3),
                        form_group: Some("General".into()),
                        ..Default::default()
                    },
                    package: None,
                },
                ResolvedProperty {
                    name: "Price".into(),
                    edm_type: "Edm.Decimal".into(),
                    label: "Price".into(),
                    max_length: None,
                    precision: Some(12),
                    scale: Some(2),
                    computed: false,
                    immutable: false,
                    hidden: false,
                    text_path: None,
                    value_list: None,
                    measure: Some(ResolvedMeasure {
                        unit_field: "Currency".into(),
                        kind: MeasureKind::Currency,
                    }),
                    presentation: ResolvedPresentation {
                        form_group: Some("Pricing".into()),
                        ..Default::default()
                    },
                    package: None,
                },
            ],
            nav_properties: vec![],
            data_points: vec![ResolvedDataPoint {
                qualifier: "Price".into(),
                value_path: "Price".into(),
                title: "Price".into(),
                max_value: None,
                visualization: None,
            }],
            header_facets: vec![ResolvedHeaderFacet {
                data_point_qualifier: "Price".into(),
                label: "Price".into(),
            }],
            facet_sections: vec![
                ResolvedFacetSection {
                    label: "General".into(),
                    id: "GeneralSection".into(),
                    field_group_qualifier: "General".into(),
                },
                ResolvedFacetSection {
                    label: "Pricing".into(),
                    id: "PricingSection".into(),
                    field_group_qualifier: "Pricing".into(),
                },
            ],
            table_facets: vec![],
            selection_fields: vec!["OrderName".into()],
            package: None,
            extra_annotations_xml: String::new(),
            custom_actions_xml: String::new(),
        }
    }

    #[test]
    fn test_ui_annotations_structure() {
        let e = sample_entity();
        let blocks = generate_ui_annotations(&e);

        // First block: main UI annotations on Service.Order
        let main = &blocks[0];
        assert_eq!(main.target, "Service.Order");

        let terms: Vec<&str> = main.annotations.iter().map(|a| a.term.as_str()).collect();
        assert!(terms.contains(&"UI.SelectionFields"));
        assert!(terms.contains(&"UI.LineItem"));
        assert!(terms.contains(&"UI.HeaderInfo"));
        assert!(terms.contains(&"UI.HeaderFacets"));
        assert!(terms.contains(&"UI.DataPoint"));
        assert!(terms.contains(&"UI.Facets"));
        assert!(terms.contains(&"UI.FieldGroup"));
    }

    #[test]
    fn test_semantic_object_from_entity_ref() {
        let e = sample_entity();
        let blocks = generate_ui_annotations(&e);

        // Should have per-property SemanticObject block for CustomerID
        let so_block = blocks.iter().find(|b| b.target == "Service.Order/CustomerID");
        assert!(so_block.is_some(), "Should have SemanticObject block for CustomerID");
        let so = so_block.unwrap();
        assert!(so.annotations.iter().any(|a| a.term == "Common.SemanticObject"));
    }

    #[test]
    fn test_capability_annotations() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        // EntitySet-level block
        let set_block = blocks
            .iter()
            .find(|b| b.target == "Service.EntityContainer/Orders")
            .unwrap();
        let terms: Vec<&str> = set_block.annotations.iter().map(|a| a.term.as_str()).collect();
        assert!(terms.contains(&"Org.OData.Capabilities.V1.UpdateRestrictions"));
        assert!(terms.contains(&"Org.OData.Capabilities.V1.InsertRestrictions"));
        assert!(terms.contains(&"Common.DraftRoot")); // no parent = DraftRoot
    }

    #[test]
    fn test_draft_node_for_child() {
        let mut e = sample_entity();
        e.parent_set_name = Some("Customers".into());
        let blocks = generate_capability_annotations(&e);

        let set_block = blocks
            .iter()
            .find(|b| b.target == "Service.EntityContainer/Orders")
            .unwrap();
        assert!(set_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.DraftNode"));
    }

    #[test]
    fn test_common_text_on_key() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        let id_block = blocks
            .iter()
            .find(|b| b.target == "Service.Order/ID")
            .unwrap();
        assert!(id_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.Text"));
    }

    #[test]
    fn test_value_list_code_list() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        let status_block = blocks
            .iter()
            .find(|b| b.target == "Service.Order/Status")
            .unwrap();
        assert!(status_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.ValueList"));
        assert!(status_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.ValueListWithFixedValues"));
    }

    #[test]
    fn test_value_list_entity_ref() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        let cust_block = blocks
            .iter()
            .find(|b| b.target == "Service.Order/CustomerID")
            .unwrap();
        assert!(cust_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.ValueList"));
        // EntityRef without fixed_values → no WithFixedValues
        assert!(!cust_block
            .annotations
            .iter()
            .any(|a| a.term == "Common.ValueListWithFixedValues"));
    }

    #[test]
    fn test_measure_annotation() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        let price_block = blocks
            .iter()
            .find(|b| b.target == "Service.Order/Price")
            .unwrap();
        assert!(price_block
            .annotations
            .iter()
            .any(|a| a.term == "Org.OData.Measures.V1.ISOCurrency"));
    }

    #[test]
    fn test_text_path_annotation() {
        let e = sample_entity();
        let blocks = generate_capability_annotations(&e);

        let status_block = blocks
            .iter()
            .find(|b| b.target == "Service.Order/Status")
            .unwrap();
        let text_ann = status_block
            .annotations
            .iter()
            .find(|a| a.term == "Common.Text")
            .unwrap();
        match &text_ann.content {
            AnnContent::PathWithChildren(path, _) => {
                assert_eq!(path, "_Status_text");
            }
            _ => panic!("Expected PathWithChildren for Common.Text"),
        }
    }

    #[test]
    fn test_value_list_with_filters() {
        // Test ValueListParameterIn generation via vocab type
        let vl = ValueListAnnotation::EntityRef {
            collection_path: "EntityFields".into(),
            key_property: "ID".into(),
            display_property: Some("FieldName".into()),
            filters: vec![crate::odata::vocab::ValueListFilter {
                local_property: "ID".into(),
                target_property: "ConfigID".into(),
            }],
            fixed_values: false,
        };
        let anns = vl.to_anns("TitlePath");
        let xml = anns_to_xml(&[Anns {
            target: "test".into(),
            annotations: anns,
        }]);
        assert!(xml.contains("Common.ValueListParameterIn"));
        assert!(xml.contains("ConfigID"));
    }

    #[test]
    fn test_full_xml_roundtrip() {
        let e = sample_entity();
        let blocks = generate_annotations(&e);
        let xml = anns_to_xml(&blocks);

        // Smoke test: valid structure
        assert!(xml.contains("<Annotations Target=\"Service.Order\">"));
        assert!(xml.contains("<Annotations Target=\"Service.EntityContainer/Orders\">"));
        assert!(xml.contains("UI.SelectionFields"));
        assert!(xml.contains("UI.LineItem"));
        assert!(xml.contains("Common.DraftRoot"));
        assert!(xml.contains("ISOCurrency"));
    }
}
