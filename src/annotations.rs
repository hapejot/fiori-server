use crate::NAMESPACE;

/// Einheitliche Feld-Definition fuer EntityType-Properties UND Annotations.
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub edm_type: &'static str,
    pub max_length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    /// Feld ist nicht editierbar (Key, berechnete Werte etc.)
    pub immutable: bool,
    /// Semantic Object fuer Intent-Based Navigation (z.B. "Products").
    pub semantic_object: Option<&'static str>,
}

/// LineItem-Referenz mit Annotation-spezifischen Attributen.
pub struct LineItemField {
    pub name: &'static str,
    /// Optionales Label-Override (falls name ein Pfad wie "Product/ProductName" ist).
    pub label: Option<&'static str>,
    pub importance: Option<&'static str>,
    pub criticality_path: Option<&'static str>,
    /// Navigation-Property-Pfad – erzeugt UI.DataFieldWithNavigationPath.
    pub navigation_path: Option<&'static str>,
    /// Semantic Object – erzeugt UI.DataFieldWithIntentBasedNavigation.
    pub semantic_object: Option<&'static str>,
}

/// NavigationProperty-Definition im EntityType.
pub struct NavigationPropertyDef {
    pub name: &'static str,
    pub target_type: &'static str,
    /// true fuer 1:n Kompositionen (erzeugt Collection-Typ)
    pub is_collection: bool,
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
    pub selection_fields: &'static [&'static str],
    pub line_item: &'static [LineItemField],
    pub header_info: HeaderInfoDef,
    pub header_facets: &'static [HeaderFacetDef],
    pub data_points: &'static [DataPointDef],
    pub facet_sections: &'static [FacetSectionDef],
    pub field_groups: &'static [FieldGroupDef],
    /// Tabellen-Facets fuer Kompositionen (z.B. OrderItems).
    pub table_facets: &'static [TableFacetDef],
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
        let label = f.label.unwrap_or_else(|| {
            fields
                .iter()
                .find(|fd| fd.name == f.name)
                .map(|fd| fd.label)
                .unwrap_or(f.name)
        });
        let record_type = if f.semantic_object.is_some() {
            "UI.DataFieldWithIntentBasedNavigation"
        } else if f.navigation_path.is_some() {
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
        if let Some(so) = f.semantic_object {
            x.push_str(&format!(
                "<PropertyValue Property=\"SemanticObject\" String=\"{}\"/>",
                so
            ));
            x.push_str("<PropertyValue Property=\"Action\" String=\"display\"/>");
        }
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
    // ── Table Facets (Composition tables) ──
    for tf in def.table_facets {
        x.push_str("<Record Type=\"UI.ReferenceFacet\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"Label\" String=\"{}\"/>",
            tf.label
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"ID\" String=\"{}\"/>",
            tf.id
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"Target\" AnnotationPath=\"{}/@UI.LineItem\"/>",
            tf.navigation_property
        ));
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
            let field_def = fields.iter().find(|fd| fd.name == *name);
            let label = field_def.map(|fd| fd.label).unwrap_or(name);
            let semantic_obj = field_def.and_then(|fd| fd.semantic_object);
            if let Some(so) = semantic_obj {
                x.push_str("<Record Type=\"UI.DataFieldWithIntentBasedNavigation\">");
                x.push_str(&format!(
                    "<PropertyValue Property=\"Value\" Path=\"{}\"/>",
                    name
                ));
                x.push_str(&format!(
                    "<PropertyValue Property=\"Label\" String=\"{}\"/>",
                    label
                ));
                x.push_str(&format!(
                    "<PropertyValue Property=\"SemanticObject\" String=\"{}\"/>",
                    so
                ));
                x.push_str("<PropertyValue Property=\"Action\" String=\"display\"/>");
            } else {
                x.push_str("<Record Type=\"UI.DataField\">");
                x.push_str(&format!(
                    "<PropertyValue Property=\"Value\" Path=\"{}\"/>",
                    name
                ));
                x.push_str(&format!(
                    "<PropertyValue Property=\"Label\" String=\"{}\"/>",
                    label
                ));
            }
            x.push_str("</Record>");
        }
        x.push_str("</Collection>");
        x.push_str("</PropertyValue>");
        x.push_str("</Record>");
        x.push_str("</Annotation>");
    }

    x.push_str("</Annotations>");

    // ── Property-level Common.SemanticObject annotations ──
    for f in fields {
        if let Some(so) = f.semantic_object {
            x.push_str(&format!(
                "<Annotations Target=\"{ns}.{et}/{prop}\">",
                ns = NAMESPACE,
                et = entity_type_name,
                prop = f.name
            ));
            x.push_str(&format!(
                "<Annotation Term=\"Common.SemanticObject\" String=\"{}\"/>",
                so
            ));
            x.push_str("</Annotations>");
        }
    }

    x
}

/// Erzeugt Capabilities-Annotations fuer ein EntitySet (UpdateRestrictions, DraftRoot/DraftNode).
pub fn build_capabilities_annotations(
    entity_set_name: &str,
    entity_type_name: &str,
    fields: &[FieldDef],
    is_draft_root: bool,
) -> String {
    let mut x = String::new();

    // UpdateRestrictions + DraftRoot/DraftNode auf dem EntitySet
    x.push_str(&format!(
        "<Annotations Target=\"{ns}.EntityContainer/{set}\">",
        ns = NAMESPACE,
        set = entity_set_name
    ));
    x.push_str("<Annotation Term=\"Org.OData.Capabilities.V1.UpdateRestrictions\">");
    x.push_str("<Record>");
    x.push_str("<PropertyValue Property=\"Updatable\" Bool=\"true\"/>");
    x.push_str("</Record>");
    x.push_str("</Annotation>");
    if is_draft_root {
        // DraftRoot – aktiviert den Edit-Button in Fiori Elements V4
        x.push_str("<Annotation Term=\"Common.DraftRoot\">");
        x.push_str("<Record Type=\"Common.DraftRootType\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"ActivationAction\" String=\"{ns}.draftActivate\"/>",
            ns = NAMESPACE
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"EditAction\" String=\"{ns}.draftEdit\"/>",
            ns = NAMESPACE
        ));
        x.push_str(&format!(
            "<PropertyValue Property=\"PreparationAction\" String=\"{ns}.draftPrepare\"/>",
            ns = NAMESPACE
        ));
        x.push_str("</Record>");
        x.push_str("</Annotation>");
    } else {
        // DraftNode – Kompositions-Kind-Entity
        x.push_str("<Annotation Term=\"Common.DraftNode\">");
        x.push_str("<Record Type=\"Common.DraftNodeType\">");
        x.push_str(&format!(
            "<PropertyValue Property=\"PreparationAction\" String=\"{ns}.draftPrepare\"/>",
            ns = NAMESPACE
        ));
        x.push_str("</Record>");
        x.push_str("</Annotation>");
    }
    x.push_str("</Annotations>");

    // Per-Property-Annotations: Common.Label + ggf. Immutable
    for f in fields {
        x.push_str(&format!(
            "<Annotations Target=\"{ns}.{ty}/{prop}\">",
            ns = NAMESPACE,
            ty = entity_type_name,
            prop = f.name
        ));
        x.push_str(&format!(
            "<Annotation Term=\"Common.Label\" String=\"{}\"/>",
            f.label
        ));
        if f.immutable {
            x.push_str("<Annotation Term=\"Org.OData.Core.V1.Immutable\" Bool=\"true\"/>");
        }
        x.push_str("</Annotations>");
    }

    x
}

/// Erzeugt das EntityType-XML aus Typ-Name, Schluesselfeld und Property-Definitionen.
/// Fuegt automatisch Draft-Properties (IsActiveEntity, HasActiveEntity, HasDraftEntity)
/// sowie Draft-NavigationProperties (SiblingEntity, DraftAdministrativeData) hinzu.
pub fn build_entity_type_xml(type_name: &str, key_field: &str, props: &[FieldDef]) -> String {
    let mut x = format!("<EntityType Name=\"{}\">", type_name);
    x.push_str("<Key>");
    x.push_str(&format!(
        "<PropertyRef Name=\"{}\"/>",
        key_field
    ));
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
        x.push_str(&format!(
            "<Property Name=\"{}\"{}{}/>",
            p.name, pad, attr
        ));
    }
    // Draft-Properties
    x.push_str("<Property Name=\"IsActiveEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"true\"/>");
    x.push_str("<Property Name=\"HasActiveEntity\"  Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");
    x.push_str("<Property Name=\"HasDraftEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");
    // Draft-NavigationProperties
    x.push_str(&format!(
        "<NavigationProperty Name=\"SiblingEntity\" Type=\"{ns}.{ty}\"/>",
        ns = NAMESPACE, ty = type_name
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
    x.push_str("<Property Name=\"CreationDateTime\"       Type=\"Edm.DateTimeOffset\" Precision=\"7\"/>");
    x.push_str("<Property Name=\"CreatedByUser\"          Type=\"Edm.String\" MaxLength=\"256\"/>");
    x.push_str("<Property Name=\"DraftIsCreatedByMe\"     Type=\"Edm.Boolean\"/>");
    x.push_str("<Property Name=\"LastChangeDateTime\"     Type=\"Edm.DateTimeOffset\" Precision=\"7\"/>");
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
                semantic_object: None,
            },
            FieldDef {
                name: "ProductName",
                label: "Product Name",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
                immutable: false,
                semantic_object: None,
            },
            FieldDef {
                name: "Price",
                label: "Price",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(15),
                scale: Some(2),
                immutable: false,
                semantic_object: None,
            },
        ]
    }

    fn simple_annotations() -> AnnotationsDef {
        AnnotationsDef {
            selection_fields: &["ProductName"],
            line_item: &[
                LineItemField {
                    name: "ProductID",
                    label: None,
                    importance: Some("High"),
                    criticality_path: None,
                    navigation_path: None,
                    semantic_object: None,
                },
                LineItemField {
                    name: "ProductName",
                    label: None,
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                    semantic_object: None,
                },
            ],
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
            field_groups: &[FieldGroupDef {
                qualifier: "Main",
                fields: &["ProductID", "ProductName", "Price"],
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
        // ProductID field: should use label from FieldDef
        assert!(xml.contains("Property=\"Value\" Path=\"ProductID\""));
        assert!(xml.contains("Property=\"Label\" String=\"Product Nr.\""));
        // High importance
        assert!(xml.contains("EnumMember=\"UI.ImportanceType/High\""));
    }

    #[test]
    fn annotations_xml_line_item_with_semantic_object() {
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[LineItemField {
                name: "CustomerID",
                label: Some("Customer"),
                importance: None,
                criticality_path: None,
                navigation_path: None,
                semantic_object: Some("Customers"),
            }],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Record Type=\"UI.DataFieldWithIntentBasedNavigation\""));
        assert!(xml.contains("Property=\"SemanticObject\" String=\"Customers\""));
        assert!(xml.contains("Property=\"Action\" String=\"display\""));
        assert!(xml.contains("Property=\"Label\" String=\"Customer\""));
    }

    #[test]
    fn annotations_xml_line_item_with_navigation_path() {
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[LineItemField {
                name: "Customer/CustomerName",
                label: Some("Kunde"),
                importance: None,
                criticality_path: None,
                navigation_path: Some("Customer"),
                semantic_object: None,
            }],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Record Type=\"UI.DataFieldWithNavigationPath\""));
        assert!(xml.contains("Property=\"Target\" NavigationPropertyPath=\"Customer\""));
        assert!(xml.contains("Property=\"Label\" String=\"Kunde\""));
    }

    #[test]
    fn annotations_xml_line_item_with_criticality() {
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[LineItemField {
                name: "Status",
                label: Some("Status"),
                importance: None,
                criticality_path: Some("StatusCriticality"),
                navigation_path: None,
                semantic_object: None,
            }],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        assert!(xml.contains("Property=\"Criticality\" Path=\"StatusCriticality\""));
    }

    #[test]
    fn annotations_xml_line_item_label_fallback() {
        // When no explicit label and no matching field → uses field name
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[LineItemField {
                name: "Unknown",
                label: None,
                importance: None,
                criticality_path: None,
                navigation_path: None,
                semantic_object: None,
            }],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
            table_facets: &[],
        };
        let xml = build_annotations_xml("Test", &def, &[]);
        // Falls back to field name as label
        assert!(xml.contains("Property=\"Label\" String=\"Unknown\""));
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
            selection_fields: &[],
            line_item: &[],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
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
            field_groups: &[],
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
            selection_fields: &[],
            line_item: &[],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
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
            field_groups: &[],
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
            selection_fields: &[],
            line_item: &[],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
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
        // Fields should have labels from FieldDef
        assert!(xml.contains("Property=\"Value\" Path=\"ProductID\""));
        assert!(xml.contains("Property=\"Label\" String=\"Product Nr.\""));
        assert!(xml.contains("Property=\"Value\" Path=\"ProductName\""));
        assert!(xml.contains("Property=\"Label\" String=\"Product Name\""));
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
            semantic_object: Some("Customers"),
        }];
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[FieldGroupDef {
                qualifier: "Main",
                fields: &["CustomerID"],
            }],
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
            semantic_object: Some("Customers"),
        }];
        let def = AnnotationsDef {
            selection_fields: &[],
            line_item: &[],
            header_info: HeaderInfoDef {
                type_name: "T", type_name_plural: "Ts",
                title_path: "X", description_path: "Y",
            },
            header_facets: &[],
            data_points: &[],
            facet_sections: &[],
            field_groups: &[],
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
    fn capabilities_draft_root() {
        let xml = build_capabilities_annotations("Products", "Product", &simple_fields(), true);
        assert!(xml.contains("Annotations Target=\"ProductsService.EntityContainer/Products\""));
        assert!(xml.contains("Org.OData.Capabilities.V1.UpdateRestrictions"));
        assert!(xml.contains("Property=\"Updatable\" Bool=\"true\""));
        assert!(xml.contains("Annotation Term=\"Common.DraftRoot\""));
        assert!(xml.contains("Record Type=\"Common.DraftRootType\""));
        assert!(xml.contains("Property=\"ActivationAction\" String=\"ProductsService.draftActivate\""));
        assert!(xml.contains("Property=\"EditAction\" String=\"ProductsService.draftEdit\""));
        assert!(xml.contains("Property=\"PreparationAction\" String=\"ProductsService.draftPrepare\""));
        assert!(!xml.contains("Common.DraftNode"));
    }

    #[test]
    fn capabilities_draft_node() {
        let xml = build_capabilities_annotations("OrderItems", "OrderItem", &simple_fields(), false);
        assert!(xml.contains("Annotations Target=\"ProductsService.EntityContainer/OrderItems\""));
        assert!(xml.contains("Annotation Term=\"Common.DraftNode\""));
        assert!(xml.contains("Record Type=\"Common.DraftNodeType\""));
        assert!(xml.contains("Property=\"PreparationAction\" String=\"ProductsService.draftPrepare\""));
        assert!(!xml.contains("Common.DraftRoot"));
        assert!(!xml.contains("EditAction"));
        assert!(!xml.contains("ActivationAction"));
    }

    #[test]
    fn capabilities_per_property_labels() {
        let xml = build_capabilities_annotations("Products", "Product", &simple_fields(), true);
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/ProductID\""));
        assert!(xml.contains("Common.Label\" String=\"Product Nr.\""));
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/ProductName\""));
        assert!(xml.contains("Common.Label\" String=\"Product Name\""));
        assert!(xml.contains("Annotations Target=\"ProductsService.Product/Price\""));
        assert!(xml.contains("Common.Label\" String=\"Price\""));
    }

    #[test]
    fn capabilities_immutable_annotation() {
        let xml = build_capabilities_annotations("Products", "Product", &simple_fields(), true);
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
        assert!(xml.contains("NavigationProperty Name=\"SiblingEntity\" Type=\"ProductsService.Product\""));
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
        }];
        append_navigation_properties(&mut xml, &navs);
        assert!(xml.contains("NavigationProperty Name=\"Items\" Type=\"Collection(ProductsService.OrderItem)\""));
    }

    #[test]
    fn append_nav_props_single() {
        let mut xml = "<EntityType Name=\"Contact\"></EntityType>".to_string();
        let navs = vec![NavigationPropertyDef {
            name: "Customer",
            target_type: "Customer",
            is_collection: false,
        }];
        append_navigation_properties(&mut xml, &navs);
        assert!(xml.contains("NavigationProperty Name=\"Customer\" Type=\"ProductsService.Customer\""));
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
            },
            NavigationPropertyDef {
                name: "Customer",
                target_type: "Customer",
                is_collection: false,
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
}
