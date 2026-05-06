//! Comprehensive CDM 3.1 site.json generation test.
//!
//! Builds sample resolved entities (parent "Order", child "OrderItem", standalone
//! "Customer"), generates the CDM site document via `build_cdm_site_json`, then
//! asserts on every aspect: site envelope, applications, visualizations, pages,
//! menus, and child-entity exclusion.

use fake_fiori_server::builders::build_cdm_site_json;
use fake_fiori_server::model::resolved::*;
use serde_json::Value;

// ══════════════════════════════════════════════════════════════
//  Sample entities (minimal — site.json only needs set_name,
//  type_name_plural, and parent_set_name)
// ══════════════════════════════════════════════════════════════

fn order_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "Orders".into(),
        type_name: "Order".into(),
        type_name_plural: "Orders".into(),
        key_field: "ID".into(),
        title_field: "OrderName".into(),
        description_field: None,
        parent_set_name: None, // root → gets a tile
        properties: vec![],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

fn order_item_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "OrderItems".into(),
        type_name: "OrderItem".into(),
        type_name_plural: "Order Items".into(),
        key_field: "ID".into(),
        title_field: "Description".into(),
        description_field: None,
        parent_set_name: Some("Orders".into()), // child → NO tile
        properties: vec![],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

fn customer_entity() -> ResolvedEntity {
    ResolvedEntity {
        set_name: "Customers".into(),
        type_name: "Customer".into(),
        type_name_plural: "Customers".into(),
        key_field: "ID".into(),
        title_field: "CustomerName".into(),
        description_field: None,
        parent_set_name: None, // root → gets a tile
        properties: vec![],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
        extra_annotations_xml: String::new(),
        custom_actions_xml: String::new(),
    }
}

fn build_test_site() -> Value {
    let entities = vec![order_entity(), order_item_entity(), customer_entity()];
    build_cdm_site_json(&entities)
}

// ══════════════════════════════════════════════════════════════
//  Site envelope
// ══════════════════════════════════════════════════════════════

#[test]
fn site_version() {
    let site = build_test_site();
    assert_eq!(site["_version"], "3.1.0");
}

#[test]
fn site_identification() {
    let site = build_test_site();
    assert_eq!(site["site"]["identification"]["id"], "local-flp-site");
    assert_eq!(
        site["site"]["identification"]["title"],
        "Local Fiori Launchpad"
    );
}

#[test]
fn site_has_all_top_level_keys() {
    let site = build_test_site();
    assert!(site["applications"].is_object());
    assert!(site["visualizations"].is_object());
    assert!(site["vizTypes"].is_object());
    assert!(site["pages"].is_object());
    assert!(site["menus"].is_object());
    assert!(site["systemAliases"].is_object());
}

// ══════════════════════════════════════════════════════════════
//  Child entity exclusion
// ══════════════════════════════════════════════════════════════

#[test]
fn child_entities_are_excluded() {
    let site = build_test_site();
    let apps = site["applications"].as_object().unwrap();
    // OrderItems has parent_set_name → no app entry
    assert!(!apps.contains_key("orderitems.app"));
    // Only Orders + Customers = 2 apps
    assert_eq!(apps.len(), 2);
}

#[test]
fn only_root_entities_get_visualizations() {
    let site = build_test_site();
    let viz = site["visualizations"].as_object().unwrap();
    assert_eq!(viz.len(), 2);
    assert!(viz.contains_key("Orders-display-viz"));
    assert!(viz.contains_key("Customers-display-viz"));
}

// ══════════════════════════════════════════════════════════════
//  Applications — Orders
// ══════════════════════════════════════════════════════════════

#[test]
fn orders_app_exists() {
    let site = build_test_site();
    assert!(site["applications"]["orders.app"].is_object());
}

#[test]
fn orders_app_id() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(app["sap.app"]["id"], "orders.app");
}

#[test]
fn orders_app_title() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(app["sap.app"]["title"], "Orders");
}

#[test]
fn orders_app_cross_navigation_inbound() {
    let site = build_test_site();
    let inbound =
        &site["applications"]["orders.app"]["sap.app"]["crossNavigation"]["inbounds"]
            ["Orders-display"];
    assert_eq!(inbound["semanticObject"], "Orders");
    assert_eq!(inbound["action"], "display");
    assert_eq!(
        inbound["signature"]["additionalParameters"],
        "allowed"
    );
}

#[test]
fn orders_app_component_name() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(app["sap.ui5"]["componentName"], "orders.app");
}

#[test]
fn orders_app_url() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(
        app["sap.platform.runtime"]["componentProperties"]["url"],
        "./apps/Orders/"
    );
}

#[test]
fn orders_app_flp_type() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(app["sap.flp"]["type"], "application");
}

#[test]
fn orders_app_ui_technology() {
    let site = build_test_site();
    let app = &site["applications"]["orders.app"];
    assert_eq!(app["sap.ui"]["technology"], "UI5");
}

#[test]
fn orders_app_device_types() {
    let site = build_test_site();
    let devices = &site["applications"]["orders.app"]["sap.ui"]["deviceTypes"];
    assert_eq!(devices["desktop"], true);
    assert_eq!(devices["tablet"], true);
    assert_eq!(devices["phone"], true);
}

// ══════════════════════════════════════════════════════════════
//  Applications — Customers
// ══════════════════════════════════════════════════════════════

#[test]
fn customers_app_exists() {
    let site = build_test_site();
    assert!(site["applications"]["customers.app"].is_object());
}

#[test]
fn customers_app_title() {
    let site = build_test_site();
    assert_eq!(
        site["applications"]["customers.app"]["sap.app"]["title"],
        "Customers"
    );
}

#[test]
fn customers_app_inbound() {
    let site = build_test_site();
    let inbound =
        &site["applications"]["customers.app"]["sap.app"]["crossNavigation"]["inbounds"]
            ["Customers-display"];
    assert_eq!(inbound["semanticObject"], "Customers");
    assert_eq!(inbound["action"], "display");
}

#[test]
fn customers_app_url() {
    let site = build_test_site();
    assert_eq!(
        site["applications"]["customers.app"]["sap.platform.runtime"]["componentProperties"]
            ["url"],
        "./apps/Customers/"
    );
}

// ══════════════════════════════════════════════════════════════
//  Visualizations — Orders
// ══════════════════════════════════════════════════════════════

#[test]
fn orders_viz_type() {
    let site = build_test_site();
    let viz = &site["visualizations"]["Orders-display-viz"];
    assert_eq!(viz["vizType"], "sap.ushell.StaticAppLauncher");
}

#[test]
fn orders_viz_business_app() {
    let site = build_test_site();
    let viz = &site["visualizations"]["Orders-display-viz"];
    assert_eq!(viz["businessApp"], "orders.app");
}

#[test]
fn orders_viz_target() {
    let site = build_test_site();
    let viz = &site["visualizations"]["Orders-display-viz"];
    assert_eq!(viz["target"]["semanticObject"], "Orders");
    assert_eq!(viz["target"]["action"], "display");
}

#[test]
fn orders_viz_config_flp_target() {
    let site = build_test_site();
    let flp = &site["visualizations"]["Orders-display-viz"]["vizConfig"]["sap.flp"]["target"];
    assert_eq!(flp["appId"], "orders.app");
    assert_eq!(flp["inboundId"], "Orders-display");
}

#[test]
fn orders_viz_config_app_title() {
    let site = build_test_site();
    let app =
        &site["visualizations"]["Orders-display-viz"]["vizConfig"]["sap.app"];
    assert_eq!(app["title"], "Orders");
    assert_eq!(app["icon"], "sap-icon://sys-help");
}

// ══════════════════════════════════════════════════════════════
//  Visualizations — Customers
// ══════════════════════════════════════════════════════════════

#[test]
fn customers_viz_exists() {
    let site = build_test_site();
    assert!(site["visualizations"]["Customers-display-viz"].is_object());
}

#[test]
fn customers_viz_business_app() {
    let site = build_test_site();
    assert_eq!(
        site["visualizations"]["Customers-display-viz"]["businessApp"],
        "customers.app"
    );
}

#[test]
fn customers_viz_config_title() {
    let site = build_test_site();
    assert_eq!(
        site["visualizations"]["Customers-display-viz"]["vizConfig"]["sap.app"]["title"],
        "Customers"
    );
}

// ══════════════════════════════════════════════════════════════
//  Pages — home page structure
// ══════════════════════════════════════════════════════════════

#[test]
fn home_page_exists() {
    let site = build_test_site();
    assert!(site["pages"]["home-page"].is_object());
}

#[test]
fn home_page_identification() {
    let site = build_test_site();
    let page = &site["pages"]["home-page"];
    assert_eq!(page["identification"]["id"], "home-page");
    assert_eq!(page["identification"]["title"], "Home");
}

#[test]
fn home_page_has_apps_section() {
    let site = build_test_site();
    let sections = &site["pages"]["home-page"]["payload"]["sections"];
    assert!(sections["apps-section"].is_object());
}

#[test]
fn apps_section_properties() {
    let site = build_test_site();
    let section = &site["pages"]["home-page"]["payload"]["sections"]["apps-section"];
    assert_eq!(section["id"], "apps-section");
    assert_eq!(section["title"], "Applications");
    assert_eq!(section["default"], true);
    assert_eq!(section["visible"], true);
    assert_eq!(section["preset"], true);
    assert_eq!(section["locked"], false);
}

#[test]
fn apps_section_viz_order_has_root_entities_only() {
    let site = build_test_site();
    let viz_order = site["pages"]["home-page"]["payload"]["sections"]["apps-section"]["layout"]
        ["vizOrder"]
        .as_array()
        .unwrap();
    assert_eq!(viz_order.len(), 2);
    assert!(viz_order.contains(&Value::String("Orders-display-viz".into())));
    assert!(viz_order.contains(&Value::String("Customers-display-viz".into())));
}

#[test]
fn apps_section_viz_refs() {
    let site = build_test_site();
    let viz = &site["pages"]["home-page"]["payload"]["sections"]["apps-section"]["viz"];
    let orders_ref = &viz["Orders-display-viz"];
    assert_eq!(orders_ref["id"], "Orders-display-viz");
    assert_eq!(orders_ref["vizId"], "Orders-display-viz");
    let customers_ref = &viz["Customers-display-viz"];
    assert_eq!(customers_ref["id"], "Customers-display-viz");
    assert_eq!(customers_ref["vizId"], "Customers-display-viz");
}

#[test]
fn section_order() {
    let site = build_test_site();
    let order = site["pages"]["home-page"]["payload"]["layout"]["sectionOrder"]
        .as_array()
        .unwrap();
    assert_eq!(order, &vec![Value::String("apps-section".into())]);
}

// ══════════════════════════════════════════════════════════════
//  Menus
// ══════════════════════════════════════════════════════════════

#[test]
fn main_menu_exists() {
    let site = build_test_site();
    assert!(site["menus"]["main"].is_object());
}

#[test]
fn main_menu_entry() {
    let site = build_test_site();
    let entries = site["menus"]["main"]["payload"]["menuEntries"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry["id"], "home-space-entry");
    assert_eq!(entry["title"], "Home");
    assert_eq!(entry["type"], "IBN");
    assert_eq!(entry["target"]["semanticObject"], "Launchpad");
    assert_eq!(entry["target"]["action"], "openFLPPage");
}

#[test]
fn main_menu_entry_parameters() {
    let site = build_test_site();
    let params = site["menus"]["main"]["payload"]["menuEntries"][0]["target"]["parameters"]
        .as_array()
        .unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0]["name"], "spaceId");
    assert_eq!(params[0]["value"], "home-space");
    assert_eq!(params[1]["name"], "pageId");
    assert_eq!(params[1]["value"], "home-page");
}

// ══════════════════════════════════════════════════════════════
//  Edge cases
// ══════════════════════════════════════════════════════════════

#[test]
fn empty_entities_produces_valid_site() {
    let site = build_cdm_site_json(&[]);
    assert_eq!(site["_version"], "3.1.0");
    assert_eq!(site["applications"].as_object().unwrap().len(), 0);
    assert_eq!(site["visualizations"].as_object().unwrap().len(), 0);
    let viz_order = site["pages"]["home-page"]["payload"]["sections"]["apps-section"]["layout"]
        ["vizOrder"]
        .as_array()
        .unwrap();
    assert!(viz_order.is_empty());
}

#[test]
fn all_children_produces_empty_apps() {
    // When every entity is a child, no tiles are generated
    let entities = vec![order_item_entity()];
    let site = build_cdm_site_json(&entities);
    assert_eq!(site["applications"].as_object().unwrap().len(), 0);
    assert_eq!(site["visualizations"].as_object().unwrap().len(), 0);
}

#[test]
fn viz_types_is_empty_object() {
    let site = build_test_site();
    assert_eq!(site["vizTypes"].as_object().unwrap().len(), 0);
}

#[test]
fn system_aliases_is_empty_object() {
    let site = build_test_site();
    assert_eq!(site["systemAliases"].as_object().unwrap().len(), 0);
}
