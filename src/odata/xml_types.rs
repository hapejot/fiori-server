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

pub fn anns_to_xml(blocks: &[Anns]) -> String {
    let mut x = String::new();
    for b in blocks {
        b.to_xml(&mut x);
    }
    x
}
