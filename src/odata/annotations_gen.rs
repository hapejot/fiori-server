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
    anns.push(Ann {
        term: "UI.SelectionFields".into(),
        qualifier: None,
        content: AnnContent::PropertyPaths(e.selection_fields.clone()),
    });

    // ── LineItem ──
    let mut line_item_props: Vec<&ResolvedProperty> = e
        .properties
        .iter()
        .filter(|p| p.presentation.show_in_list)
        .collect();
    line_item_props.sort_by_key(|p| p.presentation.list_sort_order.unwrap_or(u32::MAX));

    let mut records = Vec::new();
    for p in &line_item_props {
        let semantic_object = resolve_semantic_object(p, e);
        let record_type = if semantic_object.is_some() {
            "UI.DataFieldWithIntentBasedNavigation"
        } else {
            "UI.DataField"
        };
        let mut props = vec![PV::Path("Value".into(), p.name.clone())];
        if let Some(so) = &semantic_object {
            props.push(PV::Str("SemanticObject".into(), so.clone()));
            props.push(PV::Str("Action".into(), "display".into()));
            props.push(PV::Collection(
                "Mapping".into(),
                vec![Rec {
                    record_type: Some("Common.SemanticObjectMappingType".into()),
                    props: vec![
                        PV::PropPath("LocalProperty".into(), p.name.clone()),
                        PV::Str("SemanticObjectProperty".into(), "ID".into()),
                    ],
                }],
            ));
        }
        if let Some(imp) = &p.presentation.list_importance {
            props.push(PV::EnumMember(
                "![@UI.Importance]".into(),
                format!("UI.ImportanceType/{imp}"),
            ));
        }
        if let Some(crit) = &p.presentation.criticality_path {
            props.push(PV::Path("Criticality".into(), crit.clone()));
        }
        records.push(Rec {
            record_type: Some(record_type.into()),
            props,
        });
    }
    anns.push(Ann {
        term: "UI.LineItem".into(),
        qualifier: None,
        content: AnnContent::Collection(records),
    });

    // ── HeaderInfo ──
    let mut header_props = vec![
        PV::Str("TypeName".into(), e.type_name.clone()),
        PV::Str("TypeNamePlural".into(), e.type_name_plural.clone()),
        PV::Record(
            "Title".into(),
            Rec {
                record_type: Some("UI.DataField".into()),
                props: vec![PV::Path("Value".into(), e.title_field.clone())],
            },
        ),
    ];
    if let Some(desc) = &e.description_field {
        header_props.push(PV::Record(
            "Description".into(),
            Rec {
                record_type: Some("UI.DataField".into()),
                props: vec![PV::Path("Value".into(), desc.clone())],
            },
        ));
    }
    anns.push(Ann {
        term: "UI.HeaderInfo".into(),
        qualifier: None,
        content: AnnContent::Record(Rec {
            record_type: Some("UI.HeaderInfoType".into()),
            props: header_props,
        }),
    });

    // ── HeaderFacets ──
    let hf_records: Vec<Rec> = e
        .header_facets
        .iter()
        .map(|hf| Rec {
            record_type: Some("UI.ReferenceFacet".into()),
            props: vec![
                PV::AnnotationPath(
                    "Target".into(),
                    format!("@UI.DataPoint#{}", hf.data_point_qualifier),
                ),
                PV::Str("Label".into(), hf.label.clone()),
            ],
        })
        .collect();
    anns.push(Ann {
        term: "UI.HeaderFacets".into(),
        qualifier: None,
        content: AnnContent::Collection(hf_records),
    });

    // ── DataPoints ──
    for dp in &e.data_points {
        let mut props = vec![
            PV::Path("Value".into(), dp.value_path.clone()),
            PV::Str("Title".into(), dp.title.clone()),
        ];
        if let Some(max) = dp.max_value {
            props.push(PV::Int("MaximumValue".into(), max));
        }
        if let Some(vis) = &dp.visualization {
            props.push(PV::EnumMember(
                "Visualization".into(),
                format!("UI.VisualizationType/{vis}"),
            ));
        }
        anns.push(Ann {
            term: "UI.DataPoint".into(),
            qualifier: Some(dp.qualifier.clone()),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.DataPointType".into()),
                props,
            }),
        });
    }

    // ── Facets ──
    let mut facet_records = Vec::new();
    for sec in &e.facet_sections {
        facet_records.push(Rec {
            record_type: Some("UI.CollectionFacet".into()),
            props: vec![
                PV::Str("Label".into(), sec.label.clone()),
                PV::Str("ID".into(), sec.id.clone()),
                PV::Collection(
                    "Facets".into(),
                    vec![Rec {
                        record_type: Some("UI.ReferenceFacet".into()),
                        props: vec![
                            PV::AnnotationPath(
                                "Target".into(),
                                format!("@UI.FieldGroup#{}", sec.field_group_qualifier),
                            ),
                            PV::Str("Label".into(), sec.field_group_label.clone()),
                        ],
                    }],
                ),
            ],
        });
    }
    for tf in &e.table_facets {
        facet_records.push(Rec {
            record_type: Some("UI.ReferenceFacet".into()),
            props: vec![
                PV::Str("Label".into(), tf.label.clone()),
                PV::Str("ID".into(), tf.id.clone()),
                PV::AnnotationPath(
                    "Target".into(),
                    format!("{}/@UI.LineItem", tf.navigation_property),
                ),
            ],
        });
    }
    anns.push(Ann {
        term: "UI.Facets".into(),
        qualifier: None,
        content: AnnContent::Collection(facet_records),
    });

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
        let mut fg_records = Vec::new();
        for p in &e.properties {
            if p.presentation.form_group.as_deref() == Some(qualifier) {
                let semantic_object = resolve_semantic_object(p, e);
                let record_type = if semantic_object.is_some() {
                    "UI.DataFieldWithIntentBasedNavigation"
                } else {
                    "UI.DataField"
                };
                let mut props = vec![PV::Path("Value".into(), p.name.clone())];
                if let Some(so) = &semantic_object {
                    props.push(PV::Str("SemanticObject".into(), so.clone()));
                    props.push(PV::Str("Action".into(), "display".into()));
                    props.push(PV::Collection(
                        "Mapping".into(),
                        vec![Rec {
                            record_type: Some("Common.SemanticObjectMappingType".into()),
                            props: vec![
                                PV::PropPath("LocalProperty".into(), p.name.clone()),
                                PV::Str("SemanticObjectProperty".into(), "ID".into()),
                            ],
                        }],
                    ));
                }
                fg_records.push(Rec {
                    record_type: Some(record_type.into()),
                    props,
                });
            }
        }
        anns.push(Ann {
            term: "UI.FieldGroup".into(),
            qualifier: Some(qualifier.clone()),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.FieldGroupType".into()),
                props: vec![PV::Collection("Data".into(), fg_records)],
            }),
        });
    }

    blocks.push(Anns {
        target,
        annotations: anns,
    });

    // ── Property-level SemanticObject + SemanticObjectMapping ──
    for p in &e.properties {
        if let Some(so) = resolve_semantic_object(p, e) {
            blocks.push(Anns {
                target: format!("{NAMESPACE}.{}/{}", e.type_name, p.name),
                annotations: vec![
                    Ann {
                        term: "Common.SemanticObject".into(),
                        qualifier: None,
                        content: AnnContent::Str(so),
                    },
                    Ann {
                        term: "Common.SemanticObjectMapping".into(),
                        qualifier: None,
                        content: AnnContent::Collection(vec![Rec {
                            record_type: None,
                            props: vec![
                                PV::PropPath("LocalProperty".into(), p.name.clone()),
                                PV::Str("SemanticObjectProperty".into(), "ID".into()),
                            ],
                        }]),
                    },
                ],
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
    set_anns.push(Ann {
        term: "Org.OData.Capabilities.V1.UpdateRestrictions".into(),
        qualifier: None,
        content: AnnContent::Record(Rec {
            record_type: None,
            props: vec![PV::Bool("Updatable".into(), true)],
        }),
    });

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
    set_anns.push(Ann {
        term: "Org.OData.Capabilities.V1.InsertRestrictions".into(),
        qualifier: None,
        content: AnnContent::Record(Rec {
            record_type: Some("Capabilities.InsertRestrictionsType".into()),
            props: vec![PV::PropertyPaths(
                "NonInsertableProperties".into(),
                non_insertable,
            )],
        }),
    });

    // DraftRoot or DraftNode
    if is_draft_root {
        set_anns.push(Ann {
            term: "Common.DraftRoot".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("Common.DraftRootType".into()),
                props: vec![
                    PV::Str(
                        "ActivationAction".into(),
                        format!("{NAMESPACE}.draftActivate"),
                    ),
                    PV::Str("EditAction".into(), format!("{NAMESPACE}.draftEdit")),
                    PV::Str(
                        "PreparationAction".into(),
                        format!("{NAMESPACE}.draftPrepare"),
                    ),
                ],
            }),
        });
    } else {
        set_anns.push(Ann {
            term: "Common.DraftNode".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("Common.DraftNodeType".into()),
                props: vec![PV::Str(
                    "PreparationAction".into(),
                    format!("{NAMESPACE}.draftPrepare"),
                )],
            }),
        });
    }

    blocks.push(Anns {
        target: format!("{NAMESPACE}.EntityContainer/{}", e.set_name),
        annotations: set_anns,
    });

    // ── Per-property annotations ──
    for p in &e.properties {
        let mut prop_anns = vec![Ann {
            term: "Common.Label".into(),
            qualifier: None,
            content: AnnContent::Str(p.label.clone()),
        }];

        // UI.Hidden
        if p.hidden {
            prop_anns.push(Ann {
                term: "UI.Hidden".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        }

        // Core.Computed / Core.Immutable
        if p.computed {
            prop_anns.push(Ann {
                term: "Org.OData.Core.V1.Computed".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        } else if p.immutable {
            prop_anns.push(Ann {
                term: "Org.OData.Core.V1.Immutable".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        }

        // Common.Text on key field
        if p.name == e.key_field && e.title_field != e.key_field {
            prop_anns.push(text_arrangement_ann(&e.title_field));
        }

        // Common.Text from text_path
        if let Some(tp) = &p.text_path {
            prop_anns.push(text_arrangement_ann(tp));
        }

        // Measures
        if let Some(m) = &p.measure {
            let term = match m.kind {
                MeasureKind::Currency => "Org.OData.Measures.V1.ISOCurrency",
                MeasureKind::Unit => "Org.OData.Measures.V1.Unit",
            };
            prop_anns.push(Ann {
                term: term.into(),
                qualifier: None,
                content: AnnContent::PathWithChildren(m.unit_field.clone(), vec![]),
            });
        }

        // Common.ValueList
        if let Some(vl) = &p.value_list {
            prop_anns.extend(build_value_list_anns(&p.name, vl));
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
            annotations: vec![Ann {
                term: "Org.OData.Core.V1.Computed".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            }],
        });
    }

    blocks
}

// ── Helpers ────────────────────────────────────────────────────

/// Build Common.Text + UI.TextArrangement/TextOnly annotation.
fn text_arrangement_ann(path: &str) -> Ann {
    Ann {
        term: "Common.Text".into(),
        qualifier: None,
        content: AnnContent::PathWithChildren(
            path.into(),
            vec![Ann {
                term: "UI.TextArrangement".into(),
                qualifier: None,
                content: AnnContent::EnumMember("UI.TextArrangementType/TextOnly".into()),
            }],
        ),
    }
}

/// Build Common.ValueList (+ optional Common.ValueListWithFixedValues) annotations.
fn build_value_list_anns(local_property: &str, vl: &ResolvedValueList) -> Vec<Ann> {
    match vl {
        ResolvedValueList::CodeList {
            list_id,
            fixed_values,
        } => {
            let params = vec![
                Rec {
                    record_type: Some("Common.ValueListParameterOut".into()),
                    props: vec![
                        PV::PropPath("LocalDataProperty".into(), local_property.into()),
                        PV::Str("ValueListProperty".into(), "Code".into()),
                    ],
                },
                Rec {
                    record_type: Some("Common.ValueListParameterDisplayOnly".into()),
                    props: vec![PV::Str("ValueListProperty".into(), "Description".into())],
                },
                Rec {
                    record_type: Some("Common.ValueListParameterConstant".into()),
                    props: vec![
                        PV::Str("ValueListProperty".into(), "ListID".into()),
                        PV::Str("Constant".into(), list_id.clone()),
                    ],
                },
            ];
            let mut anns = vec![Ann {
                term: "Common.ValueList".into(),
                qualifier: None,
                content: AnnContent::Record(Rec {
                    record_type: Some("Common.ValueListType".into()),
                    props: vec![
                        PV::Str("CollectionPath".into(), "FieldValueListItems".into()),
                        PV::Collection("Parameters".into(), params),
                    ],
                }),
            }];
            if *fixed_values {
                anns.push(Ann {
                    term: "Common.ValueListWithFixedValues".into(),
                    qualifier: None,
                    content: AnnContent::Bool(true),
                });
            }
            anns
        }
        ResolvedValueList::EntityRef {
            collection_path,
            key_property,
            display_property,
            filters,
            fixed_values,
        } => {
            let mut params = vec![Rec {
                record_type: Some("Common.ValueListParameterOut".into()),
                props: vec![
                    PV::PropPath("LocalDataProperty".into(), local_property.into()),
                    PV::Str("ValueListProperty".into(), key_property.clone()),
                ],
            }];
            if let Some(dp) = display_property {
                params.push(Rec {
                    record_type: Some("Common.ValueListParameterDisplayOnly".into()),
                    props: vec![PV::Str("ValueListProperty".into(), dp.clone())],
                });
            }
            for f in filters {
                params.push(Rec {
                    record_type: Some("Common.ValueListParameterIn".into()),
                    props: vec![
                        PV::PropPath("LocalDataProperty".into(), f.local_property.clone()),
                        PV::Str("ValueListProperty".into(), f.target_property.clone()),
                    ],
                });
            }
            let mut anns = vec![Ann {
                term: "Common.ValueList".into(),
                qualifier: None,
                content: AnnContent::Record(Rec {
                    record_type: Some("Common.ValueListType".into()),
                    props: vec![
                        PV::Str("CollectionPath".into(), collection_path.clone()),
                        PV::Collection("Parameters".into(), params),
                    ],
                }),
            }];
            if *fixed_values {
                anns.push(Ann {
                    term: "Common.ValueListWithFixedValues".into(),
                    qualifier: None,
                    content: AnnContent::Bool(true),
                });
            }
            anns
        }
    }
}

/// Determine the semantic object for a property (FK referencing another entity).
///
/// If the property has an EntityRef value list, the collection_path is the semantic object.
/// This enables Intent-Based Navigation in LineItem and FieldGroup.
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
    use crate::spec::ValueListFilter;

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
                    field_group_label: "General".into(),
                },
                ResolvedFacetSection {
                    label: "Pricing".into(),
                    id: "PricingSection".into(),
                    field_group_qualifier: "Pricing".into(),
                    field_group_label: "Pricing".into(),
                },
            ],
            table_facets: vec![],
            selection_fields: vec!["OrderName".into()],
            package: None,
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
        // Test ValueListParameterIn generation
        let vl = ResolvedValueList::EntityRef {
            collection_path: "EntityFields".into(),
            key_property: "ID".into(),
            display_property: Some("FieldName".into()),
            filters: vec![ValueListFilter {
                local_property: "ID".into(),
                target_property: "ConfigID".into(),
            }],
            fixed_values: false,
        };
        let anns = build_value_list_anns("TitlePath", &vl);
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
