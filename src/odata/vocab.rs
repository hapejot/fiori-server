//! Typed SAP UI Annotation Vocabulary.
//!
//! Provides Rust enums and structs that model the SAP UI, Common, Core, and
//! Capabilities annotation terms used in OData V4 EDMX metadata.  Each type
//! converts to the low-level [`Ann`]/[`Rec`] serialisation layer via
//! `to_ann()` / `to_rec()`.
//!
//! Scope: the ~20 terms actually consumed by Fiori Elements List Report +
//! Object Page.  New terms are added by creating a struct and implementing
//! the conversion trait.

use crate::odata::xml_types::*;

// ══════════════════════════════════════════════════════════════
//  SAP Enums
// ══════════════════════════════════════════════════════════════

/// `UI.ImportanceType` — column / field importance.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportanceType {
    High,
    Medium,
    Low,
}

impl ImportanceType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        };
        format!("UI.ImportanceType/{variant}")
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "High" => Some(Self::High),
            "Medium" => Some(Self::Medium),
            "Low" => Some(Self::Low),
            _ => None,
        }
    }
}

/// `UI.VisualizationType` — DataPoint visualisation style.
#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationType {
    Number,
    BulletChart,
    Progress,
    Rating,
    Donut,
    DeltaBulletChart,
}

impl VisualizationType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::Number => "Number",
            Self::BulletChart => "BulletChart",
            Self::Progress => "Progress",
            Self::Rating => "Rating",
            Self::Donut => "Donut",
            Self::DeltaBulletChart => "DeltaBulletChart",
        };
        format!("UI.VisualizationType/{variant}")
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Number" => Some(Self::Number),
            "BulletChart" => Some(Self::BulletChart),
            "Progress" => Some(Self::Progress),
            "Rating" => Some(Self::Rating),
            "Donut" => Some(Self::Donut),
            "DeltaBulletChart" => Some(Self::DeltaBulletChart),
            _ => None,
        }
    }
}

/// `UI.TextArrangementType` — how code + text are displayed together.
#[derive(Debug, Clone, PartialEq)]
pub enum TextArrangementType {
    TextFirst,
    TextLast,
    TextSeparate,
    TextOnly,
}

impl TextArrangementType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::TextFirst => "TextFirst",
            Self::TextLast => "TextLast",
            Self::TextSeparate => "TextSeparate",
            Self::TextOnly => "TextOnly",
        };
        format!("UI.TextArrangementType/{variant}")
    }
}

/// `UI.CriticalityType` — semantic severity / colouring.
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalityType {
    VeryNegative,
    Neutral,
    Negative,
    Critical,
    Positive,
    VeryPositive,
    Information,
}

impl CriticalityType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::VeryNegative => "VeryNegative",
            Self::Neutral => "Neutral",
            Self::Negative => "Negative",
            Self::Critical => "Critical",
            Self::Positive => "Positive",
            Self::VeryPositive => "VeryPositive",
            Self::Information => "Information",
        };
        format!("UI.CriticalityType/{variant}")
    }
}

/// How criticality is supplied: fixed enum value or dynamic path.
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalitySource {
    Fixed(CriticalityType),
    Path(String),
}

/// `Common.FieldControlType` — dynamic field state in forms.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldControlType {
    /// Field is not applicable / hidden (value 0).
    Inapplicable,
    /// Field is read-only (value 1).
    ReadOnly,
    /// Field is optional / editable (value 3).
    Optional,
    /// Field is mandatory (value 7).
    Mandatory,
}

impl FieldControlType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::Inapplicable => "Inapplicable",
            Self::ReadOnly => "ReadOnly",
            Self::Optional => "Optional",
            Self::Mandatory => "Mandatory",
        };
        format!("Common.FieldControlType/{variant}")
    }

    pub fn int_value(&self) -> u32 {
        match self {
            Self::Inapplicable => 0,
            Self::ReadOnly => 1,
            Self::Optional => 3,
            Self::Mandatory => 7,
        }
    }
}

/// How field control is supplied: fixed enum value or dynamic path.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldControlSource {
    Fixed(FieldControlType),
    Path(String),
}

/// `Common.TextFormatType` — whether text content is plain or HTML.
#[derive(Debug, Clone, PartialEq)]
pub enum TextFormatType {
    Plain,
    Html,
}

impl TextFormatType {
    pub fn enum_member(&self) -> String {
        let variant = match self {
            Self::Plain => "plain",
            Self::Html => "html",
        };
        format!("Common.TextFormatType/{variant}")
    }
}

/// Sort direction for `Common.SortOrder`.
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// `Common.NumericMessageSeverityType` — severity of structured messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageSeverity {
    Success,
    Information,
    Warning,
    Error,
}

impl MessageSeverity {
    pub fn value(&self) -> u32 {
        match self {
            Self::Success => 1,
            Self::Information => 2,
            Self::Warning => 3,
            Self::Error => 4,
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  DataField variants  (UI.DataFieldAbstract hierarchy)
// ══════════════════════════════════════════════════════════════

/// Semantic-object mapping pair: `LocalProperty` ↔ `SemanticObjectProperty`.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticObjectMapping {
    pub local_property: String,
    pub semantic_object_property: String,
}

/// Models the `DataFieldAbstract` type hierarchy (subset actually used).
#[derive(Debug, Clone, PartialEq)]
pub enum DataFieldVariant {
    /// `UI.DataField` — simple property display.
    DataField {
        value: String,
        criticality: Option<CriticalitySource>,
        importance: Option<ImportanceType>,
    },
    /// `UI.DataFieldWithIntentBasedNavigation` — property + semantic link.
    DataFieldWithIntentBasedNavigation {
        value: String,
        semantic_object: String,
        action: String,
        mapping: Vec<SemanticObjectMapping>,
        importance: Option<ImportanceType>,
    },
    /// `UI.DataFieldWithNavigationPath` — property + nav-property target.
    DataFieldWithNavigationPath {
        value: String,
        target: String,
        importance: Option<ImportanceType>,
    },
    /// `UI.DataFieldForAnnotation` — references another annotation (e.g. DataPoint).
    DataFieldForAnnotation {
        target: String,
        label: Option<String>,
    },
    /// `UI.DataFieldForAction` — standalone action button.
    DataFieldForAction {
        action: String,
        label: Option<String>,
        importance: Option<ImportanceType>,
    },
}

impl DataFieldVariant {
    /// Convert to a `Rec` with the correct `Record Type`.
    pub fn to_rec(&self) -> Rec {
        match self {
            Self::DataField {
                value,
                criticality,
                importance,
            } => {
                let mut props = vec![PV::Path("Value".into(), value.clone())];
                if let Some(crit) = criticality {
                    match crit {
                        CriticalitySource::Fixed(c) => {
                            props.push(PV::EnumMember(
                                "Criticality".into(),
                                c.enum_member(),
                            ));
                        }
                        CriticalitySource::Path(p) => {
                            props.push(PV::Path("Criticality".into(), p.clone()));
                        }
                    }
                }
                if let Some(imp) = importance {
                    props.push(PV::EnumMember(
                        "![@UI.Importance]".into(),
                        imp.enum_member(),
                    ));
                }
                Rec {
                    record_type: Some("UI.DataField".into()),
                    props,
                }
            }
            Self::DataFieldWithIntentBasedNavigation {
                value,
                semantic_object,
                action,
                mapping,
                importance,
            } => {
                let mut props = vec![
                    PV::Path("Value".into(), value.clone()),
                    PV::Str("SemanticObject".into(), semantic_object.clone()),
                    PV::Str("Action".into(), action.clone()),
                ];
                if !mapping.is_empty() {
                    let recs: Vec<Rec> = mapping
                        .iter()
                        .map(|m| Rec {
                            record_type: Some("Common.SemanticObjectMappingType".into()),
                            props: vec![
                                PV::PropPath(
                                    "LocalProperty".into(),
                                    m.local_property.clone(),
                                ),
                                PV::Str(
                                    "SemanticObjectProperty".into(),
                                    m.semantic_object_property.clone(),
                                ),
                            ],
                        })
                        .collect();
                    props.push(PV::Collection("Mapping".into(), recs));
                }
                if let Some(imp) = importance {
                    props.push(PV::EnumMember(
                        "![@UI.Importance]".into(),
                        imp.enum_member(),
                    ));
                }
                Rec {
                    record_type: Some("UI.DataFieldWithIntentBasedNavigation".into()),
                    props,
                }
            }
            Self::DataFieldWithNavigationPath {
                value,
                target,
                importance,
            } => {
                let mut props = vec![
                    PV::Path("Value".into(), value.clone()),
                    PV::Str("Target".into(), target.clone()),
                ];
                if let Some(imp) = importance {
                    props.push(PV::EnumMember(
                        "![@UI.Importance]".into(),
                        imp.enum_member(),
                    ));
                }
                Rec {
                    record_type: Some("UI.DataFieldWithNavigationPath".into()),
                    props,
                }
            }
            Self::DataFieldForAnnotation { target, label } => {
                let mut props = vec![PV::AnnotationPath("Target".into(), target.clone())];
                if let Some(l) = label {
                    props.push(PV::Str("Label".into(), l.clone()));
                }
                Rec {
                    record_type: Some("UI.DataFieldForAnnotation".into()),
                    props,
                }
            }
            Self::DataFieldForAction {
                action,
                label,
                importance,
            } => {
                let mut props = vec![PV::Str("Action".into(), action.clone())];
                if let Some(l) = label {
                    props.push(PV::Str("Label".into(), l.clone()));
                }
                if let Some(imp) = importance {
                    props.push(PV::EnumMember(
                        "![@UI.Importance]".into(),
                        imp.enum_member(),
                    ));
                }
                Rec {
                    record_type: Some("UI.DataFieldForAction".into()),
                    props,
                }
            }
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Facet variants
// ══════════════════════════════════════════════════════════════

/// Models the `Facet` abstract type hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub enum FacetVariant {
    /// `UI.CollectionFacet` — container grouping child facets.
    CollectionFacet {
        id: String,
        label: String,
        facets: Vec<FacetVariant>,
    },
    /// `UI.ReferenceFacet` — points to an annotation (FieldGroup, LineItem, etc.).
    ReferenceFacet {
        id: String,
        label: String,
        target: String,
    },
}

impl FacetVariant {
    pub fn to_rec(&self) -> Rec {
        match self {
            Self::CollectionFacet { id, label, facets } => {
                let children: Vec<Rec> = facets.iter().map(|f| f.to_rec()).collect();
                Rec {
                    record_type: Some("UI.CollectionFacet".into()),
                    props: vec![
                        PV::Str("Label".into(), label.clone()),
                        PV::Str("ID".into(), id.clone()),
                        PV::Collection("Facets".into(), children),
                    ],
                }
            }
            Self::ReferenceFacet { id, label, target } => Rec {
                record_type: Some("UI.ReferenceFacet".into()),
                props: vec![
                    PV::AnnotationPath("Target".into(), target.clone()),
                    PV::Str("Label".into(), label.clone()),
                    PV::Str("ID".into(), id.clone()),
                ],
            },
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Annotation term structs
// ══════════════════════════════════════════════════════════════

/// Trait converting a typed vocabulary struct into a low-level `Ann`.
pub trait IntoAnnotation {
    fn to_ann(&self) -> Ann;
}

// ── UI.SelectionFields ──

pub struct SelectionFields(pub Vec<String>);

impl IntoAnnotation for SelectionFields {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.SelectionFields".into(),
            qualifier: None,
            content: AnnContent::PropertyPaths(self.0.clone()),
        }
    }
}

// ── UI.LineItem ──

pub struct LineItem {
    pub fields: Vec<DataFieldVariant>,
}

impl IntoAnnotation for LineItem {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.LineItem".into(),
            qualifier: None,
            content: AnnContent::Collection(
                self.fields.iter().map(|f| f.to_rec()).collect(),
            ),
        }
    }
}

// ── UI.HeaderInfo ──

pub struct HeaderInfo {
    pub type_name: String,
    pub type_name_plural: String,
    pub title: DataFieldVariant,
    pub description: Option<DataFieldVariant>,
}

impl IntoAnnotation for HeaderInfo {
    fn to_ann(&self) -> Ann {
        let mut props = vec![
            PV::Str("TypeName".into(), self.type_name.clone()),
            PV::Str("TypeNamePlural".into(), self.type_name_plural.clone()),
            PV::Record("Title".into(), self.title.to_rec()),
        ];
        if let Some(desc) = &self.description {
            props.push(PV::Record("Description".into(), desc.to_rec()));
        }
        Ann {
            term: "UI.HeaderInfo".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("UI.HeaderInfoType".into()),
                props,
            }),
        }
    }
}

// ── UI.DataPoint ──

pub struct DataPoint {
    pub qualifier: String,
    pub value: String,
    pub title: String,
    pub max_value: Option<u32>,
    pub visualization: Option<VisualizationType>,
    pub criticality: Option<CriticalitySource>,
}

impl IntoAnnotation for DataPoint {
    fn to_ann(&self) -> Ann {
        let mut props = vec![
            PV::Path("Value".into(), self.value.clone()),
            PV::Str("Title".into(), self.title.clone()),
        ];
        if let Some(max) = self.max_value {
            props.push(PV::Int("MaximumValue".into(), max));
        }
        if let Some(vis) = &self.visualization {
            props.push(PV::EnumMember("Visualization".into(), vis.enum_member()));
        }
        if let Some(crit) = &self.criticality {
            match crit {
                CriticalitySource::Fixed(c) => {
                    props.push(PV::EnumMember("Criticality".into(), c.enum_member()));
                }
                CriticalitySource::Path(p) => {
                    props.push(PV::Path("Criticality".into(), p.clone()));
                }
            }
        }
        Ann {
            term: "UI.DataPoint".into(),
            qualifier: Some(self.qualifier.clone()),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.DataPointType".into()),
                props,
            }),
        }
    }
}

// ── UI.FieldGroup ──

pub struct FieldGroup {
    pub qualifier: String,
    pub data: Vec<DataFieldVariant>,
}

impl IntoAnnotation for FieldGroup {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.FieldGroup".into(),
            qualifier: Some(self.qualifier.clone()),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.FieldGroupType".into()),
                props: vec![PV::Collection(
                    "Data".into(),
                    self.data.iter().map(|f| f.to_rec()).collect(),
                )],
            }),
        }
    }
}

// ── UI.HeaderFacets ──

pub struct HeaderFacets(pub Vec<FacetVariant>);

impl IntoAnnotation for HeaderFacets {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.HeaderFacets".into(),
            qualifier: None,
            content: AnnContent::Collection(self.0.iter().map(|f| f.to_rec()).collect()),
        }
    }
}

// ── UI.Facets ──

pub struct Facets(pub Vec<FacetVariant>);

impl IntoAnnotation for Facets {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.Facets".into(),
            qualifier: None,
            content: AnnContent::Collection(self.0.iter().map(|f| f.to_rec()).collect()),
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Property-level annotation structs
// ══════════════════════════════════════════════════════════════

// ── Common.Text + UI.TextArrangement ──

pub struct TextAnnotation {
    pub path: String,
    pub arrangement: TextArrangementType,
}

impl IntoAnnotation for TextAnnotation {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.Text".into(),
            qualifier: None,
            content: AnnContent::PathWithChildren(
                self.path.clone(),
                vec![Ann {
                    term: "UI.TextArrangement".into(),
                    qualifier: None,
                    content: AnnContent::EnumMember(self.arrangement.enum_member()),
                }],
            ),
        }
    }
}

// ── Measure annotations ──

/// `Org.OData.Measures.V1.ISOCurrency` or `Unit`.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureAnnotation {
    ISOCurrency(String),
    Unit(String),
}

impl IntoAnnotation for MeasureAnnotation {
    fn to_ann(&self) -> Ann {
        let (term, path) = match self {
            Self::ISOCurrency(p) => ("Org.OData.Measures.V1.ISOCurrency", p),
            Self::Unit(p) => ("Org.OData.Measures.V1.Unit", p),
        };
        Ann {
            term: term.into(),
            qualifier: None,
            content: AnnContent::PathWithChildren(path.clone(), vec![]),
        }
    }
}

// ── Common.SemanticObject + Mapping ──

pub struct SemanticObjectAnnotation {
    pub semantic_object: String,
    pub mapping: Vec<SemanticObjectMapping>,
}

impl SemanticObjectAnnotation {
    /// Produces two `Ann` items: Common.SemanticObject + Common.SemanticObjectMapping.
    pub fn to_anns(&self) -> Vec<Ann> {
        let mut out = vec![Ann {
            term: "Common.SemanticObject".into(),
            qualifier: None,
            content: AnnContent::Str(self.semantic_object.clone()),
        }];
        if !self.mapping.is_empty() {
            out.push(Ann {
                term: "Common.SemanticObjectMapping".into(),
                qualifier: None,
                content: AnnContent::Collection(
                    self.mapping
                        .iter()
                        .map(|m| Rec {
                            record_type: None,
                            props: vec![
                                PV::PropPath(
                                    "LocalProperty".into(),
                                    m.local_property.clone(),
                                ),
                                PV::Str(
                                    "SemanticObjectProperty".into(),
                                    m.semantic_object_property.clone(),
                                ),
                            ],
                        })
                        .collect(),
                ),
            });
        }
        out
    }
}

// ── Common.ValueList ──

/// `Common.ValueListParameterIn` filter.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueListFilter {
    pub local_property: String,
    pub target_property: String,
}

/// Typed model for the Common.ValueList annotation and its variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueListAnnotation {
    /// Code-list style: FieldValueListItems filtered by ListID.
    CodeList {
        list_id: String,
        fixed_values: bool,
    },
    /// Entity-reference style: direct lookup into another entity set.
    EntityRef {
        collection_path: String,
        key_property: String,
        display_property: Option<String>,
        filters: Vec<ValueListFilter>,
        fixed_values: bool,
    },
}

impl ValueListAnnotation {
    /// Produces `Common.ValueList` (+ optional `Common.ValueListWithFixedValues`).
    pub fn to_anns(&self, local_property: &str) -> Vec<Ann> {
        match self {
            Self::CodeList {
                list_id,
                fixed_values,
            } => {
                let params = vec![
                    Rec {
                        record_type: Some("Common.ValueListParameterOut".into()),
                        props: vec![
                            PV::PropPath(
                                "LocalDataProperty".into(),
                                local_property.into(),
                            ),
                            PV::Str("ValueListProperty".into(), "Code".into()),
                        ],
                    },
                    Rec {
                        record_type: Some("Common.ValueListParameterDisplayOnly".into()),
                        props: vec![PV::Str(
                            "ValueListProperty".into(),
                            "Description".into(),
                        )],
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
                            PV::Str(
                                "CollectionPath".into(),
                                "FieldValueListItems".into(),
                            ),
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
            Self::EntityRef {
                collection_path,
                key_property,
                display_property,
                filters,
                fixed_values,
            } => {
                let mut params = vec![Rec {
                    record_type: Some("Common.ValueListParameterOut".into()),
                    props: vec![
                        PV::PropPath(
                            "LocalDataProperty".into(),
                            local_property.into(),
                        ),
                        PV::Str("ValueListProperty".into(), key_property.clone()),
                    ],
                }];
                if let Some(dp) = display_property {
                    params.push(Rec {
                        record_type: Some(
                            "Common.ValueListParameterDisplayOnly".into(),
                        ),
                        props: vec![PV::Str(
                            "ValueListProperty".into(),
                            dp.clone(),
                        )],
                    });
                }
                for f in filters {
                    params.push(Rec {
                        record_type: Some("Common.ValueListParameterIn".into()),
                        props: vec![
                            PV::PropPath(
                                "LocalDataProperty".into(),
                                f.local_property.clone(),
                            ),
                            PV::Str(
                                "ValueListProperty".into(),
                                f.target_property.clone(),
                            ),
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
}

// ══════════════════════════════════════════════════════════════
//  Capability / Core annotation structs
// ══════════════════════════════════════════════════════════════

// ── Simple scalar annotations (Common.Label, UI.Hidden, Core.*) ──

/// Scalar annotation that wraps a term + content.
/// Used for Common.Label, UI.Hidden, Core.Computed, Core.Immutable.
pub struct ScalarAnnotation {
    pub term: &'static str,
    pub content: AnnContent,
}

impl IntoAnnotation for ScalarAnnotation {
    fn to_ann(&self) -> Ann {
        Ann {
            term: self.term.into(),
            qualifier: None,
            content: self.content.clone(),
        }
    }
}

/// Convenience constructors for common scalar annotations.
impl ScalarAnnotation {
    pub fn label(text: &str) -> Self {
        Self {
            term: "Common.Label",
            content: AnnContent::Str(text.into()),
        }
    }
    pub fn hidden() -> Self {
        Self {
            term: "UI.Hidden",
            content: AnnContent::Bool(true),
        }
    }
    pub fn computed() -> Self {
        Self {
            term: "Org.OData.Core.V1.Computed",
            content: AnnContent::Bool(true),
        }
    }
    pub fn immutable() -> Self {
        Self {
            term: "Org.OData.Core.V1.Immutable",
            content: AnnContent::Bool(true),
        }
    }
}

// ── Draft annotations ──

pub struct DraftRoot {
    pub activation_action: String,
    pub edit_action: String,
    pub preparation_action: String,
}

impl IntoAnnotation for DraftRoot {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.DraftRoot".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("Common.DraftRootType".into()),
                props: vec![
                    PV::Str("ActivationAction".into(), self.activation_action.clone()),
                    PV::Str("EditAction".into(), self.edit_action.clone()),
                    PV::Str(
                        "PreparationAction".into(),
                        self.preparation_action.clone(),
                    ),
                ],
            }),
        }
    }
}

pub struct DraftNode {
    pub preparation_action: String,
}

impl IntoAnnotation for DraftNode {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.DraftNode".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("Common.DraftNodeType".into()),
                props: vec![PV::Str(
                    "PreparationAction".into(),
                    self.preparation_action.clone(),
                )],
            }),
        }
    }
}

// ── Capability restrictions ──

pub struct UpdateRestrictions {
    pub updatable: bool,
}

impl IntoAnnotation for UpdateRestrictions {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Org.OData.Capabilities.V1.UpdateRestrictions".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: None,
                props: vec![PV::Bool("Updatable".into(), self.updatable)],
            }),
        }
    }
}

pub struct InsertRestrictions {
    pub non_insertable_properties: Vec<String>,
}

impl IntoAnnotation for InsertRestrictions {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Org.OData.Capabilities.V1.InsertRestrictions".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: Some("Capabilities.InsertRestrictionsType".into()),
                props: vec![PV::PropertyPaths(
                    "NonInsertableProperties".into(),
                    self.non_insertable_properties.clone(),
                )],
            }),
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Common vocabulary — additional annotations
// ══════════════════════════════════════════════════════════════

// ── Common.FieldControl ──

/// Dynamic field state: mandatory, optional, read-only, or hidden.
///
/// Can be applied with a fixed value or a path to a property that supplies the
/// value at runtime (enabling row-level field control).
pub struct FieldControlAnnotation(pub FieldControlSource);

impl IntoAnnotation for FieldControlAnnotation {
    fn to_ann(&self) -> Ann {
        match &self.0 {
            FieldControlSource::Fixed(fc) => Ann {
                term: "Common.FieldControl".into(),
                qualifier: None,
                content: AnnContent::EnumMember(fc.enum_member()),
            },
            FieldControlSource::Path(p) => Ann {
                term: "Common.FieldControl".into(),
                qualifier: None,
                content: AnnContent::PathWithChildren(p.clone(), vec![]),
            },
        }
    }
}

// ── Common.SemanticKey ──

/// Properties forming the human-readable key (modulo `IsActiveEntity` for drafts).
pub struct SemanticKey(pub Vec<String>);

impl IntoAnnotation for SemanticKey {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.SemanticKey".into(),
            qualifier: None,
            content: AnnContent::PropertyPaths(self.0.clone()),
        }
    }
}

// ── Common.SortOrder ──

/// One entry in a sort order collection.
#[derive(Debug, Clone, PartialEq)]
pub struct SortOrderEntry {
    pub property: String,
    pub descending: bool,
}

/// Default sort order for an entity set.
pub struct SortOrder(pub Vec<SortOrderEntry>);

impl IntoAnnotation for SortOrder {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.SortOrder".into(),
            qualifier: None,
            content: AnnContent::Collection(
                self.0
                    .iter()
                    .map(|e| {
                        let mut props = vec![PV::PropPath("Property".into(), e.property.clone())];
                        if e.descending {
                            props.push(PV::Bool("Descending".into(), true));
                        }
                        Rec {
                            record_type: None,
                            props,
                        }
                    })
                    .collect(),
            ),
        }
    }
}

// ── Common.FilterDefaultValue ──

/// Default value pre-populated in the filter bar.
pub struct FilterDefaultValue(pub String);

impl IntoAnnotation for FilterDefaultValue {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.FilterDefaultValue".into(),
            qualifier: None,
            content: AnnContent::Str(self.0.clone()),
        }
    }
}

// ── Common.IsActionCritical ──

/// Marks an action as requiring a confirmation dialog.
pub struct IsActionCritical;

impl IntoAnnotation for IsActionCritical {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.IsActionCritical".into(),
            qualifier: None,
            content: AnnContent::Bool(true),
        }
    }
}

// ── Common.TextFormat ──

/// Declares whether a property contains plain text or HTML.
pub struct TextFormat(pub TextFormatType);

impl IntoAnnotation for TextFormat {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.TextFormat".into(),
            qualifier: None,
            content: AnnContent::EnumMember(self.0.enum_member()),
        }
    }
}

// ── Common.Heading / Common.QuickInfo ──

impl ScalarAnnotation {
    /// Column heading text (shorter than Label).
    pub fn heading(text: &str) -> Self {
        Self {
            term: "Common.Heading",
            content: AnnContent::Str(text.into()),
        }
    }
    /// Tooltip / quick info text.
    pub fn quick_info(text: &str) -> Self {
        Self {
            term: "Common.QuickInfo",
            content: AnnContent::Str(text.into()),
        }
    }
}

// ── Common.Interval ──

/// Marks two properties as forming a range (e.g. DateFrom/DateTo).
pub struct Interval {
    pub lower_boundary: String,
    pub upper_boundary: String,
}

impl IntoAnnotation for Interval {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.Interval".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: None,
                props: vec![
                    PV::PropPath("LowerBoundary".into(), self.lower_boundary.clone()),
                    PV::PropPath("UpperBoundary".into(), self.upper_boundary.clone()),
                ],
            }),
        }
    }
}

// ── Common.SideEffects ──

/// Declares dependencies between properties: when sources change, targets are
/// refreshed. Optionally triggers a server action.
pub struct SideEffects {
    pub qualifier: Option<String>,
    pub source_properties: Vec<String>,
    pub source_entities: Vec<String>,
    pub target_properties: Vec<String>,
    pub target_entities: Vec<String>,
    pub trigger_action: Option<String>,
}

impl IntoAnnotation for SideEffects {
    fn to_ann(&self) -> Ann {
        let mut props = Vec::new();
        if !self.source_properties.is_empty() {
            props.push(PV::PropertyPaths(
                "SourceProperties".into(),
                self.source_properties.clone(),
            ));
        }
        if !self.source_entities.is_empty() {
            // SourceEntities uses NavigationPropertyPaths but serialises the same
            props.push(PV::PropertyPaths(
                "SourceEntities".into(),
                self.source_entities.clone(),
            ));
        }
        if !self.target_properties.is_empty() {
            props.push(PV::PropertyPaths(
                "TargetProperties".into(),
                self.target_properties.clone(),
            ));
        }
        if !self.target_entities.is_empty() {
            props.push(PV::PropertyPaths(
                "TargetEntities".into(),
                self.target_entities.clone(),
            ));
        }
        if let Some(action) = &self.trigger_action {
            props.push(PV::Str("TriggerAction".into(), action.clone()));
        }
        Ann {
            term: "Common.SideEffects".into(),
            qualifier: self.qualifier.clone(),
            content: AnnContent::Record(Rec {
                record_type: Some("Common.SideEffectsType".into()),
                props,
            }),
        }
    }
}

// ── Common.DefaultValuesFunction ──

/// Function import called by Fiori to compute default values for new entities.
pub struct DefaultValuesFunction(pub String);

impl IntoAnnotation for DefaultValuesFunction {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Common.DefaultValuesFunction".into(),
            qualifier: None,
            content: AnnContent::Str(self.0.clone()),
        }
    }
}

// ── Property tags: CreatedAt, CreatedBy, ChangedAt, ChangedBy, IsCurrency, IsUnit ──

impl ScalarAnnotation {
    pub fn created_at() -> Self {
        Self {
            term: "Common.CreatedAt",
            content: AnnContent::Bool(true),
        }
    }
    pub fn created_by() -> Self {
        Self {
            term: "Common.CreatedBy",
            content: AnnContent::Bool(true),
        }
    }
    pub fn changed_at() -> Self {
        Self {
            term: "Common.ChangedAt",
            content: AnnContent::Bool(true),
        }
    }
    pub fn changed_by() -> Self {
        Self {
            term: "Common.ChangedBy",
            content: AnnContent::Bool(true),
        }
    }
    pub fn is_currency() -> Self {
        Self {
            term: "Common.IsCurrency",
            content: AnnContent::Bool(true),
        }
    }
    pub fn is_unit() -> Self {
        Self {
            term: "Common.IsUnit",
            content: AnnContent::Bool(true),
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  UI vocabulary — additional annotations
// ══════════════════════════════════════════════════════════════

// ── UI.Identification ──

/// Object identifier fields — used in navigation targets and object markers.
pub struct Identification {
    pub fields: Vec<DataFieldVariant>,
}

impl IntoAnnotation for Identification {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.Identification".into(),
            qualifier: None,
            content: AnnContent::Collection(
                self.fields.iter().map(|f| f.to_rec()).collect(),
            ),
        }
    }
}

// ── UI.CreateHidden / UI.UpdateHidden / UI.DeleteHidden ──

/// Whether create/update/delete is conditionally hidden.
/// Can be a fixed bool or a dynamic path.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationVisibility {
    Fixed(bool),
    Path(String),
}

impl OperationVisibility {
    fn to_ann_with_term(&self, term: &str) -> Ann {
        Ann {
            term: term.into(),
            qualifier: None,
            content: match self {
                Self::Fixed(v) => AnnContent::Bool(*v),
                Self::Path(p) => AnnContent::PathWithChildren(p.clone(), vec![]),
            },
        }
    }
}

pub struct CreateHidden(pub OperationVisibility);
pub struct UpdateHidden(pub OperationVisibility);
pub struct DeleteHidden(pub OperationVisibility);

impl IntoAnnotation for CreateHidden {
    fn to_ann(&self) -> Ann {
        self.0.to_ann_with_term("UI.CreateHidden")
    }
}
impl IntoAnnotation for UpdateHidden {
    fn to_ann(&self) -> Ann {
        self.0.to_ann_with_term("UI.UpdateHidden")
    }
}
impl IntoAnnotation for DeleteHidden {
    fn to_ann(&self) -> Ann {
        self.0.to_ann_with_term("UI.DeleteHidden")
    }
}

// ── UI.MultiLineText ──

impl ScalarAnnotation {
    /// Marks a property as multi-line text (renders as textarea).
    pub fn multi_line_text() -> Self {
        Self {
            term: "UI.MultiLineText",
            content: AnnContent::Bool(true),
        }
    }
    /// Marks a property as containing an image URL.
    pub fn is_image_url() -> Self {
        Self {
            term: "UI.IsImageURL",
            content: AnnContent::Bool(true),
        }
    }
}

// ── UI.PresentationVariant ──

/// Default result shaping: sort order, grouping, visualisation references.
pub struct PresentationVariant {
    pub qualifier: Option<String>,
    pub sort_order: Vec<SortOrderEntry>,
    pub group_by: Vec<String>,
    pub total_by: Vec<String>,
    pub max_items: Option<u32>,
    pub visualizations: Vec<String>,
    pub request_at_least: Vec<String>,
}

impl IntoAnnotation for PresentationVariant {
    fn to_ann(&self) -> Ann {
        let mut props = Vec::new();
        if !self.sort_order.is_empty() {
            props.push(PV::Collection(
                "SortOrder".into(),
                self.sort_order
                    .iter()
                    .map(|e| {
                        let mut p = vec![PV::PropPath("Property".into(), e.property.clone())];
                        if e.descending {
                            p.push(PV::Bool("Descending".into(), true));
                        }
                        Rec {
                            record_type: None,
                            props: p,
                        }
                    })
                    .collect(),
            ));
        }
        if !self.group_by.is_empty() {
            props.push(PV::PropertyPaths("GroupBy".into(), self.group_by.clone()));
        }
        if !self.total_by.is_empty() {
            props.push(PV::PropertyPaths("TotalBy".into(), self.total_by.clone()));
        }
        if let Some(max) = self.max_items {
            props.push(PV::Int("MaxItems".into(), max));
        }
        if !self.visualizations.is_empty() {
            // Visualizations is a collection of AnnotationPaths
            props.push(PV::PropertyPaths(
                "Visualizations".into(),
                self.visualizations.clone(),
            ));
        }
        if !self.request_at_least.is_empty() {
            props.push(PV::PropertyPaths(
                "RequestAtLeast".into(),
                self.request_at_least.clone(),
            ));
        }
        Ann {
            term: "UI.PresentationVariant".into(),
            qualifier: self.qualifier.clone(),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.PresentationVariantType".into()),
                props,
            }),
        }
    }
}

// ── UI.SelectionVariant ──

/// A select option: property + value ranges.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub property: String,
    pub ranges: Vec<SelectionRange>,
}

/// A single range within a SelectOption.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRange {
    pub sign: SelectionRangeSign,
    pub option: SelectionRangeOption,
    pub low: String,
    pub high: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionRangeSign {
    Include,
    Exclude,
}

impl SelectionRangeSign {
    pub fn enum_member(&self) -> String {
        match self {
            Self::Include => "UI.SelectionRangeSignType/I".into(),
            Self::Exclude => "UI.SelectionRangeSignType/E".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionRangeOption {
    EQ,
    BT,
    CP,
    LE,
    GE,
    NE,
    NB,
    NP,
    GT,
    LT,
}

impl SelectionRangeOption {
    pub fn enum_member(&self) -> String {
        let v = match self {
            Self::EQ => "EQ",
            Self::BT => "BT",
            Self::CP => "CP",
            Self::LE => "LE",
            Self::GE => "GE",
            Self::NE => "NE",
            Self::NB => "NB",
            Self::NP => "NP",
            Self::GT => "GT",
            Self::LT => "LT",
        };
        format!("UI.SelectionRangeOptionType/{v}")
    }
}

/// Pre-defined query filters and parameters.
pub struct SelectionVariant {
    pub qualifier: Option<String>,
    pub text: Option<String>,
    pub select_options: Vec<SelectOption>,
}

impl IntoAnnotation for SelectionVariant {
    fn to_ann(&self) -> Ann {
        let mut props = Vec::new();
        if let Some(text) = &self.text {
            props.push(PV::Str("Text".into(), text.clone()));
        }
        if !self.select_options.is_empty() {
            let options: Vec<Rec> = self
                .select_options
                .iter()
                .map(|so| {
                    let ranges: Vec<Rec> = so
                        .ranges
                        .iter()
                        .map(|r| {
                            let mut rp = vec![
                                PV::EnumMember("Sign".into(), r.sign.enum_member()),
                                PV::EnumMember("Option".into(), r.option.enum_member()),
                                PV::Str("Low".into(), r.low.clone()),
                            ];
                            if let Some(high) = &r.high {
                                rp.push(PV::Str("High".into(), high.clone()));
                            }
                            Rec {
                                record_type: Some("UI.SelectionRangeType".into()),
                                props: rp,
                            }
                        })
                        .collect();
                    Rec {
                        record_type: Some("UI.SelectOptionType".into()),
                        props: vec![
                            PV::PropPath("PropertyName".into(), so.property.clone()),
                            PV::Collection("Ranges".into(), ranges),
                        ],
                    }
                })
                .collect();
            props.push(PV::Collection("SelectOptions".into(), options));
        }
        Ann {
            term: "UI.SelectionVariant".into(),
            qualifier: self.qualifier.clone(),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.SelectionVariantType".into()),
                props,
            }),
        }
    }
}

// ── UI.SelectionPresentationVariant ──

/// Combined selection + presentation variant reference.
pub struct SelectionPresentationVariant {
    pub qualifier: Option<String>,
    pub selection_variant_path: String,
    pub presentation_variant_path: String,
}

impl IntoAnnotation for SelectionPresentationVariant {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "UI.SelectionPresentationVariant".into(),
            qualifier: self.qualifier.clone(),
            content: AnnContent::Record(Rec {
                record_type: Some("UI.SelectionPresentationVariantType".into()),
                props: vec![
                    PV::AnnotationPath(
                        "SelectionVariant".into(),
                        self.selection_variant_path.clone(),
                    ),
                    PV::AnnotationPath(
                        "PresentationVariant".into(),
                        self.presentation_variant_path.clone(),
                    ),
                ],
            }),
        }
    }
}

// ── Capability: DeleteRestrictions ──

pub struct DeleteRestrictions {
    pub deletable: bool,
}

impl IntoAnnotation for DeleteRestrictions {
    fn to_ann(&self) -> Ann {
        Ann {
            term: "Org.OData.Capabilities.V1.DeleteRestrictions".into(),
            qualifier: None,
            content: AnnContent::Record(Rec {
                record_type: None,
                props: vec![PV::Bool("Deletable".into(), self.deletable)],
            }),
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Escape hatch
// ══════════════════════════════════════════════════════════════

/// Wraps a manually-constructed `Ann` for terms not yet modelled.
pub struct RawAnnotation(pub Ann);

impl IntoAnnotation for RawAnnotation {
    fn to_ann(&self) -> Ann {
        // Clone the inner Ann — we own PV/Rec/Ann types crate-internally.
        Ann {
            term: self.0.term.clone(),
            qualifier: self.0.qualifier.clone(),
            content: self.0.content.clone(),
        }
    }
}

// ══════════════════════════════════════════════════════════════
//  Tests
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odata::xml_types::anns_to_xml;

    #[test]
    fn test_importance_enum_member() {
        assert_eq!(ImportanceType::High.enum_member(), "UI.ImportanceType/High");
        assert_eq!(
            ImportanceType::Medium.enum_member(),
            "UI.ImportanceType/Medium"
        );
        assert_eq!(ImportanceType::Low.enum_member(), "UI.ImportanceType/Low");
    }

    #[test]
    fn test_visualization_enum_member() {
        assert_eq!(
            VisualizationType::Progress.enum_member(),
            "UI.VisualizationType/Progress"
        );
        assert_eq!(
            VisualizationType::Rating.enum_member(),
            "UI.VisualizationType/Rating"
        );
    }

    #[test]
    fn test_text_arrangement_enum_member() {
        assert_eq!(
            TextArrangementType::TextOnly.enum_member(),
            "UI.TextArrangementType/TextOnly"
        );
    }

    #[test]
    fn test_criticality_enum_member() {
        assert_eq!(
            CriticalityType::Positive.enum_member(),
            "UI.CriticalityType/Positive"
        );
    }

    #[test]
    fn test_datafield_to_rec() {
        let df = DataFieldVariant::DataField {
            value: "OrderName".into(),
            criticality: None,
            importance: Some(ImportanceType::High),
        };
        let rec = df.to_rec();
        assert_eq!(rec.record_type.as_deref(), Some("UI.DataField"));
        assert!(rec.props.len() >= 2);
    }

    #[test]
    fn test_datafield_ibn_to_rec() {
        let df = DataFieldVariant::DataFieldWithIntentBasedNavigation {
            value: "CustomerID".into(),
            semantic_object: "Customers".into(),
            action: "display".into(),
            mapping: vec![SemanticObjectMapping {
                local_property: "CustomerID".into(),
                semantic_object_property: "ID".into(),
            }],
            importance: None,
        };
        let rec = df.to_rec();
        assert_eq!(
            rec.record_type.as_deref(),
            Some("UI.DataFieldWithIntentBasedNavigation")
        );
    }

    #[test]
    fn test_selection_fields() {
        let sf = SelectionFields(vec!["Name".into(), "Status".into()]);
        let ann = sf.to_ann();
        assert_eq!(ann.term, "UI.SelectionFields");
        match ann.content {
            AnnContent::PropertyPaths(paths) => {
                assert_eq!(paths, vec!["Name", "Status"]);
            }
            _ => panic!("Expected PropertyPaths"),
        }
    }

    #[test]
    fn test_header_info() {
        let hi = HeaderInfo {
            type_name: "Order".into(),
            type_name_plural: "Orders".into(),
            title: DataFieldVariant::DataField {
                value: "OrderName".into(),
                criticality: None,
                importance: None,
            },
            description: Some(DataFieldVariant::DataField {
                value: "Status".into(),
                criticality: None,
                importance: None,
            }),
        };
        let ann = hi.to_ann();
        assert_eq!(ann.term, "UI.HeaderInfo");
    }

    #[test]
    fn test_data_point() {
        let dp = DataPoint {
            qualifier: "Price".into(),
            value: "Price".into(),
            title: "Price".into(),
            max_value: Some(1000),
            visualization: Some(VisualizationType::Progress),
            criticality: None,
        };
        let ann = dp.to_ann();
        assert_eq!(ann.term, "UI.DataPoint");
        assert_eq!(ann.qualifier.as_deref(), Some("Price"));
    }

    #[test]
    fn test_line_item() {
        let li = LineItem {
            fields: vec![
                DataFieldVariant::DataField {
                    value: "Name".into(),
                    criticality: None,
                    importance: Some(ImportanceType::High),
                },
                DataFieldVariant::DataFieldWithIntentBasedNavigation {
                    value: "CustomerID".into(),
                    semantic_object: "Customers".into(),
                    action: "display".into(),
                    mapping: vec![],
                    importance: None,
                },
            ],
        };
        let ann = li.to_ann();
        assert_eq!(ann.term, "UI.LineItem");
        match ann.content {
            AnnContent::Collection(recs) => assert_eq!(recs.len(), 2),
            _ => panic!("Expected Collection"),
        }
    }

    #[test]
    fn test_field_group() {
        let fg = FieldGroup {
            qualifier: "General".into(),
            data: vec![DataFieldVariant::DataField {
                value: "Name".into(),
                criticality: None,
                importance: None,
            }],
        };
        let ann = fg.to_ann();
        assert_eq!(ann.term, "UI.FieldGroup");
        assert_eq!(ann.qualifier.as_deref(), Some("General"));
    }

    #[test]
    fn test_facets() {
        let f = Facets(vec![
            FacetVariant::CollectionFacet {
                id: "General".into(),
                label: "General".into(),
                facets: vec![FacetVariant::ReferenceFacet {
                    id: "GeneralRef".into(),
                    label: "General".into(),
                    target: "@UI.FieldGroup#General".into(),
                }],
            },
            FacetVariant::ReferenceFacet {
                id: "ItemsTable".into(),
                label: "Items".into(),
                target: "Items/@UI.LineItem".into(),
            },
        ]);
        let ann = f.to_ann();
        assert_eq!(ann.term, "UI.Facets");
        match ann.content {
            AnnContent::Collection(recs) => assert_eq!(recs.len(), 2),
            _ => panic!("Expected Collection"),
        }
    }

    #[test]
    fn test_text_annotation() {
        let t = TextAnnotation {
            path: "_Status_text".into(),
            arrangement: TextArrangementType::TextOnly,
        };
        let ann = t.to_ann();
        assert_eq!(ann.term, "Common.Text");
        match &ann.content {
            AnnContent::PathWithChildren(p, children) => {
                assert_eq!(p, "_Status_text");
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].term, "UI.TextArrangement");
            }
            _ => panic!("Expected PathWithChildren"),
        }
    }

    #[test]
    fn test_measure_annotation() {
        let m = MeasureAnnotation::ISOCurrency("Currency".into());
        let ann = m.to_ann();
        assert_eq!(ann.term, "Org.OData.Measures.V1.ISOCurrency");
    }

    #[test]
    fn test_value_list_code_list() {
        let vl = ValueListAnnotation::CodeList {
            list_id: "abc-123".into(),
            fixed_values: true,
        };
        let anns = vl.to_anns("Status");
        assert_eq!(anns.len(), 2);
        assert_eq!(anns[0].term, "Common.ValueList");
        assert_eq!(anns[1].term, "Common.ValueListWithFixedValues");
    }

    #[test]
    fn test_value_list_entity_ref() {
        let vl = ValueListAnnotation::EntityRef {
            collection_path: "Customers".into(),
            key_property: "ID".into(),
            display_property: Some("CustomerName".into()),
            filters: vec![ValueListFilter {
                local_property: "ID".into(),
                target_property: "ConfigID".into(),
            }],
            fixed_values: false,
        };
        let anns = vl.to_anns("CustomerID");
        assert_eq!(anns.len(), 1);
        let xml = anns_to_xml(&[Anns {
            target: "test".into(),
            annotations: anns,
        }]);
        assert!(xml.contains("Common.ValueListParameterIn"));
        assert!(xml.contains("ConfigID"));
    }

    #[test]
    fn test_semantic_object_annotation() {
        let so = SemanticObjectAnnotation {
            semantic_object: "Customers".into(),
            mapping: vec![SemanticObjectMapping {
                local_property: "CustomerID".into(),
                semantic_object_property: "ID".into(),
            }],
        };
        let anns = so.to_anns();
        assert_eq!(anns.len(), 2);
        assert_eq!(anns[0].term, "Common.SemanticObject");
        assert_eq!(anns[1].term, "Common.SemanticObjectMapping");
    }

    #[test]
    fn test_draft_root() {
        let dr = DraftRoot {
            activation_action: "Service.draftActivate".into(),
            edit_action: "Service.draftEdit".into(),
            preparation_action: "Service.draftPrepare".into(),
        };
        let ann = dr.to_ann();
        assert_eq!(ann.term, "Common.DraftRoot");
    }

    #[test]
    fn test_draft_node() {
        let dn = DraftNode {
            preparation_action: "Service.draftPrepare".into(),
        };
        let ann = dn.to_ann();
        assert_eq!(ann.term, "Common.DraftNode");
    }

    #[test]
    fn test_insert_restrictions() {
        let ir = InsertRestrictions {
            non_insertable_properties: vec!["ID".into(), "CreatedAt".into()],
        };
        let ann = ir.to_ann();
        assert_eq!(ann.term, "Org.OData.Capabilities.V1.InsertRestrictions");
    }

    #[test]
    fn test_update_restrictions() {
        let ur = UpdateRestrictions { updatable: true };
        let ann = ur.to_ann();
        assert_eq!(ann.term, "Org.OData.Capabilities.V1.UpdateRestrictions");
    }

    #[test]
    fn test_scalar_label() {
        let ann = ScalarAnnotation::label("Order Name").to_ann();
        assert_eq!(ann.term, "Common.Label");
        match ann.content {
            AnnContent::Str(s) => assert_eq!(s, "Order Name"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_scalar_hidden() {
        let ann = ScalarAnnotation::hidden().to_ann();
        assert_eq!(ann.term, "UI.Hidden");
    }

    #[test]
    fn test_raw_annotation_escape_hatch() {
        let raw = RawAnnotation(Ann {
            term: "com.sap.vocabularies.Custom.v1.Something".into(),
            qualifier: None,
            content: AnnContent::Bool(true),
        });
        let ann = raw.to_ann();
        assert_eq!(ann.term, "com.sap.vocabularies.Custom.v1.Something");
    }

    // ── Common vocabulary tests ──

    #[test]
    fn test_field_control_type_enum() {
        assert_eq!(
            FieldControlType::Mandatory.enum_member(),
            "Common.FieldControlType/Mandatory"
        );
        assert_eq!(FieldControlType::Mandatory.int_value(), 7);
        assert_eq!(FieldControlType::ReadOnly.int_value(), 1);
        assert_eq!(FieldControlType::Inapplicable.int_value(), 0);
    }

    #[test]
    fn test_field_control_fixed() {
        let ann = FieldControlAnnotation(FieldControlSource::Fixed(
            FieldControlType::Mandatory,
        ))
        .to_ann();
        assert_eq!(ann.term, "Common.FieldControl");
        match ann.content {
            AnnContent::EnumMember(v) => {
                assert_eq!(v, "Common.FieldControlType/Mandatory");
            }
            _ => panic!("Expected EnumMember"),
        }
    }

    #[test]
    fn test_field_control_path() {
        let ann =
            FieldControlAnnotation(FieldControlSource::Path("FieldControl".into())).to_ann();
        assert_eq!(ann.term, "Common.FieldControl");
        match ann.content {
            AnnContent::PathWithChildren(p, _) => assert_eq!(p, "FieldControl"),
            _ => panic!("Expected PathWithChildren"),
        }
    }

    #[test]
    fn test_text_format_type() {
        assert_eq!(
            TextFormatType::Html.enum_member(),
            "Common.TextFormatType/html"
        );
        assert_eq!(
            TextFormatType::Plain.enum_member(),
            "Common.TextFormatType/plain"
        );
    }

    #[test]
    fn test_text_format() {
        let ann = TextFormat(TextFormatType::Html).to_ann();
        assert_eq!(ann.term, "Common.TextFormat");
    }

    #[test]
    fn test_semantic_key() {
        let ann = SemanticKey(vec!["OrderNumber".into(), "Year".into()]).to_ann();
        assert_eq!(ann.term, "Common.SemanticKey");
        match ann.content {
            AnnContent::PropertyPaths(paths) => {
                assert_eq!(paths, vec!["OrderNumber", "Year"]);
            }
            _ => panic!("Expected PropertyPaths"),
        }
    }

    #[test]
    fn test_sort_order() {
        let ann = SortOrder(vec![
            SortOrderEntry {
                property: "CreatedAt".into(),
                descending: true,
            },
            SortOrderEntry {
                property: "Name".into(),
                descending: false,
            },
        ])
        .to_ann();
        assert_eq!(ann.term, "Common.SortOrder");
        match ann.content {
            AnnContent::Collection(recs) => assert_eq!(recs.len(), 2),
            _ => panic!("Expected Collection"),
        }
    }

    #[test]
    fn test_filter_default_value() {
        let ann = FilterDefaultValue("Open".into()).to_ann();
        assert_eq!(ann.term, "Common.FilterDefaultValue");
        match ann.content {
            AnnContent::Str(s) => assert_eq!(s, "Open"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_is_action_critical() {
        let ann = IsActionCritical.to_ann();
        assert_eq!(ann.term, "Common.IsActionCritical");
    }

    #[test]
    fn test_interval() {
        let ann = Interval {
            lower_boundary: "StartDate".into(),
            upper_boundary: "EndDate".into(),
        }
        .to_ann();
        assert_eq!(ann.term, "Common.Interval");
    }

    #[test]
    fn test_side_effects() {
        let ann = SideEffects {
            qualifier: Some("PriceChanged".into()),
            source_properties: vec!["Price".into(), "Quantity".into()],
            source_entities: vec![],
            target_properties: vec!["NetAmount".into()],
            target_entities: vec![],
            trigger_action: Some("Service.recalculate".into()),
        }
        .to_ann();
        assert_eq!(ann.term, "Common.SideEffects");
        assert_eq!(ann.qualifier.as_deref(), Some("PriceChanged"));
    }

    #[test]
    fn test_default_values_function() {
        let ann = DefaultValuesFunction("Service.getDefaults".into()).to_ann();
        assert_eq!(ann.term, "Common.DefaultValuesFunction");
    }

    #[test]
    fn test_scalar_audit_tags() {
        assert_eq!(ScalarAnnotation::created_at().to_ann().term, "Common.CreatedAt");
        assert_eq!(ScalarAnnotation::created_by().to_ann().term, "Common.CreatedBy");
        assert_eq!(ScalarAnnotation::changed_at().to_ann().term, "Common.ChangedAt");
        assert_eq!(ScalarAnnotation::changed_by().to_ann().term, "Common.ChangedBy");
    }

    #[test]
    fn test_scalar_currency_unit_tags() {
        assert_eq!(ScalarAnnotation::is_currency().to_ann().term, "Common.IsCurrency");
        assert_eq!(ScalarAnnotation::is_unit().to_ann().term, "Common.IsUnit");
    }

    #[test]
    fn test_heading_and_quickinfo() {
        assert_eq!(ScalarAnnotation::heading("Order").to_ann().term, "Common.Heading");
        assert_eq!(
            ScalarAnnotation::quick_info("tooltip").to_ann().term,
            "Common.QuickInfo"
        );
    }

    // ── UI vocabulary tests ──

    #[test]
    fn test_identification() {
        let ann = Identification {
            fields: vec![DataFieldVariant::DataField {
                value: "OrderNumber".into(),
                criticality: None,
                importance: None,
            }],
        }
        .to_ann();
        assert_eq!(ann.term, "UI.Identification");
    }

    #[test]
    fn test_create_hidden_fixed() {
        let ann = CreateHidden(OperationVisibility::Fixed(true)).to_ann();
        assert_eq!(ann.term, "UI.CreateHidden");
        match ann.content {
            AnnContent::Bool(v) => assert!(v),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_delete_hidden_path() {
        let ann = DeleteHidden(OperationVisibility::Path("IsDeletable".into())).to_ann();
        assert_eq!(ann.term, "UI.DeleteHidden");
    }

    #[test]
    fn test_multi_line_text() {
        assert_eq!(
            ScalarAnnotation::multi_line_text().to_ann().term,
            "UI.MultiLineText"
        );
    }

    #[test]
    fn test_is_image_url() {
        assert_eq!(
            ScalarAnnotation::is_image_url().to_ann().term,
            "UI.IsImageURL"
        );
    }

    #[test]
    fn test_presentation_variant() {
        let ann = PresentationVariant {
            qualifier: Some("Default".into()),
            sort_order: vec![SortOrderEntry {
                property: "CreatedAt".into(),
                descending: true,
            }],
            group_by: vec![],
            total_by: vec![],
            max_items: Some(20),
            visualizations: vec!["@UI.LineItem".into()],
            request_at_least: vec![],
        }
        .to_ann();
        assert_eq!(ann.term, "UI.PresentationVariant");
        assert_eq!(ann.qualifier.as_deref(), Some("Default"));
    }

    #[test]
    fn test_selection_variant() {
        let ann = SelectionVariant {
            qualifier: Some("Open".into()),
            text: Some("Open Orders".into()),
            select_options: vec![SelectOption {
                property: "Status".into(),
                ranges: vec![SelectionRange {
                    sign: SelectionRangeSign::Include,
                    option: SelectionRangeOption::EQ,
                    low: "O".into(),
                    high: None,
                }],
            }],
        }
        .to_ann();
        assert_eq!(ann.term, "UI.SelectionVariant");
        assert_eq!(ann.qualifier.as_deref(), Some("Open"));
        let xml = anns_to_xml(&[Anns {
            target: "test".into(),
            annotations: vec![ann],
        }]);
        assert!(xml.contains("SelectionRangeSignType/I"));
        assert!(xml.contains("SelectionRangeOptionType/EQ"));
    }

    #[test]
    fn test_selection_presentation_variant() {
        let ann = SelectionPresentationVariant {
            qualifier: Some("Default".into()),
            selection_variant_path: "@UI.SelectionVariant#Open".into(),
            presentation_variant_path: "@UI.PresentationVariant#Default".into(),
        }
        .to_ann();
        assert_eq!(ann.term, "UI.SelectionPresentationVariant");
    }

    #[test]
    fn test_delete_restrictions() {
        let ann = DeleteRestrictions { deletable: false }.to_ann();
        assert_eq!(ann.term, "Org.OData.Capabilities.V1.DeleteRestrictions");
    }
}
