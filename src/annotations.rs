use crate::NAMESPACE;

/// Einheitliche Feld-Definition fuer EntityType-Properties UND Annotations.
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub edm_type: &'static str,
    pub max_length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    /// Feld kann beim Erstellen gesetzt, danach nicht mehr geaendert werden (Core.Immutable).
    pub immutable: bool,
    /// Feld wird vom Server berechnet/generiert (Core.Computed) – nie im Formular sichtbar.
    pub computed: bool,
    /// FK-Referenz auf ein anderes EntitySet (z.B. "Customers").
    /// Erzeugt automatisch: 1:1 NavigationProperty, Common.Text, ValueList, Intent-Based Navigation.
    pub references_entity: Option<&'static str>,
    /// Name einer Werteliste (UUID der FieldValueList) – erzeugt Common.ValueList
    /// mit CollectionPath="FieldValueListItems", Out=Code, Display=Description.
    pub value_source: Option<&'static str>,
    /// true → Suchdialog bevorzugen, false → Dropdown (betrifft value_source und references_entity).
    pub prefer_dialog: bool,
    /// Pfad fuer Common.Text bei FK-Referenzen (z.B. "_ValueList/ListName").
    /// Erzeugt Common.Text + UI.TextArrangement/TextOnly auf diesem Feld.
    pub text_path: Option<&'static str>,
    // ── Annotation-Steuerung (abgeleitet in build_annotations) ──
    /// Feld erscheint als Suchfilter (UI.SelectionFields).
    pub searchable: bool,
    /// Feld erscheint als Listenspalte (UI.LineItem).
    pub show_in_list: bool,
    /// Reihenfolge in der Liste (aufsteigend sortiert).
    pub list_sort_order: Option<u32>,
    /// Wichtigkeit in der Liste ("High", "Medium", "Low").
    pub list_importance: Option<&'static str>,
    /// Pfad fuer Criticality-Indikator in der Liste.
    pub list_criticality_path: Option<&'static str>,
    /// FieldGroup-Qualifier – ordnet das Feld einer Formulargruppe zu (z.B. "Basic", "Tile").
    pub form_group: Option<&'static str>,
}

/// Flexible ValueList-Konfiguration fuer Custom-Wertehilfen.
pub struct ValueListDef {
    /// OData-EntitySet (z.B. "FieldValueLists", "EntityConfigs")
    pub collection_path: &'static str,
    /// Feld im Ziel-EntitySet, das als Out-Parameter zurueckgegeben wird
    pub key_property: &'static str,
    /// Optionales Display-Only-Feld
    pub display_property: Option<&'static str>,
    /// true → Dropdown (Common.ValueListWithFixedValues), false → Dialog
    pub fixed_values: bool,
}

/// NavigationProperty-Definition im EntityType.
pub struct NavigationPropertyDef {
    pub name: &'static str,
    pub target_type: &'static str,
    /// true fuer 1:n Kompositionen (erzeugt Collection-Typ)
    pub is_collection: bool,
    /// Fremdschluessel-Feld auf dem Kind (bei 1:n) bzw. auf this (bei 1:1).
    /// Wenn None, wird der Schluesselname des Parents verwendet.
    pub foreign_key: Option<&'static str>,
}

/// DataPoint fuer den Object-Page-Header.
pub struct DataPointDef {
    pub qualifier: &'static str,
    pub value_path: &'static str,
    pub title: &'static str,
    pub max_value: Option<u32>,
    pub visualization: Option<&'static str>,
}

/// ReferenceFacet im HeaderFacets-Block – verweist auf einen DataPoint.
pub struct HeaderFacetDef {
    pub data_point_qualifier: &'static str,
    pub label: &'static str,
}

/// Ein CollectionFacet auf der Object Page, verweist auf eine FieldGroup.
pub struct FacetSectionDef {
    pub label: &'static str,
    pub id: &'static str,
    pub field_group_qualifier: &'static str,
    pub field_group_label: &'static str,
}

/// Tabellen-Facet: verweist auf die UI.LineItem-Annotation einer Komposition (NavProperty).
pub struct TableFacetDef {
    pub label: &'static str,
    pub id: &'static str,
    /// Name des NavigationProperty (z.B. "Items")
    pub navigation_property: &'static str,
}

/// Kopfzeile der Object Page.
pub struct HeaderInfoDef {
    pub type_name: &'static str,
    pub type_name_plural: &'static str,
    pub title_path: &'static str,
    pub description_path: &'static str,
}

/// Komplette Annotation-Definition fuer eine Entitaet.
pub struct AnnotationsDef {
    pub header_info: HeaderInfoDef,
    pub header_facets: &'static [HeaderFacetDef],
    pub data_points: &'static [DataPointDef],
    pub facet_sections: &'static [FacetSectionDef],
    /// Tabellen-Facets fuer Kompositionen (z.B. OrderItems).
    pub table_facets: &'static [TableFacetDef],
}

// ── XML building blocks ────────────────────────────────────────

/// A `<PropertyValue Property="..." .../>` element with typed content.
#[derive(Debug, PartialEq)]
pub enum PV {
    /// `String="Y"`
    Str(String, String),
    /// `Path="Y"`
    Path(String, String),
    /// `AnnotationPath="Y"`
    AnnotationPath(String, String),
    /// `NavigationPropertyPath="Y"`
    NavPropPath(String, String),
    /// `PropertyPath="Y"`
    PropPath(String, String),
    /// `EnumMember="Y"`
    EnumMember(String, String),
    /// `Int="Y"`
    Int(String, u32),
    /// `Bool="Y"`
    Bool(String, bool),
    /// Nested `<Record>` child
    Record(String, Rec),
    /// Nested `<Collection>` of `<Record>`s
    Collection(String, Vec<Rec>),
    /// Nested `<Collection>` of `<PropertyPath>`s
    PropertyPaths(String, Vec<String>),
}

/// A `<Record Type="...">` element with child PropertyValues.
#[derive(Debug, PartialEq)]
pub struct Rec {
    pub record_type: Option<String>,
    pub props: Vec<PV>,
}

/// Content wrapped by an `<Annotation>` element.
#[derive(Debug, PartialEq)]
pub enum AnnContent {
    Record(Rec),
    Collection(Vec<Rec>),
    PropertyPaths(Vec<String>),
    Str(String),
    Bool(bool),
    EnumMember(String),
    PathWithChildren(String, Vec<Ann>),
    Empty,
}

/// An `<Annotation Term="..." ...>` element.
#[derive(Debug, PartialEq)]
pub struct Ann {
    pub term: String,
    pub qualifier: Option<String>,
    pub content: AnnContent,
}

/// An `<Annotations Target="...">` block containing child annotations.
#[derive(Debug, PartialEq)]
pub struct Anns {
    pub target: String,
    pub annotations: Vec<Ann>,
}

// ── Serialization ──────────────────────────────────────────────

impl PV {
    pub fn to_xml(&self, x: &mut String) {
        match self {
            PV::Str(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" String="{v}"/>"#
            )),
            PV::Path(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" Path="{v}"/>"#
            )),
            PV::AnnotationPath(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" AnnotationPath="{v}"/>"#
            )),
            PV::NavPropPath(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" NavigationPropertyPath="{v}"/>"#
            )),
            PV::PropPath(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" PropertyPath="{v}"/>"#
            )),
            PV::EnumMember(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" EnumMember="{v}"/>"#
            )),
            PV::Int(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" Int="{v}"/>"#
            )),
            PV::Bool(p, v) => x.push_str(&format!(
                r#"<PropertyValue Property="{p}" Bool="{v}"/>"#
            )),
            PV::Record(p, rec) => {
                x.push_str(&format!(r#"<PropertyValue Property="{p}">"#));
                rec.to_xml(x);
                x.push_str("</PropertyValue>");
            }
            PV::Collection(p, recs) => {
                x.push_str(&format!(r#"<PropertyValue Property="{p}">"#));
                x.push_str("<Collection>");
                for r in recs {
                    r.to_xml(x);
                }
                x.push_str("</Collection>");
                x.push_str("</PropertyValue>");
            }
            PV::PropertyPaths(p, paths) => {
                x.push_str(&format!(r#"<PropertyValue Property="{p}">"#));
                x.push_str("<Collection>");
                for path in paths {
                    x.push_str(&format!("<PropertyPath>{path}</PropertyPath>"));
                }
                x.push_str("</Collection>");
                x.push_str("</PropertyValue>");
            }
        }
    }
}

impl Rec {
    pub fn to_xml(&self, x: &mut String) {
        match &self.record_type {
            Some(rt) => x.push_str(&format!(r#"<Record Type="{rt}">"#)),
            None => x.push_str("<Record>"),
        }
        for pv in &self.props {
            pv.to_xml(x);
        }
        x.push_str("</Record>");
    }
}

impl Ann {
    pub fn to_xml(&self, x: &mut String) {
        // Opening: <Annotation Term="..." [Qualifier="..."]
        let q_attr = match &self.qualifier {
            Some(q) => format!(r#" Qualifier="{q}""#),
            None => String::new(),
        };
        match &self.content {
            AnnContent::Str(val) => {
                x.push_str(&format!(
                    r#"<Annotation Term="{t}"{q} String="{val}"/>"#,
                    t = self.term,
                    q = q_attr
                ));
            }
            AnnContent::Bool(val) => {
                x.push_str(&format!(
                    r#"<Annotation Term="{t}"{q} Bool="{val}"/>"#,
                    t = self.term,
                    q = q_attr
                ));
            }
            AnnContent::EnumMember(val) => {
                x.push_str(&format!(
                    r#"<Annotation Term="{t}"{q} EnumMember="{val}"/>"#,
                    t = self.term,
                    q = q_attr
                ));
            }
            AnnContent::PathWithChildren(path, children) => {
                x.push_str(&format!(
                    r#"<Annotation Term="{t}"{q} Path="{path}">"#,
                    t = self.term,
                    q = q_attr
                ));
                for c in children {
                    c.to_xml(x);
                }
                x.push_str("</Annotation>");
            }
            content => {
                x.push_str(&format!(
                    r#"<Annotation Term="{t}"{q}>"#,
                    t = self.term,
                    q = q_attr
                ));
                match content {
                    AnnContent::Record(rec) => rec.to_xml(x),
                    AnnContent::Collection(recs) => {
                        x.push_str("<Collection>");
                        for r in recs {
                            r.to_xml(x);
                        }
                        x.push_str("</Collection>");
                    }
                    AnnContent::PropertyPaths(paths) => {
                        x.push_str("<Collection>");
                        for p in paths {
                            x.push_str(&format!("<PropertyPath>{p}</PropertyPath>"));
                        }
                        x.push_str("</Collection>");
                    }
                    AnnContent::Empty => {}
                    _ => unreachable!(),
                }
                x.push_str("</Annotation>");
            }
        }
    }
}

impl Anns {
    pub fn to_xml(&self, x: &mut String) {
        x.push_str(&format!(r#"<Annotations Target="{}">"#, self.target));
        for ann in &self.annotations {
            ann.to_xml(x);
        }
        x.push_str("</Annotations>");
    }
}

fn anns_to_xml(blocks: &[Anns]) -> String {
    let mut x = String::new();
    for b in blocks {
        b.to_xml(&mut x);
    }
    x
}

// ── Structured builders ────────────────────────────────────────

/// Builds structured annotation blocks for an entity.
pub fn build_annotations(
    entity_type_name: &str,
    def: &AnnotationsDef,
    fields: &[FieldDef],
) -> Vec<Anns> {
    let mut blocks = Vec::new();
    let target = format!("{}.{}", NAMESPACE, entity_type_name);
    let mut anns = Vec::new();

    // ── SelectionFields (derived from FieldDef.searchable) ──
    anns.push(Ann {
        term: "UI.SelectionFields".into(),
        qualifier: None,
        content: AnnContent::PropertyPaths(
            fields
                .iter()
                .filter(|f| f.searchable)
                .map(|f| f.name.into())
                .collect(),
        ),
    });

    // ── LineItem (derived from FieldDef.show_in_list, sorted by list_sort_order) ──
    let mut line_item_fields: Vec<&FieldDef> = fields
        .iter()
        .filter(|f| f.show_in_list)
        .collect();
    line_item_fields.sort_by_key(|f| f.list_sort_order.unwrap_or(u32::MAX));
    let mut records = Vec::new();
    for f in &line_item_fields {
        let record_type = if f.references_entity.is_some() {
            "UI.DataFieldWithIntentBasedNavigation"
        } else {
            "UI.DataField"
        };
        let mut props = vec![
            PV::Path("Value".into(), f.name.into()),
        ];
        if let Some(re) = f.references_entity {
            props.push(PV::Str("SemanticObject".into(), re.into()));
            props.push(PV::Str("Action".into(), "display".into()));
            props.push(PV::Collection("Mapping".into(), vec![Rec {
                record_type: Some("Common.SemanticObjectMappingType".into()),
                props: vec![
                    PV::PropPath("LocalProperty".into(), f.name.into()),
                    PV::Str("SemanticObjectProperty".into(), "ID".into()),
                ],
            }]));
        }
        if let Some(imp) = f.list_importance {
            props.push(PV::EnumMember(
                "![@UI.Importance]".into(),
                format!("UI.ImportanceType/{}", imp),
            ));
        }
        if let Some(crit) = f.list_criticality_path {
            props.push(PV::Path("Criticality".into(), crit.into()));
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
    anns.push(Ann {
        term: "UI.HeaderInfo".into(),
        qualifier: None,
        content: AnnContent::Record(Rec {
            record_type: Some("UI.HeaderInfoType".into()),
            props: vec![
                PV::Str("TypeName".into(), def.header_info.type_name.into()),
                PV::Str(
                    "TypeNamePlural".into(),
                    def.header_info.type_name_plural.into(),
                ),
                PV::Record(
                    "Title".into(),
                    Rec {
                        record_type: Some("UI.DataField".into()),
                        props: vec![PV::Path(
                            "Value".into(),
                            def.header_info.title_path.into(),
                        )],
                    },
                ),
                PV::Record(
                    "Description".into(),
                    Rec {
                        record_type: Some("UI.DataField".into()),
                        props: vec![PV::Path(
                            "Value".into(),
                            def.header_info.description_path.into(),
                        )],
                    },
                ),
            ],
        }),
    });

    // ── HeaderFacets ──
    let mut hf_records = Vec::new();
    for hf in def.header_facets {
        hf_records.push(Rec {
            record_type: Some("UI.ReferenceFacet".into()),
            props: vec![
                PV::AnnotationPath(
                    "Target".into(),
                    format!("@UI.DataPoint#{}", hf.data_point_qualifier),
                ),
                PV::Str("Label".into(), hf.label.into()),
            ],
        });
    }
    anns.push(Ann {
        term: "UI.HeaderFacets".into(),
        qualifier: None,
        content: AnnContent::Collection(hf_records),
    });

    // ── DataPoints ──
    for dp in def.data_points {
        let mut props = vec![
            PV::Path("Value".into(), dp.value_path.into()),
            PV::Str("Title".into(), dp.title.into()),
        ];
        if let Some(max) = dp.max_value {
            props.push(PV::Int("MaximumValue".into(), max));
        }
        if let Some(vis) = dp.visualization {
            props.push(PV::EnumMember(
                "Visualization".into(),
                format!("UI.VisualizationType/{}", vis),
            ));
        }
        anns.push(Ann {
            term: "UI.DataPoint".into(),
            qualifier: Some(dp.qualifier.into()),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.DataPointType".into()),
                props,
            }),
        });
    }

    // ── Facets (Object Page Sections) ──
    let mut facet_records = Vec::new();
    for sec in def.facet_sections {
        facet_records.push(Rec {
            record_type: Some("UI.CollectionFacet".into()),
            props: vec![
                PV::Str("Label".into(), sec.label.into()),
                PV::Str("ID".into(), sec.id.into()),
                PV::Collection(
                    "Facets".into(),
                    vec![Rec {
                        record_type: Some("UI.ReferenceFacet".into()),
                        props: vec![
                            PV::AnnotationPath(
                                "Target".into(),
                                format!("@UI.FieldGroup#{}", sec.field_group_qualifier),
                            ),
                            PV::Str("Label".into(), sec.field_group_label.into()),
                        ],
                    }],
                ),
            ],
        });
    }
    for tf in def.table_facets {
        facet_records.push(Rec {
            record_type: Some("UI.ReferenceFacet".into()),
            props: vec![
                PV::Str("Label".into(), tf.label.into()),
                PV::Str("ID".into(), tf.id.into()),
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

    // ── FieldGroups (derived from FieldDef.form_group) ──
    // Collect unique qualifiers in order of first appearance, then emit one FieldGroup per qualifier.
    let mut seen_qualifiers: Vec<&str> = Vec::new();
    for f in fields {
        if let Some(q) = f.form_group {
            if !seen_qualifiers.contains(&q) {
                seen_qualifiers.push(q);
            }
        }
    }
    for qualifier in &seen_qualifiers {
        let mut fg_records = Vec::new();
        for f in fields {
            if f.form_group == Some(qualifier) {
                let mut props = vec![
                    PV::Path("Value".into(), f.name.into()),
                ];
                let record_type = if let Some(re) = f.references_entity {
                    props.push(PV::Str("SemanticObject".into(), re.into()));
                    props.push(PV::Str("Action".into(), "display".into()));
                    props.push(PV::Collection("Mapping".into(), vec![Rec {
                        record_type: Some("Common.SemanticObjectMappingType".into()),
                        props: vec![
                            PV::PropPath("LocalProperty".into(), f.name.into()),
                            PV::Str("SemanticObjectProperty".into(), "ID".into()),
                        ],
                    }]));
                    "UI.DataFieldWithIntentBasedNavigation"
                } else {
                    "UI.DataField"
                };
                fg_records.push(Rec {
                    record_type: Some(record_type.into()),
                    props,
                });
            }
        }
        anns.push(Ann {
            term: "UI.FieldGroup".into(),
            qualifier: Some((*qualifier).into()),
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

    // ── Property-level Common.SemanticObject + SemanticObjectMapping annotations ──
    for f in fields {
        if let Some(re) = f.references_entity {
            blocks.push(Anns {
                target: format!("{}.{}/{}", NAMESPACE, entity_type_name, f.name),
                annotations: vec![
                    Ann {
                        term: "Common.SemanticObject".into(),
                        qualifier: None,
                        content: AnnContent::Str(re.into()),
                    },
                    Ann {
                        term: "Common.SemanticObjectMapping".into(),
                        qualifier: None,
                        content: AnnContent::Collection(vec![Rec {
                            record_type: None,
                            props: vec![
                                PV::PropPath("LocalProperty".into(), f.name.into()),
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

/// Erzeugt das Annotations-XML fuer eine Entitaet aus ihrer AnnotationsDef.
pub fn build_annotations_xml(
    entity_type_name: &str,
    def: &AnnotationsDef,
    fields: &[FieldDef],
) -> String {
    anns_to_xml(&build_annotations(entity_type_name, def, fields))
}

/// Builds structured ValueList annotation(s) for a field.
fn build_value_list_anns(
    local_property: &str,
    vl: &ValueListDef,
    list_id_filter: Option<&str>,
) -> Vec<Ann> {
    let mut params = vec![Rec {
        record_type: Some("Common.ValueListParameterOut".into()),
        props: vec![
            PV::PropPath("LocalDataProperty".into(), local_property.into()),
            PV::Str("ValueListProperty".into(), vl.key_property.into()),
        ],
    }];
    if let Some(dp) = vl.display_property {
        params.push(Rec {
            record_type: Some("Common.ValueListParameterDisplayOnly".into()),
            props: vec![PV::Str("ValueListProperty".into(), dp.into())],
        });
    }
    if let Some(list_id) = list_id_filter {
        params.push(Rec {
            record_type: Some("Common.ValueListParameterConstant".into()),
            props: vec![
                PV::Str("ValueListProperty".into(), "ListID".into()),
                PV::Str("Constant".into(), list_id.into()),
            ],
        });
    }
    let mut anns = vec![Ann {
        term: "Common.ValueList".into(),
        qualifier: None,
        content: AnnContent::Record(Rec {
            record_type: Some("Common.ValueListType".into()),
            props: vec![
                PV::Str("CollectionPath".into(), vl.collection_path.into()),
                PV::Collection("Parameters".into(), params),
            ],
        }),
    }];
    if vl.fixed_values {
        anns.push(Ann {
            term: "Common.ValueListWithFixedValues".into(),
            qualifier: None,
            content: AnnContent::Bool(true),
        });
    }
    anns
}

/// Builds structured capability annotation blocks for an entity set.
pub fn build_capabilities(
    entity_set_name: &str,
    entity_type_name: &str,
    key_field: &str,
    title_field: Option<&str>,
    fields: &[FieldDef],
    is_draft_root: bool,
) -> Vec<Anns> {
    let mut blocks = Vec::new();

    // ── EntitySet-level annotations ──
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

    // InsertRestrictions
    let mut non_insertable: Vec<String> = fields
        .iter()
        .filter(|f| f.computed)
        .map(|f| f.name.into())
        .collect();
    non_insertable.extend(
        ["IsActiveEntity", "HasActiveEntity", "HasDraftEntity"]
            .iter()
            .map(|&s| s.into()),
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
                        format!("{}.draftActivate", NAMESPACE),
                    ),
                    PV::Str("EditAction".into(), format!("{}.draftEdit", NAMESPACE)),
                    PV::Str(
                        "PreparationAction".into(),
                        format!("{}.draftPrepare", NAMESPACE),
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
                    format!("{}.draftPrepare", NAMESPACE),
                )],
            }),
        });
    }

    blocks.push(Anns {
        target: format!("{}.EntityContainer/{}", NAMESPACE, entity_set_name),
        annotations: set_anns,
    });

    // ── Per-property annotations ──
    for f in fields {
        let mut prop_anns = vec![Ann {
            term: "Common.Label".into(),
            qualifier: None,
            content: AnnContent::Str(f.label.into()),
        }];
        if f.edm_type == "Edm.Guid" && f.computed {
            prop_anns.push(Ann {
                term: "UI.Hidden".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        }
        if f.computed {
            prop_anns.push(Ann {
                term: "Org.OData.Core.V1.Computed".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        } else if f.immutable {
            prop_anns.push(Ann {
                term: "Org.OData.Core.V1.Immutable".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            });
        }
        if f.name == key_field {
            if let Some(tf) = title_field {
                if tf != key_field {
                    prop_anns.push(Ann {
                        term: "Common.Text".into(),
                        qualifier: None,
                        content: AnnContent::PathWithChildren(
                            tf.into(),
                            vec![Ann {
                                term: "UI.TextArrangement".into(),
                                qualifier: None,
                                content: AnnContent::EnumMember(
                                    "UI.TextArrangementType/TextOnly".into(),
                                ),
                            }],
                        ),
                    });
                }
            }
        }
        if let Some(tp) = f.text_path {
            prop_anns.push(Ann {
                term: "Common.Text".into(),
                qualifier: None,
                content: AnnContent::PathWithChildren(
                    tp.into(),
                    vec![Ann {
                        term: "UI.TextArrangement".into(),
                        qualifier: None,
                        content: AnnContent::EnumMember(
                            "UI.TextArrangementType/TextOnly".into(),
                        ),
                    }],
                ),
            });
        }
        if let Some(re) = f.references_entity {
            // FK→entity reference: derive ValueList from references_entity.
            // Display property derived from text_path (e.g. "Customer/CustomerName" → "CustomerName").
            let display = f.text_path.and_then(|tp| tp.rsplit('/').next());
            let vl = ValueListDef {
                collection_path: re,
                key_property: "ID",
                display_property: display,
                fixed_values: !f.prefer_dialog,
            };
            prop_anns.extend(build_value_list_anns(f.name, &vl, None));
        } else if let Some(vs) = f.value_source {
            let classic_vl = ValueListDef {
                collection_path: "FieldValueListItems",
                key_property: "Code",
                display_property: Some("Description"),
                fixed_values: !f.prefer_dialog,
            };
            prop_anns.extend(build_value_list_anns(f.name, &classic_vl, Some(vs)));
        }
        blocks.push(Anns {
            target: format!("{}.{}/{}", NAMESPACE, entity_type_name, f.name),
            annotations: prop_anns,
        });
    }

    // Core.Computed on draft-internal properties
    for draft_prop in ["IsActiveEntity", "HasActiveEntity", "HasDraftEntity"] {
        blocks.push(Anns {
            target: format!("{}.{}/{}", NAMESPACE, entity_type_name, draft_prop),
            annotations: vec![Ann {
                term: "Org.OData.Core.V1.Computed".into(),
                qualifier: None,
                content: AnnContent::Bool(true),
            }],
        });
    }

    blocks
}

/// Erzeugt Capabilities-Annotations fuer ein EntitySet (UpdateRestrictions, DraftRoot/DraftNode).
pub fn build_capabilities_annotations(
    entity_set_name: &str,
    entity_type_name: &str,
    key_field: &str,
    title_field: Option<&str>,
    fields: &[FieldDef],
    is_draft_root: bool,
) -> String {
    anns_to_xml(&build_capabilities(
        entity_set_name,
        entity_type_name,
        key_field,
        title_field,
        fields,
        is_draft_root,
    ))
}

/// Erzeugt das EntityType-XML aus Typ-Name, Schluesselfeld und Property-Definitionen.
/// Fuegt automatisch Draft-Properties (IsActiveEntity, HasActiveEntity, HasDraftEntity)
/// sowie Draft-NavigationProperties (SiblingEntity, DraftAdministrativeData) hinzu.
pub fn build_entity_type_xml(type_name: &str, key_field: &str, props: &[FieldDef]) -> String {
    let mut x = format!("<EntityType Name=\"{}\">", type_name);
    x.push_str("<Key>");
    x.push_str(&format!("<PropertyRef Name=\"{}\"/>", key_field));
    x.push_str("<PropertyRef Name=\"IsActiveEntity\"/>");
    x.push_str("</Key>");
    for p in props {
        let mut attr = format!("Type=\"{}\"", p.edm_type);
        if p.name == key_field {
            attr.push_str(" Nullable=\"false\"");
        }
        if let Some(ml) = p.max_length {
            attr.push_str(&format!(" MaxLength=\"{}\"", ml));
        }
        if let Some(prec) = p.precision {
            attr.push_str(&format!(" Precision=\"{}\"", prec));
        }
        if let Some(sc) = p.scale {
            attr.push_str(&format!(" Scale=\"{}\"", sc));
        }
        // Padding fuer lesbare Ausrichtung
        let pad = if p.name.len() < 18 {
            " ".repeat(18 - p.name.len())
        } else {
            " ".to_string()
        };
        x.push_str(&format!("<Property Name=\"{}\"{}{}/>", p.name, pad, attr));
    }
    // Draft-Properties
    x.push_str("<Property Name=\"IsActiveEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"true\"/>");
    x.push_str("<Property Name=\"HasActiveEntity\"  Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");
    x.push_str("<Property Name=\"HasDraftEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");
    // Draft-NavigationProperties
    x.push_str(&format!(
        "<NavigationProperty Name=\"SiblingEntity\" Type=\"{ns}.{ty}\"/>",
        ns = NAMESPACE,
        ty = type_name
    ));
    x.push_str(&format!(
        "<NavigationProperty Name=\"DraftAdministrativeData\" Type=\"{ns}.DraftAdministrativeData\" ContainsTarget=\"true\"/>",
        ns = NAMESPACE
    ));
    x.push_str("</EntityType>");
    x
}

/// Erzeugt das DraftAdministrativeData EntityType-XML.
pub fn build_draft_admin_type_xml() -> String {
    let mut x = String::from("<EntityType Name=\"DraftAdministrativeData\">");
    x.push_str("<Key><PropertyRef Name=\"DraftUUID\"/></Key>");
    x.push_str("<Property Name=\"DraftUUID\"              Type=\"Edm.Guid\" Nullable=\"false\"/>");
    x.push_str(
        "<Property Name=\"CreationDateTime\"       Type=\"Edm.DateTimeOffset\" Precision=\"7\"/>",
    );
    x.push_str("<Property Name=\"CreatedByUser\"          Type=\"Edm.String\" MaxLength=\"256\"/>");
    x.push_str("<Property Name=\"DraftIsCreatedByMe\"     Type=\"Edm.Boolean\"/>");
    x.push_str(
        "<Property Name=\"LastChangeDateTime\"     Type=\"Edm.DateTimeOffset\" Precision=\"7\"/>",
    );
    x.push_str("<Property Name=\"LastChangedByUser\"      Type=\"Edm.String\" MaxLength=\"256\"/>");
    x.push_str("<Property Name=\"InProcessByUser\"        Type=\"Edm.String\" MaxLength=\"256\"/>");
    x.push_str("<Property Name=\"DraftIsProcessedByMe\"   Type=\"Edm.Boolean\"/>");
    x.push_str("</EntityType>");
    x
}

/// Erzeugt die gebundenen Draft-Actions (draftEdit, draftActivate, draftPrepare)
/// fuer einen Entity-Typ.
pub fn build_draft_actions_xml(type_name: &str) -> String {
    let fqn = format!("{}.{}", NAMESPACE, type_name);
    let mut x = String::new();
    // draftEdit
    x.push_str(&format!(
        "<Action Name=\"draftEdit\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <Parameter Name=\"PreserveChanges\" Type=\"Edm.Boolean\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    // draftActivate
    x.push_str(&format!(
        "<Action Name=\"draftActivate\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    // draftPrepare
    x.push_str(&format!(
        "<Action Name=\"draftPrepare\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <Parameter Name=\"SideEffectsQualifier\" Type=\"Edm.String\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    x
}

/// Haengt NavigationProperty-Elemente an einen EntityType-XML-String an.
pub fn append_navigation_properties(xml: &mut String, nav_props: &[NavigationPropertyDef]) {
    for np in nav_props {
        let type_attr = if np.is_collection {
            format!("Collection({}.{})", NAMESPACE, np.target_type)
        } else {
            format!("{}.{}", NAMESPACE, np.target_type)
        };
        xml.insert_str(
            xml.rfind("</EntityType>").unwrap(),
            &format!(
                "<NavigationProperty Name=\"{}\" Type=\"{}\"/>",
                np.name, type_attr
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ─────────────────────────────────────────────────

    fn simple_fields() -> Vec<FieldDef> {
        vec![
            FieldDef {
                name: "ProductID",
                label: "Product Nr.",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
                immutable: true,
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
                form_group: Some("Main"),
            },
            FieldDef {
                name: "ProductName",
                label: "Product Name",
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
                searchable: true,
                show_in_list: true,
                list_sort_order: Some(1),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Main"),
            },
            FieldDef {
                name: "Price",
                label: "Price",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(15),
                scale: Some(2),
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
                form_group: Some("Main"),
            },
        ]
    }

    fn simple_annotations() -> AnnotationsDef {
        AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Product",
                type_name_plural: "Products",
                title_path: "ProductName",
                description_path: "ProductID",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[FacetSectionDef {
                label: "General",
                id: "GeneralSection",
                field_group_qualifier: "Main",
                field_group_label: "Main Data",
            }],
            table_facets: &[],
        }
    }

    // ── build_annotations_xml ───────────────────────────────────

    #[test]
    fn annotations_xml_contains_target() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("Annotations Target=\"ProductsService.Product\""));
    }

    #[test]
    fn annotations_xml_selection_fields() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("<Annotation Term=\"UI.SelectionFields\">"));
        assert!(xml.contains("<PropertyPath>ProductName</PropertyPath>"));
    }

    #[test]
    fn annotations_xml_line_item_basic() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("<Annotation Term=\"UI.LineItem\">"));
        // ProductID field: value path from FieldDef
        assert!(xml.contains("Property=\"Value\" Path=\"ProductID\""));
        // Label comes from Common.Label on the property, not from DataField
        // High importance
        assert!(xml.contains("EnumMember=\"UI.ImportanceType/High\""));
    }

    #[test]
    fn annotations_xml_line_item_with_semantic_object() {
        let fields = vec![FieldDef {
            name: "CustomerID",
            label: "Customer",
            edm_type: "Edm.String",
            max_length: None,
            precision: None,
            scale: None,
            immutable: false,
            computed: false,
            references_entity: Some("Customers"),
            value_source: None,
            prefer_dialog: false,
            text_path: None,
            searchable: false,
            show_in_list: true,
            list_sort_order: Some(0),
            list_importance: None,
            list_criticality_path: None,
            form_group: None,
        }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("Record Type=\"UI.DataFieldWithIntentBasedNavigation\""));
        assert!(xml.contains("Property=\"SemanticObject\" String=\"Customers\""));
        assert!(xml.contains("Property=\"Action\" String=\"display\""));
    }

    #[test]
    fn annotations_xml_line_item_with_criticality() {
        let fields = vec![FieldDef {
            name: "Status",
            label: "Status",
            edm_type: "Edm.String",
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
            list_importance: None,
            list_criticality_path: Some("StatusCriticality"),
            form_group: None,
        }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("Property=\"Criticality\" Path=\"StatusCriticality\""));
    }

    #[test]
    fn annotations_xml_header_info() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("<Annotation Term=\"UI.HeaderInfo\">"));
        assert!(xml.contains("Property=\"TypeName\" String=\"Product\""));
        assert!(xml.contains("Property=\"TypeNamePlural\" String=\"Products\""));
        assert!(xml.contains("Property=\"Value\" Path=\"ProductName\""));
        assert!(xml.contains("Property=\"Value\" Path=\"ProductID\""));
    }

    #[test]
    fn annotations_xml_header_facets_empty() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("<Annotation Term=\"UI.HeaderFacets\">"));
        assert!(xml.contains("<Collection></Collection>"));
    }

    #[test]
    fn annotations_xml_header_facets_with_data() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[HeaderFacetDef {
                data_point_qualifier: "Rating",
                label: "Bewertung",
            }],
            data_points: &[DataPointDef {
                qualifier: "Rating",
                value_path: "RatingValue",
                title: "Bewertung",
                max_value: Some(5),
                visualization: Some("Rating"),
            }],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("AnnotationPath=\"@UI.DataPoint#Rating\""));
        assert!(xml.contains("Property=\"Label\" String=\"Bewertung\""));
        // DataPoint
        assert!(xml.contains("Annotation Term=\"UI.DataPoint\" Qualifier=\"Rating\""));
        assert!(xml.contains("Property=\"Value\" Path=\"RatingValue\""));
        assert!(xml.contains("Property=\"Title\" String=\"Bewertung\""));
        assert!(xml.contains("Property=\"MaximumValue\" Int=\"5\""));
        assert!(xml.contains("EnumMember=\"UI.VisualizationType/Rating\""));
    }

    #[test]
    fn annotations_xml_data_point_minimal() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[DataPointDef {
                qualifier: "Amount",
                value_path: "TotalAmount",
                title: "Gesamtbetrag",
                max_value: None,
                visualization: None,
            }],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Qualifier=\"Amount\""));
        assert!(xml.contains("Property=\"Value\" Path=\"TotalAmount\""));
        assert!(!xml.contains("MaximumValue"));
        assert!(!xml.contains("VisualizationType"));
    }

    #[test]
    fn annotations_xml_facet_sections() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("<Annotation Term=\"UI.Facets\">"));
        assert!(xml.contains("Record Type=\"UI.CollectionFacet\""));
        assert!(xml.contains("Property=\"Label\" String=\"General\""));
        assert!(xml.contains("Property=\"ID\" String=\"GeneralSection\""));
        assert!(xml.contains("AnnotationPath=\"@UI.FieldGroup#Main\""));
        assert!(xml.contains("Property=\"Label\" String=\"Main Data\""));
    }

    #[test]
    fn annotations_xml_table_facets() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[TableFacetDef {
                label: "Positionen",
                id: "ItemsFacet",
                navigation_property: "Items",
            }],
        };
        let xml = build_annotations_xml("Order", &def, &[]);
        assert!(xml.contains("Record Type=\"UI.ReferenceFacet\""));
        assert!(xml.contains("Property=\"Label\" String=\"Positionen\""));
        assert!(xml.contains("Property=\"ID\" String=\"ItemsFacet\""));
        assert!(xml.contains("AnnotationPath=\"Items/@UI.LineItem\""));
    }

    #[test]
    fn annotations_xml_field_groups() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(xml.contains("Annotation Term=\"UI.FieldGroup\" Qualifier=\"Main\""));
        assert!(xml.contains("Record Type=\"UI.FieldGroupType\""));
        // Fields should use value paths from FieldDef (labels come from Common.Label)
        assert!(xml.contains("Property=\"Value\" Path=\"ProductID\""));
        assert!(xml.contains("Property=\"Value\" Path=\"ProductName\""));
        assert!(xml.contains("Property=\"Value\" Path=\"Price\""));
    }

    #[test]
    fn annotations_xml_field_group_with_semantic_object() {
        let fields = vec![FieldDef {
            name: "CustomerID",
            label: "Customer",
            edm_type: "Edm.String",
            max_length: None,
            precision: None,
            scale: None,
            immutable: false,
            computed: false,
            references_entity: Some("Customers"),
            value_source: None,
            prefer_dialog: false,
            text_path: None,
            searchable: false,
            show_in_list: false,
            list_sort_order: None,
            list_importance: None,
            list_criticality_path: None,
            form_group: Some("Main"),
        }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Contact", &def, &fields);
        // FieldGroup should use DataFieldWithIntentBasedNavigation
        assert!(xml.contains("Record Type=\"UI.DataFieldWithIntentBasedNavigation\""));
        assert!(xml.contains("Property=\"SemanticObject\" String=\"Customers\""));
        assert!(xml.contains("Property=\"Action\" String=\"display\""));
    }

    #[test]
    fn annotations_xml_semantic_object_property_level() {
        let fields = vec![FieldDef {
            name: "CustomerID",
            label: "Customer",
            edm_type: "Edm.String",
            max_length: None,
            precision: None,
            scale: None,
            immutable: false,
            computed: false,
            references_entity: Some("Customers"),
            value_source: None,
            prefer_dialog: false,
            text_path: None,
        searchable: false,
        show_in_list: false,
        list_sort_order: None,
        list_importance: None,
        list_criticality_path: None,
        form_group: None,
    }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Contact", &def, &fields);
        assert!(xml.contains("Annotations Target=\"ProductsService.Contact/CustomerID\""));
        assert!(xml.contains("Annotation Term=\"Common.SemanticObject\" String=\"Customers\""));
    }

    #[test]
    fn annotations_xml_closes_properly() {
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        // The main Annotations block should be closed
        assert!(xml.starts_with("<Annotations Target="));
        // Should end with a closing tag
        assert!(xml.ends_with("</Annotations>"));
    }

    // ── build_capabilities_annotations ──────────────────────────

    #[test]
    fn capabilities_entity_config() {
        let xml = build_capabilities_annotations(
            "EntityConfigs",
            "EntityConfig",
            "EntityID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        assert!(xml.contains("Annotations Target=\"Products"));
    }

    #[test]
    fn capabilities_draft_root() {
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        assert!(xml.contains("Annotations Target=\"ProductsService.EntityContainer/Products\""));
        assert!(xml.contains("Org.OData.Capabilities.V1.UpdateRestrictions"));
        assert!(xml.contains("Property=\"Updatable\" Bool=\"true\""));
        assert!(xml.contains("Annotation Term=\"Common.DraftRoot\""));
        assert!(xml.contains("Record Type=\"Common.DraftRootType\""));
        assert!(
            xml.contains("Property=\"ActivationAction\" String=\"ProductsService.draftActivate\"")
        );
        assert!(xml.contains("Property=\"EditAction\" String=\"ProductsService.draftEdit\""));
        assert!(
            xml.contains("Property=\"PreparationAction\" String=\"ProductsService.draftPrepare\"")
        );
        assert!(!xml.contains("Common.DraftNode"));
    }

    #[test]
    fn capabilities_draft_node() {
        let xml = build_capabilities_annotations(
            "OrderItems",
            "OrderItem",
            "ProductID",
            None,
            &simple_fields(),
            false,
        );
        assert!(xml.contains("Annotations Target=\"ProductsService.EntityContainer/OrderItems\""));
        assert!(xml.contains("Annotation Term=\"Common.DraftNode\""));
        assert!(xml.contains("Record Type=\"Common.DraftNodeType\""));
        assert!(
            xml.contains("Property=\"PreparationAction\" String=\"ProductsService.draftPrepare\"")
        );
        assert!(!xml.contains("Common.DraftRoot"));
        assert!(!xml.contains("EditAction"));
        assert!(!xml.contains("ActivationAction"));
    }

    #[test]
    fn capabilities_per_property_labels() {
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/ProductID\""));
        assert!(xml.contains("Common.Label\" String=\"Product Nr.\""));
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/ProductName\""));
        assert!(xml.contains("Common.Label\" String=\"Product Name\""));
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/Price\""));
        assert!(xml.contains("Common.Label\" String=\"Price\""));
    }

    #[test]
    fn capabilities_immutable_annotation() {
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        // ProductID is immutable
        let product_id_section = &xml[xml.find("Product/ProductID").unwrap()..];
        let section_end = product_id_section.find("</Annotations>").unwrap();
        let section = &product_id_section[..section_end];
        assert!(section.contains("Org.OData.Core.V1.Immutable\" Bool=\"true\""));

        // ProductName is NOT immutable
        let product_name_section = &xml[xml.find("Product/ProductName").unwrap()..];
        let section_end = product_name_section.find("</Annotations>").unwrap();
        let section = &product_name_section[..section_end];
        assert!(!section.contains("Immutable"));
    }

    #[test]
    fn capabilities_insert_restrictions() {
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        // Even without computed fields, InsertRestrictions must be emitted for draft properties
        assert!(xml.contains("Org.OData.Capabilities.V1.InsertRestrictions"));
        assert!(xml.contains("<PropertyPath>IsActiveEntity</PropertyPath>"));
        assert!(xml.contains("<PropertyPath>HasActiveEntity</PropertyPath>"));
        assert!(xml.contains("<PropertyPath>HasDraftEntity</PropertyPath>"));
        // No computed fields → only draft properties in NonInsertableProperties
        assert!(!xml.contains("<PropertyPath>ProductID</PropertyPath>"));

        // With a computed field, InsertRestrictions should include it alongside draft properties
        let mut fields = simple_fields();
        fields[0].computed = true; // make ProductID computed
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &fields,
            true,
        );
        assert!(xml.contains("Org.OData.Capabilities.V1.InsertRestrictions"));
        assert!(xml.contains("Capabilities.InsertRestrictionsType"));
        assert!(xml.contains("NonInsertableProperties"));
        assert!(xml.contains("<PropertyPath>ProductID</PropertyPath>"));
        // ProductName and Price are NOT computed, should not be in NonInsertableProperties
        assert!(!xml.contains("<PropertyPath>ProductName</PropertyPath>"));
        assert!(!xml.contains("<PropertyPath>Price</PropertyPath>"));
    }

    // ── build_entity_type_xml ───────────────────────────────────

    #[test]
    fn entity_type_xml_basic_structure() {
        let xml = build_entity_type_xml("Product", "ProductID", &simple_fields());
        assert!(xml.starts_with("<EntityType Name=\"Product\">"));
        assert!(xml.ends_with("</EntityType>"));
        assert!(xml.contains("<Key>"));
        assert!(xml.contains("<PropertyRef Name=\"ProductID\"/>"));
        assert!(xml.contains("<PropertyRef Name=\"IsActiveEntity\"/>"));
    }

    #[test]
    fn entity_type_xml_properties() {
        let xml = build_entity_type_xml("Product", "ProductID", &simple_fields());
        // Key field: Nullable=false
        assert!(xml.contains("Name=\"ProductID\""));
        assert!(xml.contains("Type=\"Edm.String\" Nullable=\"false\" MaxLength=\"10\""));
        // Normal field
        assert!(xml.contains("Name=\"ProductName\""));
        assert!(xml.contains("Type=\"Edm.String\" MaxLength=\"80\""));
        // Decimal field with precision/scale
        assert!(xml.contains("Name=\"Price\""));
        assert!(xml.contains("Type=\"Edm.Decimal\" Precision=\"15\" Scale=\"2\""));
    }

    #[test]
    fn entity_type_xml_draft_properties() {
        let xml = build_entity_type_xml("Product", "ProductID", &simple_fields());
        assert!(xml.contains("Name=\"IsActiveEntity\""));
        assert!(xml.contains("Name=\"HasActiveEntity\""));
        assert!(xml.contains("Name=\"HasDraftEntity\""));
    }

    #[test]
    fn entity_type_xml_draft_navigation_properties() {
        let xml = build_entity_type_xml("Product", "ProductID", &simple_fields());
        assert!(xml.contains(
            "NavigationProperty Name=\"SiblingEntity\" Type=\"ProductsService.Product\""
        ));
        assert!(xml.contains("NavigationProperty Name=\"DraftAdministrativeData\" Type=\"ProductsService.DraftAdministrativeData\""));
        assert!(xml.contains("ContainsTarget=\"true\""));
    }

    #[test]
    fn entity_type_xml_empty_fields() {
        let xml = build_entity_type_xml("Empty", "ID", &[]);
        assert!(xml.contains("<EntityType Name=\"Empty\">"));
        assert!(xml.contains("<PropertyRef Name=\"ID\"/>"));
        // Should still have draft properties
        assert!(xml.contains("IsActiveEntity"));
        assert!(xml.contains("SiblingEntity"));
    }

    // ── build_draft_admin_type_xml ──────────────────────────────

    #[test]
    fn draft_admin_type_xml() {
        let xml = build_draft_admin_type_xml();
        assert!(xml.contains("EntityType Name=\"DraftAdministrativeData\""));
        assert!(xml.contains("PropertyRef Name=\"DraftUUID\""));
        assert!(xml.contains("Name=\"DraftUUID\""));
        assert!(xml.contains("Name=\"CreationDateTime\""));
        assert!(xml.contains("Name=\"CreatedByUser\""));
        assert!(xml.contains("Name=\"DraftIsCreatedByMe\""));
        assert!(xml.contains("Name=\"LastChangeDateTime\""));
        assert!(xml.contains("Name=\"LastChangedByUser\""));
        assert!(xml.contains("Name=\"InProcessByUser\""));
        assert!(xml.contains("Name=\"DraftIsProcessedByMe\""));
    }

    // ── build_draft_actions_xml ─────────────────────────────────

    #[test]
    fn draft_actions_xml() {
        let xml = build_draft_actions_xml("Product");
        assert!(xml.contains("Action Name=\"draftEdit\" IsBound=\"true\""));
        assert!(xml.contains("Parameter Name=\"in\" Type=\"ProductsService.Product\""));
        assert!(xml.contains("Parameter Name=\"PreserveChanges\" Type=\"Edm.Boolean\""));
        assert!(xml.contains("ReturnType Type=\"ProductsService.Product\""));

        assert!(xml.contains("Action Name=\"draftActivate\" IsBound=\"true\""));
        assert!(xml.contains("Action Name=\"draftPrepare\" IsBound=\"true\""));
        assert!(xml.contains("Parameter Name=\"SideEffectsQualifier\" Type=\"Edm.String\""));
    }

    // ── append_navigation_properties ────────────────────────────

    #[test]
    fn append_nav_props_collection() {
        let mut xml = "<EntityType Name=\"Order\"></EntityType>".to_string();
        let navs = vec![NavigationPropertyDef {
            name: "Items",
            target_type: "OrderItem",
            is_collection: true,
            foreign_key: None,
        }];
        append_navigation_properties(&mut xml, &navs);
        assert!(xml.contains(
            "NavigationProperty Name=\"Items\" Type=\"Collection(ProductsService.OrderItem)\""
        ));
    }

    #[test]
    fn append_nav_props_single() {
        let mut xml = "<EntityType Name=\"Contact\"></EntityType>".to_string();
        let navs = vec![NavigationPropertyDef {
            name: "Customer",
            target_type: "Customer",
            is_collection: false,
            foreign_key: None,
        }];
        append_navigation_properties(&mut xml, &navs);
        assert!(
            xml.contains("NavigationProperty Name=\"Customer\" Type=\"ProductsService.Customer\"")
        );
        assert!(!xml.contains("Collection("));
    }

    #[test]
    fn append_nav_props_multiple() {
        let mut xml = "<EntityType Name=\"Order\"></EntityType>".to_string();
        let navs = vec![
            NavigationPropertyDef {
                name: "Items",
                target_type: "OrderItem",
                is_collection: true,
                foreign_key: None,
            },
            NavigationPropertyDef {
                name: "Customer",
                target_type: "Customer",
                is_collection: false,
                foreign_key: None,
            },
        ];
        append_navigation_properties(&mut xml, &navs);
        assert!(xml.contains("Name=\"Items\""));
        assert!(xml.contains("Name=\"Customer\""));
        // Both should be before </EntityType>
        assert!(xml.ends_with("</EntityType>"));
    }

    #[test]
    fn append_nav_props_empty() {
        let original = "<EntityType Name=\"Simple\"></EntityType>".to_string();
        let mut xml = original.clone();
        append_navigation_properties(&mut xml, &[]);
        assert_eq!(xml, original);
    }

    // ── build_annotations_xml: additional coverage ──────────────

    #[test]
    fn annotations_xml_multiple_selection_fields() {
        let fields = vec![
            FieldDef { name: "ProductName", label: "Name", edm_type: "Edm.String", max_length: None, precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: true, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
            FieldDef { name: "Price", label: "Price", edm_type: "Edm.Decimal", max_length: None, precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: true, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
            FieldDef { name: "Category", label: "Category", edm_type: "Edm.String", max_length: None, precision: None, scale: None, immutable: false, computed: false, references_entity: None, value_source: None, prefer_dialog: false, text_path: None, searchable: true, show_in_list: false, list_sort_order: None, list_importance: None, list_criticality_path: None, form_group: None },
        ];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("<PropertyPath>ProductName</PropertyPath>"));
        assert!(xml.contains("<PropertyPath>Price</PropertyPath>"));
        assert!(xml.contains("<PropertyPath>Category</PropertyPath>"));
    }

    #[test]
    fn annotations_xml_empty_selection_fields() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("<Annotation Term=\"UI.SelectionFields\">"));
        assert!(xml.contains("<Collection></Collection>"));
    }

    #[test]
    fn annotations_xml_semantic_object_takes_precedence_over_navigation_path() {
        // semantic_object on FieldDef generates IntentBasedNavigation in LineItem
        let fields = vec![FieldDef {
            name: "CustomerID", label: "Customer", edm_type: "Edm.String", max_length: None, precision: None, scale: None, immutable: false, computed: false, references_entity: Some("Customers"), value_source: None, prefer_dialog: false, text_path: None, searchable: false, show_in_list: true, list_sort_order: Some(0), list_importance: None, list_criticality_path: None, form_group: None,
        }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("Record Type=\"UI.DataFieldWithIntentBasedNavigation\""));
        assert!(!xml.contains("Record Type=\"UI.DataFieldWithNavigationPath\""));
    }

    #[test]
    fn annotations_xml_multiple_data_points() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[
                DataPointDef {
                    qualifier: "Rating",
                    value_path: "RatingValue",
                    title: "Bewertung",
                    max_value: Some(5),
                    visualization: Some("Rating"),
                },
                DataPointDef {
                    qualifier: "Progress",
                    value_path: "Completion",
                    title: "Fortschritt",
                    max_value: Some(100),
                    visualization: Some("Progress"),
                },
            ],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Qualifier=\"Rating\""));
        assert!(xml.contains("Qualifier=\"Progress\""));
        assert!(xml.contains("Property=\"Value\" Path=\"RatingValue\""));
        assert!(xml.contains("Property=\"Value\" Path=\"Completion\""));
        assert!(xml.contains("Property=\"MaximumValue\" Int=\"5\""));
        assert!(xml.contains("Property=\"MaximumValue\" Int=\"100\""));
        assert!(xml.contains("VisualizationType/Rating"));
        assert!(xml.contains("VisualizationType/Progress"));
    }

    #[test]
    fn annotations_xml_multiple_field_groups() {
        let fields = vec![
            FieldDef {
                name: "Name",
                label: "Name",
                edm_type: "Edm.String",
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
                form_group: Some("General"),
            },
            FieldDef {
                name: "Price",
                label: "Price",
                edm_type: "Edm.Decimal",
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
                form_group: Some("Pricing"),
            },
        ];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("Qualifier=\"General\""));
        assert!(xml.contains("Qualifier=\"Pricing\""));
    }

    #[test]
    fn annotations_xml_multiple_facet_sections() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef {
                    label: "General",
                    id: "GeneralSection",
                    field_group_qualifier: "Main",
                    field_group_label: "Main Data",
                },
                FacetSectionDef {
                    label: "Details",
                    id: "DetailsSection",
                    field_group_qualifier: "Detail",
                    field_group_label: "Detail Data",
                },
            ],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Property=\"ID\" String=\"GeneralSection\""));
        assert!(xml.contains("Property=\"ID\" String=\"DetailsSection\""));
        assert!(xml.contains("AnnotationPath=\"@UI.FieldGroup#Main\""));
        assert!(xml.contains("AnnotationPath=\"@UI.FieldGroup#Detail\""));
    }

    #[test]
    fn annotations_xml_mixed_facets_and_table_facets() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[FacetSectionDef {
                label: "General",
                id: "GeneralSection",
                field_group_qualifier: "Main",
                field_group_label: "Main",
            }],
            table_facets: &[TableFacetDef {
                label: "Items",
                id: "ItemsFacet",
                navigation_property: "Items",
            }],
        };
        let xml = build_annotations_xml("Order", &def, &[]);
        // Both should be inside the UI.Facets collection
        assert!(xml.contains("Record Type=\"UI.CollectionFacet\""));
        assert!(xml.contains("AnnotationPath=\"Items/@UI.LineItem\""));
    }

    #[test]
    fn annotations_xml_multiple_header_facets() {
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[
                HeaderFacetDef {
                    data_point_qualifier: "Rating",
                    label: "Bewertung",
                },
                HeaderFacetDef {
                    data_point_qualifier: "Price",
                    label: "Preis",
                },
            ],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("AnnotationPath=\"@UI.DataPoint#Rating\""));
        assert!(xml.contains("AnnotationPath=\"@UI.DataPoint#Price\""));
        assert!(xml.contains("Property=\"Label\" String=\"Bewertung\""));
        assert!(xml.contains("Property=\"Label\" String=\"Preis\""));
    }

    #[test]
    fn annotations_xml_field_group_unknown_field_uses_name_as_label() {
        // Field label from FieldDef is used directly
        let fields = vec![FieldDef {
            name: "UnknownField",
            label: "UnknownField",
            edm_type: "Edm.String",
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
            form_group: Some("Main"),
        }];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &fields);
        assert!(xml.contains("Property=\"Value\" Path=\"UnknownField\""));
    }

    #[test]
    fn annotations_xml_no_semantic_object_property_annotations() {
        // Fields without semantic_object should not produce property-level annotations
        let xml = build_annotations_xml("Product", &simple_annotations(), &simple_fields());
        assert!(!xml.contains("Common.SemanticObject"));
    }

    #[test]
    fn annotations_xml_multiple_semantic_object_property_annotations() {
        let fields = vec![
            FieldDef {
                name: "CustomerID",
                label: "Customer",
                edm_type: "Edm.String",
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: Some("Customers"),
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
                name: "ProductID",
                label: "Product",
                edm_type: "Edm.String",
                max_length: None,
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: Some("Products"),
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
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "T",
                type_name_plural: "Ts",
                title_path: "X",
                description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Order", &def, &fields);
        assert!(xml.contains("Target=\"ProductsService.Order/CustomerID\""));
        assert!(xml.contains("Common.SemanticObject\" String=\"Customers\""));
        assert!(xml.contains("Target=\"ProductsService.Order/ProductID\""));
        assert!(xml.contains("Common.SemanticObject\" String=\"Products\""));
    }

    // ── build_capabilities_annotations: additional coverage ─────

    #[test]
    fn capabilities_no_immutable_fields() {
        let fields = vec![FieldDef {
            name: "Name",
            label: "Name",
            edm_type: "Edm.String",
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
        form_group: None,
    }];
        let xml = build_capabilities_annotations("Tests", "Test", "Name", None, &fields, true);
        assert!(!xml.contains("Immutable"));
        assert!(xml.contains("Common.Label\" String=\"Name\""));
    }

    #[test]
    fn capabilities_empty_fields() {
        let xml = build_capabilities_annotations("Tests", "Test", "ID", None, &[], true);
        // Should still have EntitySet-level annotations
        assert!(xml.contains("Target=\"ProductsService.EntityContainer/Tests\""));
        assert!(xml.contains("Common.DraftRoot"));
        // No property-level annotations
        assert!(!xml.contains("Common.Label"));
    }

    #[test]
    fn capabilities_multiple_immutable_fields() {
        let fields = vec![
            FieldDef {
                name: "ID",
                label: "ID",
                edm_type: "Edm.String",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
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
                name: "Code",
                label: "Code",
                edm_type: "Edm.String",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
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
                name: "Name",
                label: "Name",
                edm_type: "Edm.String",
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
            form_group: None,
        },
        ];
        let xml = build_capabilities_annotations("Tests", "Test", "ID", None, &fields, true);
        // Count occurrences of Immutable
        let count = xml.matches("Org.OData.Core.V1.Immutable").count();
        assert_eq!(count, 2, "Expected exactly 2 immutable annotations");
    }

    // ── build_entity_type_xml: additional coverage ──────────────

    #[test]
    fn entity_type_xml_field_without_optional_attributes() {
        let fields = vec![FieldDef {
            name: "Description",
            label: "Description",
            edm_type: "Edm.String",
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
        form_group: None,
    }];
        let xml = build_entity_type_xml("Simple", "ID", &fields);
        assert!(xml.contains("Name=\"Description\""));
        assert!(xml.contains("Type=\"Edm.String\""));
        // Should NOT have MaxLength, Precision, or Scale
        let desc_section = &xml[xml.find("Name=\"Description\"").unwrap()..];
        let end = desc_section.find("/>").unwrap();
        let prop = &desc_section[..end];
        assert!(!prop.contains("MaxLength"));
        assert!(!prop.contains("Precision"));
        assert!(!prop.contains("Scale"));
    }

    #[test]
    fn entity_type_xml_key_field_not_nullable() {
        let fields = vec![
            FieldDef {
                name: "OrderID",
                label: "Order",
                edm_type: "Edm.Int32",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
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
                name: "Description",
                label: "Desc",
                edm_type: "Edm.String",
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
            form_group: None,
        },
        ];
        let xml = build_entity_type_xml("Order", "OrderID", &fields);
        // Key field (same name in both fields and key_field param) => Nullable="false"
        assert!(xml.contains("Name=\"OrderID\""));
        assert!(xml.contains("Nullable=\"false\""));
        // Non-key field should NOT have Nullable="false"
        let desc_section = &xml[xml.find("Name=\"Description\"").unwrap()..];
        let end = desc_section.find("/>").unwrap();
        let prop = &desc_section[..end];
        assert!(!prop.contains("Nullable"));
    }

    #[test]
    fn entity_type_xml_non_key_field_is_nullable() {
        let fields = vec![FieldDef {
            name: "Description",
            label: "Desc",
            edm_type: "Edm.String",
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
        form_group: None,
    }];
        let xml = build_entity_type_xml("Test", "ID", &fields);
        let desc_section = &xml[xml.find("Name=\"Description\"").unwrap()..];
        let end = desc_section.find("/>").unwrap();
        let prop = &desc_section[..end];
        assert!(!prop.contains("Nullable"));
    }

    // ── build_draft_actions_xml: additional coverage ─────────────

    #[test]
    fn draft_actions_xml_uses_correct_namespace() {
        let xml = build_draft_actions_xml("Order");
        assert!(xml.contains("Type=\"ProductsService.Order\""));
        assert!(!xml.contains("Type=\"ProductsService.Product\""));
    }

    #[test]
    fn draft_actions_xml_all_three_actions_present() {
        let xml = build_draft_actions_xml("Test");
        let edit_count = xml.matches("Name=\"draftEdit\"").count();
        let activate_count = xml.matches("Name=\"draftActivate\"").count();
        let prepare_count = xml.matches("Name=\"draftPrepare\"").count();
        assert_eq!(edit_count, 1);
        assert_eq!(activate_count, 1);
        assert_eq!(prepare_count, 1);
    }

    // ── Comprehensive integration-style test ────────────────────

    #[test]
    fn annotation_partners_example() {
        let fields = vec![
            FieldDef {
                name: "ID",
                label: "ID",
                edm_type: "Edm.GUID",
                max_length: Some(10),
                precision: None,
                scale: None,
                immutable: true,
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
                name: "Name",
                label: "Name",
                edm_type: "Edm.String",
                max_length: Some(100),
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
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Main"),
            },
        ];
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Partner",
                type_name_plural: "Partners",
                title_path: "Name",
                description_path: "Name",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[
                FacetSectionDef {
                    label: "General",
                    id: "GeneralSection",
                    field_group_qualifier: "Main",
                    field_group_label: "Partner Data",
                },
            ],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Partner", &def, &fields);
        let expected_parts = vec![ "",
            r#"Annotations Target="ProductsService.Partner">"#,
                r#"Annotation Term="UI.SelectionFields">"#,
                    r#"Collection>"#,
                        r#"PropertyPath>Name"#,
                        r#"/PropertyPath>"#,
                    r#"/Collection>"#,
                r#"/Annotation>"#,
                r#"Annotation Term="UI.LineItem">"#,
                    r#"Collection>"#,
                        r#"Record Type="UI.DataField">"#,
                            r#"PropertyValue Property="Value" Path="Name"/>"#,
                        r#"/Record>"#,
                    r#"/Collection>"#,
                r#"/Annotation>"#,
                r#"Annotation Term="UI.HeaderInfo">"#,
                    r#"Record Type="UI.HeaderInfoType">"#,
                        r#"PropertyValue Property="TypeName" String="Partner"/>"#,
                        r#"PropertyValue Property="TypeNamePlural" String="Partners"/>"#,
                        r#"PropertyValue Property="Title">"#,
                            r#"Record Type="UI.DataField">"#,
                                r#"PropertyValue Property="Value" Path="Name"/>"#,
                            r#"/Record>"#,
                        r#"/PropertyValue>"#,
                        r#"PropertyValue Property="Description">"#,
                            r#"Record Type="UI.DataField">"#,
                                r#"PropertyValue Property="Value" Path="Name"/>"#,
                            r#"/Record>"#,
                        r#"/PropertyValue>"#,
                    r#"/Record>"#,
                r#"/Annotation>"#,
                r#"Annotation Term="UI.HeaderFacets">"#,
                    r#"Collection>"#,
                    r#"/Collection>"#,
                r#"/Annotation>"#,
                r#"Annotation Term="UI.Facets">"#,
                    r#"Collection>"#,
                        r#"Record Type="UI.CollectionFacet">"#,
                            r#"PropertyValue Property="Label" String="General"/>"#,
                            r#"PropertyValue Property="ID" String="GeneralSection"/>"#,
                            r#"PropertyValue Property="Facets">"#,
                                r#"Collection>"#,
                                    r#"Record Type="UI.ReferenceFacet">"#,
                                        r#"PropertyValue Property="Target" AnnotationPath="@UI.FieldGroup#Main"/>"#,
                                        r#"PropertyValue Property="Label" String="Partner Data"/>"#,
                                    r#"/Record>"#,
                                r#"/Collection>"#,
                            r#"/PropertyValue>"#,
                        r#"/Record>"#,
                    r#"/Collection>"#,
                r#"/Annotation>"#,
                r#"Annotation Term="UI.FieldGroup" Qualifier="Main">"#,
                    r#"Record Type="UI.FieldGroupType">"#,
                        r#"PropertyValue Property="Data">"#,
                            r#"Collection>"#,
                                r#"Record Type="UI.DataField">"#,
                                    r#"PropertyValue Property="Value" Path="Name"/>"#,
                                r#"/Record>"#,
                            r#"/Collection>"#,
                        r#"/PropertyValue>"#,
                    r#"/Record>"#,
                r#"/Annotation>"#,
            r#"/Annotations>"#,
        ];
        let xml_parts: Vec<&str> = xml.split('<').collect();
        assert_eq!(xml_parts.len(), expected_parts.len(), "different number of elements");
        for (i, (got, exp)) in xml_parts.iter().zip(expected_parts.iter()).enumerate() {
            assert_eq!(got, exp, "mismatch at element {i}: <{got} vs <{exp}");
        }
    }

    #[test]
    fn annotations_xml_full_definition() {
        let fields = vec![
            FieldDef {
                name: "OrderID",
                label: "Order Nr.",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
                immutable: true,
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
                form_group: Some("Main"),
            },
            FieldDef {
                name: "CustomerID",
                label: "Customer",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: Some("Customers"),
                value_source: None,
                prefer_dialog: false,
                text_path: None,
                searchable: false,
                show_in_list: true,
                list_sort_order: Some(1),
                list_importance: None,
                list_criticality_path: None,
                form_group: Some("Main"),
            },
            FieldDef {
                name: "TotalAmount",
                label: "Total",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(15),
                scale: Some(2),
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
                form_group: Some("Finance"),
            },
            FieldDef {
                name: "Status",
                label: "Status",
                edm_type: "Edm.String",
                max_length: Some(20),
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
        let def = AnnotationsDef {
            header_info: HeaderInfoDef {
                type_name: "Order",
                type_name_plural: "Orders",
                title_path: "OrderID",
                description_path: "CustomerID",
            },
            header_facets: &[HeaderFacetDef {
                data_point_qualifier: "Total",
                label: "Total Amount",
            }],
            data_points: &[DataPointDef {
                qualifier: "Total",
                value_path: "TotalAmount",
                title: "Total Amount",
                max_value: None,
                visualization: None,
            }],
            facet_sections: &[
                FacetSectionDef {
                    label: "General",
                    id: "GeneralSection",
                    field_group_qualifier: "Main",
                    field_group_label: "Order Data",
                },
                FacetSectionDef {
                    label: "Financial",
                    id: "FinancialSection",
                    field_group_qualifier: "Finance",
                    field_group_label: "Financial Data",
                },
            ],
            table_facets: &[TableFacetDef {
                label: "Items",
                id: "ItemsFacet",
                navigation_property: "Items",
            }],
        };
        let xml = build_annotations_xml("Order", &def, &fields);

        // Verify all major sections present
        assert!(xml.contains("Target=\"ProductsService.Order\""));
        assert!(xml.contains("UI.SelectionFields"));
        assert!(xml.contains("UI.LineItem"));
        assert!(xml.contains("UI.HeaderInfo"));
        assert!(xml.contains("UI.HeaderFacets"));
        assert!(xml.contains("UI.DataPoint"));
        assert!(xml.contains("UI.Facets"));
        assert!(xml.contains("UI.FieldGroup"));

        // Verify field group with semantic object uses intent-based navigation
        assert!(xml.contains("Qualifier=\"Main\""));
        // CustomerID in FieldGroup Main should be DataFieldWithIntentBasedNavigation
        let main_fg = &xml[xml.find("Qualifier=\"Main\"").unwrap()..];
        let main_fg_end = main_fg.find("</Annotation>").unwrap();
        let main_fg_section = &main_fg[..main_fg_end];
        assert!(main_fg_section.contains("UI.DataFieldWithIntentBasedNavigation"));
        assert!(main_fg_section.contains("SemanticObject\" String=\"Customers\""));

        // Property-level semantic object annotation
        assert!(xml.contains("Target=\"ProductsService.Order/CustomerID\""));
        assert!(xml.contains("Common.SemanticObject\" String=\"Customers\""));
    }

    #[test]
    fn capabilities_value_source_emits_value_list() {
        // value_source now stores the UUID of the FieldValueList directly
        let list_uuid = "ea102ff5-5777-5155-b0c3-8dd507435f93";
        let fields = vec![
            FieldDef {
                name: "EdmType",
                label: "Datentyp",
                edm_type: "Edm.String",
                max_length: Some(30),
                precision: None,
                scale: None,
                immutable: false,
                computed: false,
                references_entity: None,
                value_source: Some(list_uuid),
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
                name: "Name",
                label: "Name",
                edm_type: "Edm.String",
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
            form_group: None,
        },
        ];
        let xml = build_capabilities_annotations("Tests", "Test", "EdmType", None, &fields, true);

        // ValueList annotation on EdmType
        assert!(xml.contains("Common.ValueList"));
        assert!(xml.contains("CollectionPath\" String=\"FieldValueListItems\""));
        assert!(xml.contains("LocalDataProperty\" PropertyPath=\"EdmType\""));
        assert!(xml.contains("ValueListProperty\" String=\"Code\""));
        assert!(xml.contains("ValueListProperty\" String=\"Description\""));
        assert!(xml.contains("Common.ValueListWithFixedValues\" Bool=\"true\""));
        // ListID should appear as a ValueListParameterConstant with the UUID
        assert!(xml.contains("Common.ValueListParameterConstant"));
        assert!(xml.contains("ValueListProperty\" String=\"ListID\""));
        assert!(xml.contains(&format!("Constant\" String=\"{}\"", list_uuid)));

        // No ValueList on Name (value_source is None)
        let name_section_start = xml.find("Target=\"ProductsService.Test/Name\"").unwrap();
        let name_section = &xml[name_section_start..];
        let name_section_end = name_section.find("</Annotations>").unwrap();
        let name_section = &name_section[..name_section_end];
        assert!(!name_section.contains("Common.ValueList"));
    }

    #[test]
    fn references_entity_emits_value_list() {
        let fields = vec![FieldDef {
            name: "ValueSource",
            label: "Werteliste",
            edm_type: "Edm.String",
            max_length: Some(40),
            precision: None,
            scale: None,
            immutable: false,
            computed: false,
            references_entity: Some("FieldValueLists"),
            value_source: None,
            prefer_dialog: true,
            text_path: Some("ValueList/Description"),
            searchable: false,
            show_in_list: false,
            list_sort_order: None,
            list_importance: None,
            list_criticality_path: None,
            form_group: None,
        }];
        let xml =
            build_capabilities_annotations("Tests", "Test", "ValueSource", None, &fields, true);

        assert!(xml.contains("Common.ValueList"));
        assert!(xml.contains("CollectionPath\" String=\"FieldValueLists\""));
        assert!(xml.contains("LocalDataProperty\" PropertyPath=\"ValueSource\""));
        assert!(xml.contains("ValueListProperty\" String=\"ID\""));
        assert!(xml.contains("ValueListProperty\" String=\"Description\""));
        // prefer_dialog=true → no ValueListWithFixedValues annotation
        assert!(!xml.contains("Common.ValueListWithFixedValues"));
    }

    #[test]
    fn capabilities_common_text_on_key_field() {
        let xml = build_capabilities_annotations(
            "Products",
            "Product",
            "ProductID",
            Some("ProductName"),
            &simple_fields(),
            true,
        );
        // Key field should have Common.Text pointing to title_field
        let key_section_start = xml.find("Product/ProductID").unwrap();
        let key_section = &xml[key_section_start..];
        let key_section_end = key_section.find("</Annotations>").unwrap();
        let key_section = &key_section[..key_section_end];
        assert!(key_section.contains("Common.Text\" Path=\"ProductName\""));
        assert!(key_section
            .contains("UI.TextArrangement\" EnumMember=\"UI.TextArrangementType/TextOnly\""));

        // Non-key field should NOT have Common.Text
        let name_section_start = xml.find("Product/ProductName").unwrap();
        let name_section = &xml[name_section_start..];
        let name_section_end = name_section.find("</Annotations>").unwrap();
        let name_section = &name_section[..name_section_end];
        assert!(!name_section.contains("Common.Text"));
    }

    #[test]
    fn capabilities_no_common_text_when_key_equals_title() {
        // When key_field == title_field, no Common.Text should be emitted
        let fields = vec![FieldDef {
            name: "SetName",
            label: "EntitySet",
            edm_type: "Edm.String",
            max_length: Some(40),
            precision: None,
            scale: None,
            immutable: true,
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
    }];
        let xml = build_capabilities_annotations(
            "EntityConfigs",
            "EntityConfig",
            "SetName",
            Some("SetName"),
            &fields,
            true,
        );
        let key_section_start = xml.find("EntityConfig/SetName").unwrap();
        let key_section = &xml[key_section_start..];
        let key_section_end = key_section.find("</Annotations>").unwrap();
        let key_section = &key_section[..key_section_end];
        assert!(!key_section.contains("Common.Text"));
        assert!(!key_section.contains("UI.TextArrangement"));
    }

    #[test]
    fn capabilities_no_common_text_when_title_none() {
        let xml =
            build_capabilities_annotations("Tests", "Test", "ID", None, &simple_fields(), true);
        assert!(!xml.contains("Common.Text"));
        assert!(!xml.contains("UI.TextArrangement"));
    }
}
