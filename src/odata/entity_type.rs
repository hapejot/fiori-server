//! Layer 3: EntityType + EntitySet XML generation from ResolvedEntity.

use crate::model::resolved::*;
use crate::NAMESPACE;

/// Generate `<EntityType>` XML from a ResolvedEntity.
///
/// Includes Key, Properties, draft properties, NavigationProperties
/// (both from relationships and draft-internal).
pub fn generate_entity_type(e: &ResolvedEntity) -> String {
    let mut x = format!("<EntityType Name=\"{}\">", e.type_name);

    // Key
    x.push_str("<Key>");
    x.push_str(&format!("<PropertyRef Name=\"{}\"/>", e.key_field));
    x.push_str("<PropertyRef Name=\"IsActiveEntity\"/>");
    x.push_str("</Key>");

    // Properties
    for p in &e.properties {
        let mut attr = format!("Type=\"{}\"", p.edm_type);
        if p.name == e.key_field {
            attr.push_str(" Nullable=\"false\"");
        }
        if let Some(ml) = p.max_length {
            attr.push_str(&format!(" MaxLength=\"{ml}\""));
        }
        if let Some(prec) = p.precision {
            attr.push_str(&format!(" Precision=\"{prec}\""));
        }
        if let Some(sc) = p.scale {
            attr.push_str(&format!(" Scale=\"{sc}\""));
        }
        let pad = if p.name.len() < 18 {
            " ".repeat(18 - p.name.len())
        } else {
            " ".to_string()
        };
        x.push_str(&format!("<Property Name=\"{}\"{}{}/>", p.name, pad, attr));
    }

    // Draft properties
    x.push_str("<Property Name=\"IsActiveEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"true\"/>");
    x.push_str("<Property Name=\"HasActiveEntity\"  Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");
    x.push_str("<Property Name=\"HasDraftEntity\"   Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\"/>");

    // Navigation properties from relationships
    for nav in &e.nav_properties {
        if nav.is_collection {
            x.push_str(&format!(
                "<NavigationProperty Name=\"{}\" Type=\"Collection({NAMESPACE}.{})\"/>",
                nav.name, nav.target_type
            ));
        } else {
            x.push_str(&format!(
                "<NavigationProperty Name=\"{}\" Type=\"{NAMESPACE}.{}\"/>",
                nav.name, nav.target_type
            ));
        }
    }

    // Draft navigation properties
    x.push_str(&format!(
        "<NavigationProperty Name=\"SiblingEntity\" Type=\"{NAMESPACE}.{}\"/>",
        e.type_name
    ));
    x.push_str(&format!(
        "<NavigationProperty Name=\"DraftAdministrativeData\" Type=\"{NAMESPACE}.DraftAdministrativeData\" ContainsTarget=\"true\"/>"
    ));

    x.push_str("</EntityType>");
    x
}

/// Generate `<EntitySet>` XML from a ResolvedEntity.
///
/// Includes NavigationPropertyBindings for all nav properties + draft navs.
pub fn generate_entity_set(e: &ResolvedEntity) -> String {
    let mut x = format!(
        "<EntitySet Name=\"{}\" EntityType=\"{NAMESPACE}.{}\">",
        e.set_name, e.type_name
    );

    for nav in &e.nav_properties {
        x.push_str(&format!(
            "<NavigationPropertyBinding Path=\"{}\" Target=\"{}\"/>",
            nav.name, nav.target_set
        ));
    }

    // Draft bindings
    x.push_str(&format!(
        "<NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"{}\"/>",
        e.set_name
    ));
    x.push_str(
        "<NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>"
    );

    x.push_str("</EntitySet>");
    x
}

/// Generate draft action XMLs (draftEdit, draftActivate, draftPrepare) for an entity.
pub fn generate_draft_actions(e: &ResolvedEntity) -> String {
    let fqn = format!("{NAMESPACE}.{}", e.type_name);
    let mut x = String::new();

    x.push_str(&format!(
        "<Action Name=\"draftEdit\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <Parameter Name=\"PreserveChanges\" Type=\"Edm.Boolean\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    x.push_str(&format!(
        "<Action Name=\"draftActivate\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    x.push_str(&format!(
        "<Action Name=\"draftPrepare\" IsBound=\"true\" EntitySetPath=\"in\">\
         <Parameter Name=\"in\" Type=\"{fqn}\"/>\
         <Parameter Name=\"SideEffectsQualifier\" Type=\"Edm.String\"/>\
         <ReturnType Type=\"{fqn}\"/>\
         </Action>"
    ));
    x
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
            description_field: None,
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
                    presentation: ResolvedPresentation::default(),
                    package: None,
                },
            ],
            nav_properties: vec![
                ResolvedNavProperty {
                    name: "Items".into(),
                    target_type: "OrderItem".into(),
                    target_set: "OrderItems".into(),
                    is_collection: true,
                    foreign_key: Some("OrderID".into()),
                    relationship: "Order_Items".into(),
                    is_composition: true,
                },
            ],
            data_points: vec![],
            header_facets: vec![],
            facet_sections: vec![],
            table_facets: vec![],
            selection_fields: vec![],
            package: None,
        }
    }

    #[test]
    fn test_entity_type_xml() {
        let e = sample_entity();
        let xml = generate_entity_type(&e);

        assert!(xml.starts_with("<EntityType Name=\"Order\">"));
        assert!(xml.contains("<PropertyRef Name=\"ID\"/>"));
        assert!(xml.contains("<PropertyRef Name=\"IsActiveEntity\"/>"));
        assert!(xml.contains("Name=\"ID\""));
        assert!(xml.contains("Type=\"Edm.Guid\" Nullable=\"false\""));
        assert!(xml.contains("Name=\"OrderName\""));
        assert!(xml.contains("MaxLength=\"80\""));
        assert!(xml.contains("Type=\"Collection(Service.OrderItem)\""));
        assert!(xml.contains("Name=\"SiblingEntity\""));
        assert!(xml.contains("Name=\"DraftAdministrativeData\""));
        assert!(xml.ends_with("</EntityType>"));
    }

    #[test]
    fn test_entity_set_xml() {
        let e = sample_entity();
        let xml = generate_entity_set(&e);

        assert!(xml.starts_with("<EntitySet Name=\"Orders\""));
        assert!(xml.contains("Path=\"Items\" Target=\"OrderItems\""));
        assert!(xml.contains("Path=\"SiblingEntity\" Target=\"Orders\""));
        assert!(xml.contains("Path=\"DraftAdministrativeData\""));
        assert!(xml.ends_with("</EntitySet>"));
    }

    #[test]
    fn test_draft_actions_xml() {
        let e = sample_entity();
        let xml = generate_draft_actions(&e);

        assert!(xml.contains("Name=\"draftEdit\""));
        assert!(xml.contains("Name=\"draftActivate\""));
        assert!(xml.contains("Name=\"draftPrepare\""));
        assert!(xml.contains("Type=\"Service.Order\""));
    }
}
