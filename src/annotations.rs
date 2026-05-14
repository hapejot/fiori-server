
/// Unified field definition for EntityType properties AND annotations.
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub edm_type: &'static str,
    pub max_length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    /// Field can be set at creation time, then becomes read-only (Core.Immutable).
    pub immutable: bool,
    /// Field is server-computed/generated (Core.Computed) – never visible in forms.
    pub computed: bool,
    /// FK reference to another EntitySet (e.g. "Customers").
    /// Automatically generates: 1:1 NavigationProperty, Common.Text, ValueList, Intent-Based Navigation.
    pub references_entity: Option<&'static str>,
    /// Name of a value list (UUID of the FieldValueList) – generates Common.ValueList
    /// with CollectionPath="FieldValueListItems", Out=Code, Display=Description.
    pub value_source: Option<&'static str>,
    /// true → prefer search dialog, false → dropdown (applies to value_source and references_entity).
    pub prefer_dialog: bool,
    /// Path for Common.Text on FK references (e.g. "_ValueList/ListName").
    /// Generates Common.Text + UI.TextArrangement/TextOnly on this field.
    pub text_path: Option<&'static str>,
    // ── Annotation control (derived in build_annotations) ──
    /// Field appears as a search filter (UI.SelectionFields).
    pub searchable: bool,
    /// Field appears as a list column (UI.LineItem).
    pub show_in_list: bool,
    /// Sort order in the list (ascending).
    pub list_sort_order: Option<u32>,
    /// Importance in the list ("High", "Medium", "Low").
    pub list_importance: Option<&'static str>,
    /// Path for the criticality indicator in the list.
    pub list_criticality_path: Option<&'static str>,
    /// FieldGroup qualifier – assigns the field to a form group (e.g. "Basic", "Tile").
    pub form_group: Option<&'static str>,
}

/// Flexible ValueList configuration for custom value helps.
pub struct ValueListDef {
    /// OData EntitySet (e.g. "FieldValueLists", "EntityConfigs")
    pub collection_path: &'static str,
    /// Field in the target EntitySet returned as an out-parameter
    pub key_property: &'static str,
    /// Optional display-only field
    pub display_property: Option<&'static str>,
    /// true → Dropdown (Common.ValueListWithFixedValues), false → Dialog
    pub fixed_values: bool,
}

/// NavigationProperty definition in the EntityType.
#[derive(Debug)]
pub struct NavigationPropertyDef {
    pub name: &'static str,
    pub target_type: &'static str,
    /// true for 1:n compositions (generates Collection type)
    pub is_collection: bool,
    /// Foreign key field on the child (for 1:n) or on this entity (for 1:1).
    /// If None, the parent's key field name is used.
    pub foreign_key: Option<&'static str>,
}

/// DataPoint for the Object Page header.
#[derive(Debug, Clone)]
pub struct DataPointDef {
    pub qualifier: &'static str,
    pub value_path: &'static str,
    pub title: &'static str,
    pub max_value: Option<u32>,
    pub visualization: Option<&'static str>,
}

/// ReferenceFacet in the HeaderFacets block – points to a DataPoint.
#[derive(Debug, Clone)]
pub struct HeaderFacetDef {
    pub data_point_qualifier: &'static str,
    pub label: &'static str,
}

/// A CollectionFacet on the Object Page, pointing to a FieldGroup.
pub struct FacetSectionDef {
    pub label: &'static str,
    pub id: &'static str,
    pub field_group_qualifier: &'static str,
    pub field_group_label: &'static str,
}

/// Table facet: points to the UI.LineItem annotation of a composition (NavProperty).
pub struct TableFacetDef {
    pub label: &'static str,
    pub id: &'static str,
    /// Name of the NavigationProperty (e.g. "Items")
    pub navigation_property: &'static str,
}

/// Object Page header.
pub struct HeaderInfoDef {
    pub type_name: &'static str,
    pub type_name_plural: &'static str,
    pub title_path: &'static str,
    pub description_path: &'static str,
}

/// Complete annotation definition for an entity.
pub struct AnnotationsDef {
    pub header_info: HeaderInfoDef,
    pub header_facets: &'static [HeaderFacetDef],
    pub data_points: &'static [DataPointDef],
    pub facet_sections: &'static [FacetSectionDef],
    /// Table facets for compositions (e.g. OrderItems).
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