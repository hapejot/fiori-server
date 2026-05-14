//! Comprehensive metadata XML generation test.
//!
//! Builds a rich sample entity (parent "Order" + child "OrderItem") covering all
//! annotation features, generates the full EDMX document via `build_metadata_xml`,
//! then asserts on every aspect: EDMX structure, EntityType, EntitySet, draft actions,
//! UI annotations, capability annotations, and per-property annotations.

use std::fs::File;

use simple_fiori_server::builders::build_metadata_xml;
use simple_fiori_server::model::resolved::*;
use simple_fiori_server::spec::{MeasureKind, ValueListFilter};

// ══════════════════════════════════════════════════════════════
//  Sample entities
// ══════════════════════════════════════════════════════════════

/// Rich parent entity exercising every annotation feature.
fn order_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "Orders".into(),
        type_name: "Order".into(),
        type_name_plural: "Orders".into(),
        key_field: "ID".into(),
        title_field: "OrderName".into(),
        description_field: Some("Status".into()),
        parent_set_name: None, // DraftRoot
        properties: vec![
            // 1. Key field: Edm.Guid, computed, hidden → Core.Computed, UI.Hidden, Common.Text on key
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
            // 2. Title field: searchable, in list, form group
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
                    list_importance: Some("High".into()),
                    criticality_path: None,
                    form_group: Some("General".into()),
                },
                package: None,
            },
            // 3. Status with CodeList value list + text_path + criticality
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
                    searchable: true,
                    show_in_list: true,
                    list_sort_order: Some(2),
                    list_importance: None,
                    criticality_path: Some("StatusCriticality".into()),
                    form_group: Some("General".into()),
                },
                package: None,
            },
            // 4. FK field: EntityRef value list → SemanticObject + IBN
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
            // 5. Decimal with measure (Currency)
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
                    show_in_list: true,
                    list_sort_order: Some(4),
                    form_group: Some("Pricing".into()),
                    ..Default::default()
                },
                package: None,
            },
            // 6. Currency companion (computed)
            ResolvedProperty {
                name: "Currency".into(),
                edm_type: "Edm.String".into(),
                label: "Currency".into(),
                max_length: Some(3),
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    form_group: Some("Pricing".into()),
                    ..Default::default()
                },
                package: None,
            },
            // 7. Immutable field
            ResolvedProperty {
                name: "OrderDate".into(),
                edm_type: "Edm.Date".into(),
                label: "Order Date".into(),
                max_length: None,
                precision: None,
                scale: None,
                computed: false,
                immutable: true,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    show_in_list: true,
                    list_sort_order: Some(5),
                    form_group: Some("General".into()),
                    ..Default::default()
                },
                package: None,
            },
            // 8. Hidden computed field (StatusCriticality)
            ResolvedProperty {
                name: "StatusCriticality".into(),
                edm_type: "Edm.Byte".into(),
                label: "Status Criticality".into(),
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
            // 9. Hidden computed text field (_Status_text)
            ResolvedProperty {
                name: "_Status_text".into(),
                edm_type: "Edm.String".into(),
                label: "Status Text".into(),
                max_length: Some(100),
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
            // 10. FK field with ValueListParameterIn filter
            ResolvedProperty {
                name: "RegionID".into(),
                edm_type: "Edm.Guid".into(),
                label: "Region".into(),
                max_length: None,
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: true,
                text_path: Some("Region/RegionName".into()),
                value_list: Some(ResolvedValueList::EntityRef {
                    collection_path: "Regions".into(),
                    key_property: "ID".into(),
                    display_property: Some("RegionName".into()),
                    filters: vec![ValueListFilter {
                        local_property: "CountryCode".into(),
                        target_property: "Country".into(),
                    }],
                    fixed_values: false,
                }),
                measure: None,
                presentation: ResolvedPresentation {
                    form_group: Some("General".into()),
                    ..Default::default()
                },
                package: None,
            },
        ],
        nav_properties: vec![
            // Composition: Order → OrderItems
            ResolvedNavProperty {
                name: "Items".into(),
                target_type: "OrderItem".into(),
                target_set: "OrderItems".into(),
                is_collection: true,
                foreign_key: Some("OrderID".into()),
                relationship: "Order_Items".into(),
                is_composition: true,
            },
            // 1:1 reference: Order → Customer
            ResolvedNavProperty {
                name: "Customer".into(),
                target_type: "Customer".into(),
                target_set: "Customers".into(),
                is_collection: false,
                foreign_key: Some("CustomerID".into()),
                relationship: "Order_Customer".into(),
                is_composition: false,
            },
        ],
        data_points: vec![
            ResolvedDataPoint {
                qualifier: "Price".into(),
                value_path: "Price".into(),
                title: "Price".into(),
                max_value: None,
                visualization: None,
            },
            ResolvedDataPoint {
                qualifier: "Stock".into(),
                value_path: "Stock".into(),
                title: "Stock Level".into(),
                max_value: Some(100),
                visualization: Some("Progress".into()),
            },
        ],
        header_facets: vec![
            ResolvedHeaderFacet {
                data_point_qualifier: "Price".into(),
                label: "Price".into(),
            },
            ResolvedHeaderFacet {
                data_point_qualifier: "Stock".into(),
                label: "Stock".into(),
            },
        ],
        facet_sections: vec![
            ResolvedFacetSection {
                label: "General Information".into(),
                id: "GeneralSection".into(),
                field_group_qualifier: "General".into(),
            },
            ResolvedFacetSection {
                label: "Pricing".into(),
                id: "PricingSection".into(),
                field_group_qualifier: "Pricing".into(),
            },
        ],
        table_facets: vec![ResolvedTableFacet {
            label: "Order Items".into(),
            id: "ItemsSection".into(),
            navigation_property: "Items".into(),
        }],
        selection_fields: vec!["OrderName".into(), "Status".into()],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

/// Child entity — exercises DraftNode (not DraftRoot).
fn order_item_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "OrderItems".into(),
        type_name: "OrderItem".into(),
        type_name_plural: "Order Items".into(),
        key_field: "ID".into(),
        title_field: "Description".into(),
        description_field: None,
        parent_set_name: Some("Orders".into()), // DraftNode
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
                name: "Description".into(),
                edm_type: "Edm.String".into(),
                label: "Description".into(),
                max_length: Some(200),
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    show_in_list: true,
                    list_sort_order: Some(1),
                    ..Default::default()
                },
                package: None,
            },
            ResolvedProperty {
                name: "Quantity".into(),
                edm_type: "Edm.Int32".into(),
                label: "Quantity".into(),
                max_length: None,
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: Some(ResolvedMeasure {
                    unit_field: "UoM".into(),
                    kind: MeasureKind::Unit,
                }),
                presentation: ResolvedPresentation {
                    show_in_list: true,
                    list_sort_order: Some(2),
                    form_group: Some("Details".into()),
                    ..Default::default()
                },
                package: None,
            },
            ResolvedProperty {
                name: "UoM".into(),
                edm_type: "Edm.String".into(),
                label: "Unit".into(),
                max_length: Some(3),
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    form_group: Some("Details".into()),
                    ..Default::default()
                },
                package: None,
            },
            // Computed FK back to parent
            ResolvedProperty {
                name: "OrderID".into(),
                edm_type: "Edm.Guid".into(),
                label: "Order".into(),
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
        ],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![ResolvedFacetSection {
            label: "Details".into(),
            id: "DetailsSection".into(),
            field_group_qualifier: "Details".into(),
        }],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

/// Referenced entity — standalone root with no parent, referenced by Order.CustomerID.
fn customer_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "Customers".into(),
        type_name: "Customer".into(),
        type_name_plural: "Customers".into(),
        key_field: "ID".into(),
        title_field: "CustomerName".into(),
        description_field: Some("City".into()),
        parent_set_name: None, // DraftRoot
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
                name: "CustomerName".into(),
                edm_type: "Edm.String".into(),
                label: "Customer Name".into(),
                max_length: Some(120),
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
                    list_importance: Some("High".into()),
                    criticality_path: None,
                    form_group: Some("General".into()),
                },
                package: None,
            },
            ResolvedProperty {
                name: "City".into(),
                edm_type: "Edm.String".into(),
                label: "City".into(),
                max_length: Some(60),
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
                    list_sort_order: Some(2),
                    form_group: Some("General".into()),
                    ..Default::default()
                },
                package: None,
            },
            ResolvedProperty {
                name: "Email".into(),
                edm_type: "Edm.String".into(),
                label: "Email".into(),
                max_length: Some(255),
                precision: None,
                scale: None,
                computed: false,
                immutable: false,
                hidden: false,
                text_path: None,
                value_list: None,
                measure: None,
                presentation: ResolvedPresentation {
                    show_in_list: true,
                    list_sort_order: Some(3),
                    form_group: Some("Contact".into()),
                    ..Default::default()
                },
                package: None,
            },
        ],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![
            ResolvedFacetSection {
                label: "General".into(),
                id: "GeneralSection".into(),
                field_group_qualifier: "General".into(),
            },
            ResolvedFacetSection {
                label: "Contact".into(),
                id: "ContactSection".into(),
                field_group_qualifier: "Contact".into(),
            },
        ],
        table_facets: vec![],
        selection_fields: vec!["CustomerName".into(), "City".into()],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

/// Build the full EDMX from all three entities.
fn build_test_edmx() -> String {
    let entities = vec![order_entity(), order_item_entity(), customer_entity()];
    build_metadata_xml(&entities)
}

// ══════════════════════════════════════════════════════════════
//  EDMX document structure
// ══════════════════════════════════════════════════════════════

#[test]
fn edmx_printing() {
    use std::io::Write;
    let mut f = File::create("test_edmx.xml").unwrap();
    let xml = build_test_edmx();
    f.write_all(xml.as_bytes()).unwrap();
}

#[test]
fn edmx_has_xml_declaration() {
    let xml = build_test_edmx();
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn edmx_has_version_4() {
    let xml = build_test_edmx();
    assert!(xml.contains("edmx:Edmx Version=\"4.0\""));
}

#[test]
fn edmx_references_all_vocabularies() {
    let xml = build_test_edmx();
    assert!(xml.contains("Org.OData.Capabilities.V1"));
    assert!(xml.contains("Org.OData.Core.V1"));
    assert!(xml.contains("Org.OData.Measures.V1"));
    assert!(xml.contains("com.sap.vocabularies.UI.v1"));
    assert!(xml.contains("com.sap.vocabularies.Common.v1"));
}

#[test]
fn edmx_has_schema_namespace() {
    let xml = build_test_edmx();
    assert!(xml.contains("Namespace=\"Service\""));
}

#[test]
fn edmx_has_entity_container() {
    let xml = build_test_edmx();
    assert!(xml.contains("<EntityContainer Name=\"EntityContainer\">"));
    assert!(xml.contains("</EntityContainer>"));
}

// ══════════════════════════════════════════════════════════════
//  EntityType — Order (parent)
// ══════════════════════════════════════════════════════════════

#[test]
fn order_entity_type_exists() {
    let xml = build_test_edmx();
    assert!(xml.contains("<EntityType Name=\"Order\">"));
    assert!(xml.contains("</EntityType>"));
}

#[test]
fn order_key_has_id_and_is_active() {
    let xml = build_test_edmx();
    assert!(xml.contains("<PropertyRef Name=\"ID\"/>"));
    assert!(xml.contains("<PropertyRef Name=\"IsActiveEntity\"/>"));
}

#[test]
fn order_property_id_guid_not_nullable() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"ID\""));
    assert!(xml.contains("Type=\"Edm.Guid\" Nullable=\"false\""));
}

#[test]
fn order_property_name_string_with_max_length() {
    let xml = build_test_edmx();
    // OrderName: Edm.String with MaxLength=80
    assert!(xml.contains("Name=\"OrderName\""));
    assert!(xml.contains("MaxLength=\"80\""));
}

#[test]
fn order_property_price_decimal_with_precision_scale() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"Price\""));
    assert!(xml.contains("Precision=\"12\""));
    assert!(xml.contains("Scale=\"2\""));
}

#[test]
fn order_property_date_type() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"OrderDate\""));
    assert!(xml.contains("Type=\"Edm.Date\""));
}

#[test]
fn order_has_draft_properties() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"IsActiveEntity\""));
    assert!(xml.contains("Name=\"HasActiveEntity\""));
    assert!(xml.contains("Name=\"HasDraftEntity\""));
    // All boolean, not nullable, with defaults
    assert!(xml.contains("Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"true\""));
    assert!(xml.contains("Type=\"Edm.Boolean\" Nullable=\"false\" DefaultValue=\"false\""));
}

#[test]
fn order_nav_property_items_collection() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"Items\" Type=\"Collection(Service.OrderItem)\""));
}

#[test]
fn order_nav_property_customer_single() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"Customer\" Type=\"Service.Customer\""));
}

#[test]
fn order_has_sibling_entity_nav() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"SiblingEntity\" Type=\"Service.Order\""));
}

#[test]
fn order_has_draft_admin_data_nav() {
    let xml = build_test_edmx();
    assert!(xml.contains(
        "Name=\"DraftAdministrativeData\" Type=\"Service.DraftAdministrativeData\" ContainsTarget=\"true\""
    ));
}

// ══════════════════════════════════════════════════════════════
//  EntityType — OrderItem (child)
// ══════════════════════════════════════════════════════════════

#[test]
fn order_item_entity_type_exists() {
    let xml = build_test_edmx();
    assert!(xml.contains("<EntityType Name=\"OrderItem\">"));
}

#[test]
fn order_item_has_order_id_fk() {
    let xml = build_test_edmx();
    // OrderID is a Guid FK back to parent
    assert!(xml.contains("Name=\"OrderID\""));
}

#[test]
fn order_item_has_quantity_int() {
    let xml = build_test_edmx();
    assert!(xml.contains("Name=\"Quantity\""));
    assert!(xml.contains("Type=\"Edm.Int32\""));
}

// ══════════════════════════════════════════════════════════════
//  DraftAdministrativeData type
// ══════════════════════════════════════════════════════════════

#[test]
fn draft_admin_type_exists() {
    let xml = build_test_edmx();
    assert!(xml.contains("<EntityType Name=\"DraftAdministrativeData\">"));
    assert!(xml.contains("Name=\"DraftUUID\""));
    assert!(xml.contains("Name=\"CreationDateTime\""));
    assert!(xml.contains("Name=\"CreatedByUser\""));
    assert!(xml.contains("Name=\"DraftIsCreatedByMe\""));
    assert!(xml.contains("Name=\"LastChangeDateTime\""));
    assert!(xml.contains("Name=\"LastChangedByUser\""));
    assert!(xml.contains("Name=\"InProcessByUser\""));
    assert!(xml.contains("Name=\"DraftIsProcessedByMe\""));
}

// ══════════════════════════════════════════════════════════════
//  EntitySet — both entities
// ══════════════════════════════════════════════════════════════

#[test]
fn order_entity_set() {
    let xml = build_test_edmx();
    assert!(xml.contains("EntitySet Name=\"Orders\" EntityType=\"Service.Order\""));
}

#[test]
fn order_item_entity_set() {
    let xml = build_test_edmx();
    assert!(xml.contains("EntitySet Name=\"OrderItems\" EntityType=\"Service.OrderItem\""));
}

#[test]
fn order_set_binds_items_nav() {
    let xml = build_test_edmx();
    assert!(xml.contains("Path=\"Items\" Target=\"OrderItems\""));
}

#[test]
fn order_set_binds_customer_nav() {
    let xml = build_test_edmx();
    assert!(xml.contains("Path=\"Customer\" Target=\"Customers\""));
}

#[test]
fn order_set_binds_sibling() {
    let xml = build_test_edmx();
    assert!(xml.contains("Path=\"SiblingEntity\" Target=\"Orders\""));
}

#[test]
fn order_set_binds_draft_admin() {
    let xml = build_test_edmx();
    assert!(xml.contains("Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\""));
}

#[test]
fn draft_admin_entity_set() {
    let xml = build_test_edmx();
    assert!(xml.contains(
        "EntitySet Name=\"DraftAdministrativeData\" EntityType=\"Service.DraftAdministrativeData\""
    ));
}

// ══════════════════════════════════════════════════════════════
//  Draft actions
// ══════════════════════════════════════════════════════════════

#[test]
fn draft_edit_action() {
    let xml = build_test_edmx();
    assert!(xml.contains("Action Name=\"draftEdit\" IsBound=\"true\""));
    assert!(xml.contains("Parameter Name=\"PreserveChanges\" Type=\"Edm.Boolean\""));
}

#[test]
fn draft_activate_action() {
    let xml = build_test_edmx();
    assert!(xml.contains("Action Name=\"draftActivate\" IsBound=\"true\""));
}

#[test]
fn draft_prepare_action() {
    let xml = build_test_edmx();
    assert!(xml.contains("Action Name=\"draftPrepare\" IsBound=\"true\""));
    assert!(xml.contains("Parameter Name=\"SideEffectsQualifier\" Type=\"Edm.String\""));
}

#[test]
fn draft_actions_bound_to_order_type() {
    let xml = build_test_edmx();
    assert!(xml.contains("Parameter Name=\"in\" Type=\"Service.Order\""));
}

#[test]
fn draft_actions_bound_to_order_item_type() {
    let xml = build_test_edmx();
    assert!(xml.contains("Parameter Name=\"in\" Type=\"Service.OrderItem\""));
}

// ══════════════════════════════════════════════════════════════
//  UI.SelectionFields
// ══════════════════════════════════════════════════════════════

#[test]
fn ui_selection_fields() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.SelectionFields\""));
    assert!(xml.contains("<PropertyPath>OrderName</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>Status</PropertyPath>"));
}

// ══════════════════════════════════════════════════════════════
//  UI.LineItem
// ══════════════════════════════════════════════════════════════

#[test]
fn ui_line_item_exists() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.LineItem\""));
}

#[test]
fn line_item_has_data_field_for_name() {
    let xml = build_test_edmx();
    // OrderName: UI.DataField with Value path and High importance
    assert!(xml.contains("Record Type=\"UI.DataField\""));
    assert!(xml.contains("Property=\"Value\" Path=\"OrderName\""));
    assert!(xml.contains("UI.ImportanceType/High"));
}

#[test]
fn line_item_has_ibn_for_customer() {
    let xml = build_test_edmx();
    // CustomerID with EntityRef → DataFieldWithIntentBasedNavigation
    assert!(xml.contains("Record Type=\"UI.DataFieldWithIntentBasedNavigation\""));
    assert!(xml.contains("Property=\"SemanticObject\" String=\"Customers\""));
    assert!(xml.contains("Property=\"Action\" String=\"display\""));
}

#[test]
fn line_item_has_criticality_on_status() {
    let xml = build_test_edmx();
    assert!(xml.contains("Property=\"Criticality\" Path=\"StatusCriticality\""));
}

// ══════════════════════════════════════════════════════════════
//  UI.HeaderInfo
// ══════════════════════════════════════════════════════════════

#[test]
fn ui_header_info() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.HeaderInfo\""));
    assert!(xml.contains("Record Type=\"UI.HeaderInfoType\""));
    assert!(xml.contains("Property=\"TypeName\" String=\"Order\""));
    assert!(xml.contains("Property=\"TypeNamePlural\" String=\"Orders\""));
}

#[test]
fn header_info_title_field() {
    let xml = build_test_edmx();
    // Title → DataField with Value=OrderName
    assert!(xml.contains("Property=\"Title\""));
    assert!(xml.contains("Property=\"Value\" Path=\"OrderName\""));
}

#[test]
fn header_info_description_field() {
    let xml = build_test_edmx();
    // Description → DataField with Value=Status
    assert!(xml.contains("Property=\"Description\""));
    assert!(xml.contains("Property=\"Value\" Path=\"Status\""));
}

// ══════════════════════════════════════════════════════════════
//  UI.HeaderFacets
// ══════════════════════════════════════════════════════════════

#[test]
fn ui_header_facets() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.HeaderFacets\""));
    // Two ReferenceFacets pointing to DataPoints
    assert!(xml.contains("AnnotationPath=\"@UI.DataPoint#Price\""));
    assert!(xml.contains("AnnotationPath=\"@UI.DataPoint#Stock\""));
}

// ══════════════════════════════════════════════════════════════
//  UI.DataPoint
// ══════════════════════════════════════════════════════════════

#[test]
fn data_point_price() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.DataPoint\" Qualifier=\"Price\""));
    assert!(xml.contains("Record Type=\"UI.DataPointType\""));
    assert!(xml.contains("Property=\"Value\" Path=\"Price\""));
    assert!(xml.contains("Property=\"Title\" String=\"Price\""));
}

#[test]
fn data_point_stock_with_progress_visualization() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.DataPoint\" Qualifier=\"Stock\""));
    assert!(xml.contains("Property=\"MaximumValue\" Int=\"100\""));
    assert!(xml.contains("UI.VisualizationType/Progress"));
}

// ══════════════════════════════════════════════════════════════
//  UI.Facets
// ══════════════════════════════════════════════════════════════

#[test]
fn ui_facets_has_reference_facets() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.Facets\""));
    assert!(xml.contains("Record Type=\"UI.ReferenceFacet\""));
    assert!(xml.contains("Property=\"Label\" String=\"General Information\""));
    assert!(xml.contains("Property=\"Label\" String=\"Pricing\""));
}

#[test]
fn facets_reference_field_groups() {
    let xml = build_test_edmx();
    assert!(xml.contains("AnnotationPath=\"@UI.FieldGroup#General\""));
    assert!(xml.contains("AnnotationPath=\"@UI.FieldGroup#Pricing\""));
}

#[test]
fn facets_include_table_facet_for_items() {
    let xml = build_test_edmx();
    // Table facet: ReferenceFacet pointing to Items/@UI.LineItem
    assert!(xml.contains("AnnotationPath=\"Items/@UI.LineItem\""));
    assert!(xml.contains("Property=\"Label\" String=\"Order Items\""));
}

// ══════════════════════════════════════════════════════════════
//  UI.FieldGroup
// ══════════════════════════════════════════════════════════════

#[test]
fn field_group_general() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.FieldGroup\" Qualifier=\"General\""));
    assert!(xml.contains("Record Type=\"UI.FieldGroupType\""));
}

#[test]
fn field_group_pricing() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.FieldGroup\" Qualifier=\"Pricing\""));
}

#[test]
fn field_group_general_contains_order_name() {
    let xml = build_test_edmx();
    // General field group should contain OrderName, Status, CustomerID, OrderDate, RegionID
    // CustomerID has EntityRef → DataFieldWithIntentBasedNavigation
    // Others → DataField
    assert!(xml.contains("Property=\"Value\" Path=\"OrderName\""));
    assert!(xml.contains("Property=\"Value\" Path=\"OrderDate\""));
}

// ══════════════════════════════════════════════════════════════
//  Common.SemanticObject (property-level)
// ══════════════════════════════════════════════════════════════

#[test]
fn semantic_object_on_customer_id() {
    let xml = build_test_edmx();
    // Property-level block: Target="Service.Order/CustomerID"
    assert!(xml.contains("Target=\"Service.Order/CustomerID\""));
    assert!(xml.contains("Term=\"Common.SemanticObject\" String=\"Customers\""));
}

#[test]
fn semantic_object_mapping() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Common.SemanticObjectMapping\""));
    assert!(xml.contains("Record Type=\"Common.SemanticObjectMappingType\""));
    // Maps CustomerID → ID
    assert!(xml.contains("Property=\"SemanticObjectProperty\" String=\"ID\""));
}

// ══════════════════════════════════════════════════════════════
//  Capability annotations — EntitySet level
// ══════════════════════════════════════════════════════════════

#[test]
fn capability_target_is_entity_container() {
    let xml = build_test_edmx();
    assert!(xml.contains("Target=\"Service.EntityContainer/Orders\""));
    assert!(xml.contains("Target=\"Service.EntityContainer/OrderItems\""));
    assert!(xml.contains("Target=\"Service.EntityContainer/Customers\""));
}

#[test]
fn update_restrictions() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Org.OData.Capabilities.V1.UpdateRestrictions\""));
    assert!(xml.contains("Property=\"Updatable\" Bool=\"true\""));
}

#[test]
fn insert_restrictions_has_computed_fields() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Org.OData.Capabilities.V1.InsertRestrictions\""));
    assert!(xml.contains("Record Type=\"Capabilities.InsertRestrictionsType\""));
    // Computed fields are listed as non-insertable
    assert!(xml.contains("<PropertyPath>ID</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>StatusCriticality</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>_Status_text</PropertyPath>"));
    // Draft flags always included
    assert!(xml.contains("<PropertyPath>IsActiveEntity</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>HasActiveEntity</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>HasDraftEntity</PropertyPath>"));
}

// ══════════════════════════════════════════════════════════════
//  DraftRoot vs DraftNode
// ══════════════════════════════════════════════════════════════

#[test]
fn order_has_draft_root() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Common.DraftRoot\""));
    assert!(xml.contains("Record Type=\"Common.DraftRootType\""));
    assert!(xml.contains("Property=\"ActivationAction\" String=\"Service.draftActivate\""));
    assert!(xml.contains("Property=\"EditAction\" String=\"Service.draftEdit\""));
    assert!(xml.contains("Property=\"PreparationAction\" String=\"Service.draftPrepare\""));
}

#[test]
fn order_item_has_draft_node() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Common.DraftNode\""));
    assert!(xml.contains("Record Type=\"Common.DraftNodeType\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — Common.Label
// ══════════════════════════════════════════════════════════════

#[test]
fn every_property_has_label() {
    let xml = build_test_edmx();
    // Spot-check labels
    assert!(xml.contains("Term=\"Common.Label\" String=\"Order Name\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"Status\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"Customer\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"Price\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"Order Date\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — UI.Hidden
// ══════════════════════════════════════════════════════════════

#[test]
fn hidden_properties() {
    let xml = build_test_edmx();
    // ID, CustomerID, StatusCriticality, _Status_text, RegionID are hidden
    assert!(xml.contains("Term=\"UI.Hidden\" Bool=\"true\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — Core.Computed / Core.Immutable
// ══════════════════════════════════════════════════════════════

#[test]
fn computed_properties() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Org.OData.Core.V1.Computed\" Bool=\"true\""));
}

#[test]
fn immutable_property() {
    let xml = build_test_edmx();
    // OrderDate is immutable
    assert!(xml.contains("Term=\"Org.OData.Core.V1.Immutable\" Bool=\"true\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — Common.Text + UI.TextArrangement
// ══════════════════════════════════════════════════════════════

#[test]
fn common_text_on_key_field() {
    let xml = build_test_edmx();
    // ID key field → Common.Text pointing to OrderName (title_field)
    assert!(xml.contains("Target=\"Service.Order/ID\""));
    assert!(xml.contains("Term=\"Common.Text\" Path=\"OrderName\""));
}

#[test]
fn text_arrangement_text_only() {
    let xml = build_test_edmx();
    assert!(
        xml.contains("Term=\"UI.TextArrangement\" EnumMember=\"UI.TextArrangementType/TextOnly\"")
    );
}

#[test]
fn common_text_on_status_field() {
    let xml = build_test_edmx();
    // Status → text_path = "_Status_text"
    assert!(xml.contains("Term=\"Common.Text\" Path=\"_Status_text\""));
}

#[test]
fn common_text_on_customer_fk() {
    let xml = build_test_edmx();
    // CustomerID → text_path = "Customer/CustomerName"
    assert!(xml.contains("Term=\"Common.Text\" Path=\"Customer/CustomerName\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — Measures
// ══════════════════════════════════════════════════════════════

#[test]
fn iso_currency_on_price() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Org.OData.Measures.V1.ISOCurrency\" Path=\"Currency\""));
}

#[test]
fn unit_on_quantity() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Org.OData.Measures.V1.Unit\" Path=\"UoM\""));
}

// ══════════════════════════════════════════════════════════════
//  Per-property annotations — Common.ValueList
// ══════════════════════════════════════════════════════════════

#[test]
fn value_list_code_list_on_status() {
    let xml = build_test_edmx();
    // Status has CodeList → FieldValueListItems with Constant filter
    assert!(xml.contains("Term=\"Common.ValueList\""));
    assert!(xml.contains("Record Type=\"Common.ValueListType\""));
    assert!(xml.contains("Property=\"CollectionPath\" String=\"FieldValueListItems\""));
    assert!(xml.contains("Record Type=\"Common.ValueListParameterOut\""));
    assert!(xml.contains("Property=\"ValueListProperty\" String=\"Code\""));
    assert!(xml.contains("Record Type=\"Common.ValueListParameterDisplayOnly\""));
    assert!(xml.contains("Property=\"ValueListProperty\" String=\"Description\""));
    assert!(xml.contains("Record Type=\"Common.ValueListParameterConstant\""));
    assert!(xml.contains("Property=\"Constant\" String=\"abc-123\""));
}

#[test]
fn value_list_with_fixed_values_on_status() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Common.ValueListWithFixedValues\" Bool=\"true\""));
}

#[test]
fn value_list_entity_ref_on_customer() {
    let xml = build_test_edmx();
    assert!(xml.contains("Property=\"CollectionPath\" String=\"Customers\""));
    assert!(xml.contains("Property=\"ValueListProperty\" String=\"CustomerName\""));
}

#[test]
fn value_list_with_filter_on_region() {
    let xml = build_test_edmx();
    // RegionID has a ValueListParameterIn filter: CountryCode → Country
    assert!(xml.contains("Record Type=\"Common.ValueListParameterIn\""));
    assert!(xml.contains("PropertyPath=\"CountryCode\""));
    assert!(xml.contains("Property=\"ValueListProperty\" String=\"Country\""));
}

// ══════════════════════════════════════════════════════════════
//  Draft flag properties — Core.Computed
// ══════════════════════════════════════════════════════════════

#[test]
fn draft_flag_properties_are_computed() {
    let xml = build_test_edmx();
    // IsActiveEntity, HasActiveEntity, HasDraftEntity each get their own
    // annotation block with Core.Computed
    assert!(xml.contains("Target=\"Service.Order/IsActiveEntity\""));
    assert!(xml.contains("Target=\"Service.Order/HasActiveEntity\""));
    assert!(xml.contains("Target=\"Service.Order/HasDraftEntity\""));
    assert!(xml.contains("Target=\"Service.OrderItem/IsActiveEntity\""));
}

// ══════════════════════════════════════════════════════════════
//  Child entity — OrderItem-specific
// ══════════════════════════════════════════════════════════════

#[test]
fn order_item_ui_annotations_target() {
    let xml = build_test_edmx();
    assert!(xml.contains("Target=\"Service.OrderItem\""));
}

#[test]
fn order_item_line_item() {
    let xml = build_test_edmx();
    // OrderItem has Description and Quantity in LineItem
    assert!(xml.contains("Property=\"Value\" Path=\"Description\""));
    assert!(xml.contains("Property=\"Value\" Path=\"Quantity\""));
}

#[test]
fn order_item_field_group_details() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.FieldGroup\" Qualifier=\"Details\""));
}

#[test]
fn order_item_header_info() {
    let xml = build_test_edmx();
    assert!(xml.contains("Property=\"TypeName\" String=\"OrderItem\""));
    assert!(xml.contains("Property=\"TypeNamePlural\" String=\"Order Items\""));
}

// ══════════════════════════════════════════════════════════════
//  Extra/custom XML passthrough
// ══════════════════════════════════════════════════════════════

#[test]
fn extra_annotations_xml_appended() {
    let mut entities = vec![order_entity(), order_item_entity(), customer_entity()];
    entities[0].extra_annotations_xml =
        "<Annotations Target=\"Service.Order\"><Annotation Term=\"UI.Identification\"/></Annotations>".into();
    let xml = build_metadata_xml(&entities);
    assert!(xml.contains("Term=\"UI.Identification\""));
}

#[test]
fn custom_actions_xml_appended() {
    let mut entities = vec![order_entity(), order_item_entity(), customer_entity()];
    entities[0].custom_actions_xml =
        "<Action Name=\"publishConfig\" IsBound=\"true\"><Parameter Name=\"in\" Type=\"Service.Order\"/></Action>"
            .into();
    let xml = build_metadata_xml(&entities);
    assert!(xml.contains("Action Name=\"publishConfig\""));
}

// ══════════════════════════════════════════════════════════════
//  Well-formedness: every opened tag is closed
// ══════════════════════════════════════════════════════════════

#[test]
fn xml_well_formed_entity_types() {
    let xml = build_test_edmx();
    let open = xml.matches("<EntityType ").count();
    let close = xml.matches("</EntityType>").count();
    // Order + OrderItem + Customer + DraftAdministrativeData = 4
    assert_eq!(open, 4, "Expected 4 EntityTypes");
    assert_eq!(open, close, "Mismatched EntityType open/close tags");
}

#[test]
fn xml_well_formed_entity_sets() {
    let xml = build_test_edmx();
    let open = xml.matches("<EntitySet ").count();
    let close = xml.matches("</EntitySet>").count();
    // Orders + OrderItems + Customers + DraftAdministrativeData = 4
    // But DraftAdministrativeData is self-closing → check manually
    assert!(open >= 3);
    // Close tags only for Orders, OrderItems, Customers (DraftAdmin is within Container)
    assert_eq!(close, 3);
}

#[test]
fn xml_well_formed_annotations() {
    let xml = build_test_edmx();
    let open = xml.matches("<Annotations ").count();
    let close = xml.matches("</Annotations>").count();
    assert_eq!(open, close, "Mismatched Annotations open/close tags");
    // At minimum: 2 entity-level + 2 entity-set + many per-property
    assert!(open > 10, "Expected many Annotations blocks, got {open}");
}

#[test]
fn xml_well_formed_schema() {
    let xml = build_test_edmx();
    assert_eq!(xml.matches("<Schema ").count(), 1);
    assert_eq!(xml.matches("</Schema>").count(), 1);
}

// ══════════════════════════════════════════════════════════════
//  Customer entity — standalone referenced entity
// ══════════════════════════════════════════════════════════════

#[test]
fn customer_entity_type_exists() {
    let xml = build_test_edmx();
    assert!(xml.contains("<EntityType Name=\"Customer\">"));
}

#[test]
fn customer_entity_set() {
    let xml = build_test_edmx();
    assert!(xml.contains("EntitySet Name=\"Customers\" EntityType=\"Service.Customer\""));
}

#[test]
fn customer_has_draft_root() {
    let xml = build_test_edmx();
    // Customer is a standalone root entity
    assert!(xml.contains("Target=\"Service.EntityContainer/Customers\""));
}

#[test]
fn customer_header_info() {
    let xml = build_test_edmx();
    assert!(xml.contains("Property=\"TypeName\" String=\"Customer\""));
    assert!(xml.contains("Property=\"TypeNamePlural\" String=\"Customers\""));
}

#[test]
fn customer_common_text_on_key() {
    let xml = build_test_edmx();
    // Customer key → Common.Text pointing to CustomerName
    assert!(xml.contains("Target=\"Service.Customer/ID\""));
    assert!(xml.contains("Term=\"Common.Text\" Path=\"CustomerName\""));
}

#[test]
fn customer_selection_fields() {
    let xml = build_test_edmx();
    assert!(xml.contains("<PropertyPath>CustomerName</PropertyPath>"));
    assert!(xml.contains("<PropertyPath>City</PropertyPath>"));
}

#[test]
fn customer_labels() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"Common.Label\" String=\"Customer Name\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"City\""));
    assert!(xml.contains("Term=\"Common.Label\" String=\"Email\""));
}

#[test]
fn customer_field_group_general() {
    let xml = build_test_edmx();
    // CustomerName and City are in the General field group
    assert!(xml.contains("Property=\"Value\" Path=\"CustomerName\""));
    assert!(xml.contains("Property=\"Value\" Path=\"City\""));
}

#[test]
fn customer_field_group_contact() {
    let xml = build_test_edmx();
    assert!(xml.contains("Term=\"UI.FieldGroup\" Qualifier=\"Contact\""));
    assert!(xml.contains("Property=\"Value\" Path=\"Email\""));
}

#[test]
fn customer_description_in_header() {
    let xml = build_test_edmx();
    // Description → City
    assert!(xml.contains("Property=\"Value\" Path=\"City\""));
}

#[test]
fn customer_line_item_importance() {
    let xml = build_test_edmx();
    // CustomerName has High importance in LineItem
    assert!(xml.contains("UI.ImportanceType/High"));
}

#[test]
fn customer_max_lengths() {
    let xml = build_test_edmx();
    assert!(xml.contains("MaxLength=\"120\""));
    assert!(xml.contains("MaxLength=\"60\""));
    assert!(xml.contains("MaxLength=\"255\""));
}

#[test]
fn customer_draft_actions() {
    let xml = build_test_edmx();
    assert!(xml.contains("Parameter Name=\"in\" Type=\"Service.Customer\""));
}
