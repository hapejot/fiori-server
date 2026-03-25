use crate::NAMESPACE;

/// Einheitliche Feld-Definition fuer EntityType-Properties UND Annotations.
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub edm_type: &'static str,
    pub max_length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
}

/// LineItem-Referenz mit Annotation-spezifischen Attributen.
pub struct LineItemField {
    pub name: &'static str,
    pub importance: Option<&'static str>,
    pub criticality_path: Option<&'static str>,
    /// Navigation-Property-Pfad – erzeugt UI.DataFieldWithNavigationPath.
    pub navigation_path: Option<&'static str>,
}

/// NavigationProperty-Definition im EntityType.
pub struct NavigationPropertyDef {
    pub name: &'static str,
    pub target_type: &'static str,
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

/// Eine Gruppe von Feldern (z.B. "General", "Pricing").
pub struct FieldGroupDef {
    pub qualifier: &'static str,
    pub fields: &'static [&'static str],
}

/// Ein CollectionFacet auf der Object Page, verweist auf eine FieldGroup.
pub struct FacetSectionDef {
    pub label: &'static str,
    pub id: &'static str,
    pub field_group_qualifier: &'static str,
    pub field_group_label: &'static str,
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
    pub selection_fields: &'static [&'static str],
    pub line_item: &'static [LineItemField],
    pub header_info: HeaderInfoDef,
    pub header_facets: &'static [HeaderFacetDef],
    pub data_points: &'static [DataPointDef],
    pub facet_sections: &'static [FacetSectionDef],
    pub field_groups: &'static [FieldGroupDef],
}

/// Erzeugt das Annotations-XML fuer eine Entitaet aus ihrer AnnotationsDef.
pub fn build_annotations_xml(
    entity_type_name: &str,
    def: &AnnotationsDef,
    fields: &[FieldDef],
) -> String {
    let mut x = format!(
        "<Annotations Target=\"{}.{}\">",
        NAMESPACE, entity_type_name
    );

    // ── SelectionFields ──
    x.push_str("<Annotation Term=\"UI.SelectionFields\">");
    x.push_str("<Collection>");
    for f in def.selection_fields {
        x.push_str(&format!("<PropertyPath>{}</PropertyPath>", f));
    }
    x.push_str("</Collection>");
    x.push_str("</Annotation>");

    // ── LineItem ──
    x.push_str("<Annotation Term=\"UI.LineItem\">");
    x.push_str("<Collection>");
    for f in def.line_item {
        let label = fields
            .iter()
            .find(|fd| fd.name == f.name)
            .map(|fd| fd.label)
            .unwrap_or(f.name);
        let record_type = if f.navigation_path.is_some() {
            "UI.DataFieldWithNavigationPath"
        } else {
            "UI.DataField"
        };
        x.push_str(&format!("<Record Type=\"{}\">", record_type));
        x.push_str(&format!(
            "<PropertyValue Property=\"Value\" Path=\"{}\"/>",
            f.name
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"Label\" String=\"{}\"/>",
            label
        ));
        if let Some(nav) = f.navigation_path {
            x.push_str(&format!(
                "<PropertyValue Property=\"Target\" NavigationPropertyPath=\"{}\"/>",
                nav
            ));
        }
        if let Some(imp) = f.importance {
            x.push_str(&format!(
                "<PropertyValue Property=\"![@UI.Importance]\" EnumMember=\"UI.ImportanceType/{}\"/>",
                imp
            ));
        }
        if let Some(crit) = f.criticality_path {
            x.push_str(&format!(
                "<PropertyValue Property=\"Criticality\" Path=\"{}\"/>",
                crit
            ));
        }
        x.push_str("</Record>");
    }
    x.push_str("</Collection>");
    x.push_str("</Annotation>");

    // ── HeaderInfo ──
    x.push_str("<Annotation Term=\"UI.HeaderInfo\">");
    x.push_str("<Record Type=\"UI.HeaderInfoType\">");
    x.push_str(&format!(
        "<PropertyValue Property=\"TypeName\" String=\"{}\"/>",
        def.header_info.type_name
    ));
    x.push_str(&format!(
        "<PropertyValue Property=\"TypeNamePlural\" String=\"{}\"/>",
        def.header_info.type_name_plural
    ));
    x.push_str("<PropertyValue Property=\"Title\">");
    x.push_str("<Record Type=\"UI.DataField\">");
    x.push_str(&format!(
        "  <PropertyValue Property=\"Value\" Path=\"{}\"/>",
        def.header_info.title_path
    ));
    x.push_str("</Record>");
    x.push_str("</PropertyValue>");
    x.push_str("<PropertyValue Property=\"Description\">");
    x.push_str("<Record Type=\"UI.DataField\">");
    x.push_str(&format!(
        "  <PropertyValue Property=\"Value\" Path=\"{}\"/>",
        def.header_info.description_path
    ));
    x.push_str("</Record>");
    x.push_str("</PropertyValue>");
    x.push_str("</Record>");
    x.push_str("</Annotation>");

    // ── HeaderFacets ──
    x.push_str("<Annotation Term=\"UI.HeaderFacets\">");
    x.push_str("<Collection>");
    for hf in def.header_facets {
        x.push_str("<Record Type=\"UI.ReferenceFacet\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"Target\" AnnotationPath=\"@UI.DataPoint#{}\"/>",
            hf.data_point_qualifier
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"Label\" String=\"{}\"/>",
            hf.label
        ));
        x.push_str("</Record>");
    }
    x.push_str("</Collection>");
    x.push_str("</Annotation>");

    // ── DataPoints ──
    for dp in def.data_points {
        x.push_str(&format!(
            "<Annotation Term=\"UI.DataPoint\" Qualifier=\"{}\">",
            dp.qualifier
        ));
        x.push_str("<Record Type=\"UI.DataPointType\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"Value\" Path=\"{}\"/>",
            dp.value_path
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"Title\" String=\"{}\"/>",
            dp.title
        ));
        if let Some(max) = dp.max_value {
            x.push_str(&format!(
                "<PropertyValue Property=\"MaximumValue\" Int=\"{}\"/>",
                max
            ));
        }
        if let Some(vis) = dp.visualization {
            x.push_str(&format!(
                "<PropertyValue Property=\"Visualization\" EnumMember=\"UI.VisualizationType/{}\"/>",
                vis
            ));
        }
        x.push_str("</Record>");
        x.push_str("</Annotation>");
    }

    // ── Facets (Object Page Sections) ──
    x.push_str("<Annotation Term=\"UI.Facets\">");
    x.push_str("<Collection>");
    for sec in def.facet_sections {
        x.push_str("<Record Type=\"UI.CollectionFacet\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"Label\" String=\"{}\"/>",
            sec.label
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"ID\" String=\"{}\"/>",
            sec.id
        ));
        x.push_str("<PropertyValue Property=\"Facets\">");
        x.push_str("<Collection>");
        x.push_str("<Record Type=\"UI.ReferenceFacet\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"Target\" AnnotationPath=\"@UI.FieldGroup#{}\"/>",
            sec.field_group_qualifier
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"Label\" String=\"{}\"/>",
            sec.field_group_label
        ));
        x.push_str("</Record>");
        x.push_str("</Collection>");
        x.push_str("</PropertyValue>");
        x.push_str("</Record>");
    }
    x.push_str("</Collection>");
    x.push_str("</Annotation>");

    // ── FieldGroups ──
    for fg in def.field_groups {
        x.push_str(&format!(
            "<Annotation Term=\"UI.FieldGroup\" Qualifier=\"{}\">",
            fg.qualifier
        ));
        x.push_str("<Record Type=\"UI.FieldGroupType\">");
        x.push_str("<PropertyValue Property=\"Data\">");
        x.push_str("<Collection>");
        for name in fg.fields {
            let label = fields
                .iter()
                .find(|fd| fd.name == *name)
                .map(|fd| fd.label)
                .unwrap_or(name);
            x.push_str("<Record Type=\"UI.DataField\">");
            x.push_str(&format!(
                "<PropertyValue Property=\"Value\" Path=\"{}\"/>",
                name
            ));
            x.push_str(&format!(
                "<PropertyValue Property=\"Label\" String=\"{}\"/>",
                label
            ));
            x.push_str("</Record>");
        }
        x.push_str("</Collection>");
        x.push_str("</PropertyValue>");
        x.push_str("</Record>");
        x.push_str("</Annotation>");
    }

    x.push_str("</Annotations>");
    x
}

/// Erzeugt das EntityType-XML aus Typ-Name, Schluesselfeld und Property-Definitionen.
pub fn build_entity_type_xml(type_name: &str, key_field: &str, props: &[FieldDef]) -> String {
    let mut x = format!("<EntityType Name=\"{}\">", type_name);
    x.push_str("<Key>");
    x.push_str(&format!(
        "<PropertyRef Name=\"{}\"/>",
        key_field
    ));
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
        x.push_str(&format!(
            "<Property Name=\"{}\"{}{}/>",
            p.name, pad, attr
        ));
    }
    x.push_str("</EntityType>");
    x
}

/// Haengt NavigationProperty-Elemente an einen EntityType-XML-String an.
pub fn append_navigation_properties(xml: &mut String, nav_props: &[NavigationPropertyDef]) {
    for np in nav_props {
        xml.insert_str(
            xml.rfind("</EntityType>").unwrap(),
            &format!(
                "<NavigationProperty Name=\"{}\" Type=\"{}.{}\"/>",
                np.name, NAMESPACE, np.target_type
            ),
        );
    }
}
