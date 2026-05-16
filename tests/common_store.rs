use std::sync::Arc;

use serde_json::{json, Value};
use simple_fiori_server::{
    entities::EntityFacetEntity,
    entity::{ODataEntity, ODataEntityImp},
    runtime::data_store::*,
    spec::{FieldDef, NavigationPropertyDef},
};

// ── EntityKey tests ─────────────────────────────────────────────

#[test]
fn entity_key_single() {
    let key = EntityKey::single("ID", "P001");
    assert_eq!(key.get("ID"), Some("P001"));
    assert_eq!(key.resolve_key_value("ID"), Some("P001"));
    assert!(key.is_active()); // default true
}

#[test]
fn entity_key_composite() {
    let key = EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "true")]);
    assert_eq!(key.get("ID"), Some("O001"));
    assert_eq!(key.get("IsActiveEntity"), Some("true"));
    assert!(key.is_active());
}

#[test]
fn entity_key_composite_inactive() {
    let key = EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "false")]);
    assert!(!key.is_active());
}

#[test]
fn entity_key_parse_simple() {
    let key = EntityKey::parse("'P001'");
    assert_eq!(key.get("_key"), Some("P001"));
    assert_eq!(key.resolve_key_value("ID"), Some("P001"));
}

#[test]
fn entity_key_parse_composite() {
    let key = EntityKey::parse("ID='O001',IsActiveEntity=true");
    assert_eq!(key.get("ID"), Some("O001"));
    assert_eq!(key.get("IsActiveEntity"), Some("true"));
    assert!(key.is_active());
}

#[test]
fn entity_key_parse_composite_with_quotes() {
    let key = EntityKey::parse("ID='P001',IsActiveEntity=false");
    assert_eq!(key.get("ID"), Some("P001"));
    assert!(!key.is_active());
    assert_eq!(key.resolve_key_value("ID"), Some("P001"));
}

#[test]
fn entity_key_missing_field_returns_none() {
    let key = EntityKey::single("ID", "P001");
    assert_eq!(key.get("OrderID"), None);
}

// ── ParentKey tests ─────────────────────────────────────────────

#[test]
fn parent_key_construction() {
    let parent = ParentKey::new("Orders", EntityKey::single("ID", "O001"));
    assert_eq!(parent.set_name, "Orders");
    assert_eq!(parent.key.get("ID"), Some("O001"));
}

// ── ODataQuery tests ────────────────────────────────────────────

#[test]
fn odata_query_empty() {
    let q = ODataQuery::empty();
    assert!(q.filter.is_empty());
    assert!(q.select.is_empty());
    assert!(q.expand.is_empty());
    assert!(q.orderby.is_none());
    assert!(q.top.is_none());
    assert!(q.skip.is_none());
    assert!(!q.count);
}

#[test]
fn odata_query_parse_filter() {
    let q = ODataQuery::parse("$filter=Status%20eq%20'A'");
    assert!(!q.filter.is_empty());
}

#[test]
fn odata_query_parse_select() {
    let q = ODataQuery::parse("$select=ProductID,ProductName,Price");
    assert_eq!(q.select, vec!["ProductID", "ProductName", "Price"]);
}

#[test]
fn odata_query_parse_expand_simple() {
    let q = ODataQuery::parse("$expand=Items");
    assert_eq!(q.expand.len(), 1);
    assert_eq!(q.expand[0].nav_property, "Items");
    assert!(q.expand[0].select.is_empty());
}

#[test]
fn odata_query_parse_expand_with_select() {
    let q = ODataQuery::parse("$expand=DraftAdministrativeData($select=DraftUUID,InProcessByUser)");
    assert_eq!(q.expand.len(), 1);
    assert_eq!(q.expand[0].nav_property, "DraftAdministrativeData");
    assert_eq!(q.expand[0].select, vec!["DraftUUID", "InProcessByUser"]);
}

#[test]
fn odata_query_parse_expand_multiple() {
    let q = ODataQuery::parse("$expand=Items,DraftAdministrativeData($select=DraftUUID)");
    assert_eq!(q.expand.len(), 2);
    assert_eq!(q.expand[0].nav_property, "Items");
    assert_eq!(q.expand[1].nav_property, "DraftAdministrativeData");
}

#[test]
fn odata_query_parse_orderby_asc() {
    let q = ODataQuery::parse("$orderby=Price");
    let ob = q.orderby.unwrap();
    assert_eq!(ob.field, "Price");
    assert!(!ob.descending);
}

#[test]
fn odata_query_parse_orderby_desc() {
    let q = ODataQuery::parse("$orderby=Price%20desc");
    let ob = q.orderby.unwrap();
    assert_eq!(ob.field, "Price");
    assert!(ob.descending);
}

#[test]
fn odata_query_parse_top_skip() {
    let q = ODataQuery::parse("$top=10&$skip=20");
    assert_eq!(q.top, Some(10));
    assert_eq!(q.skip, Some(20));
}

#[test]
fn odata_query_parse_count() {
    let q = ODataQuery::parse("$count=true");
    assert!(q.count);
}

#[test]
fn odata_query_parse_combined() {
    let q = ODataQuery::parse(
            "$filter=Status%20eq%20'A'&$orderby=Price%20desc&$top=5&$skip=0&$count=true&$select=ProductID,Price",
        );

    assert_eq!(q.select, vec!["ProductID", "Price"]);
    let ob = q.orderby.unwrap();
    assert_eq!(ob.field, "Price");
    assert!(ob.descending);
    assert_eq!(q.top, Some(5));
    assert_eq!(q.skip, Some(0));
    assert!(q.count);
}

#[test]
fn odata_query_to_query_map_roundtrip() {
    let q = ODataQuery {
        filter: ODataFilterExpression::new("Status eq 'A'"),
        select: vec!["ProductID".to_string(), "Price".to_string()],
        expand: vec![ExpandClause {
            nav_property: "Items".to_string(),
            select: vec![],
        }],
        orderby: Some(OrderByClause {
            field: "Price".to_string(),
            descending: true,
        }),
        top: Some(10),
        skip: Some(5),
        count: true,
    };
    let map = q.to_query_map();
    assert_eq!(map.get("$filter").unwrap(), "Status eq 'A'");
    assert_eq!(map.get("$select").unwrap(), "ProductID,Price");
    assert_eq!(map.get("$expand").unwrap(), "Items");
    assert_eq!(map.get("$orderby").unwrap(), "Price desc");
    assert_eq!(map.get("$top").unwrap(), "10");
    assert_eq!(map.get("$skip").unwrap(), "5");
    assert_eq!(map.get("$count").unwrap(), "true");
}

#[test]
fn parse_filter() {
    let q = ODataQuery::parse("$filter=Price%20gt%20100");
    assert!(q.filter.try_parse());

    let ast = q.filter.ast();
    assert!(ast.is_some());

    assert_eq!(
        ast,
        Some(odata_params::filters::Expr::Compare(
            Box::new(odata_params::filters::Expr::Identifier("Price".to_string())),
            odata_params::filters::CompareOperator::GreaterThan,
            Box::new(odata_params::filters::Expr::Value(
                odata_params::filters::Value::Number(100.into())
            )),
        ))
    );


    assert_eq!(q.filter.eval(&json!({"Price": 100})), false);
    assert_eq!(q.filter.eval(&json!({"Price": 150})), true);
}

#[test]
fn parse_filter_with_string(){
    let q = ODataQuery::parse("$filter=Status eq 'A'");
    assert!(q.filter.try_parse());

    let ast = q.filter.ast();
    assert!(ast.is_some());

    assert_eq!(
        ast,
        Some(odata_params::filters::Expr::Compare(
            Box::new(odata_params::filters::Expr::Identifier("Status".to_string())),
            odata_params::filters::CompareOperator::Equal,
            Box::new(odata_params::filters::Expr::Value(
                odata_params::filters::Value::String("A".to_string())
            )),
        ))
    );

    assert!(q.filter.eval(&json!({"ID":"P001","Price":"1299.99","ProductName":"Laptop","Status":"A"})));
    assert!(!q.filter.eval(&json!({"ID":"P003","Price":"499.99","ProductName":"Monitor","Status":"D"})));
}

// ── StoreError tests ────────────────────────────────────────────

#[test]
fn store_error_display() {
    let e = StoreError::NotFound("test".to_string());
    assert_eq!(format!("{}", e), "Not found: test");
    let e = StoreError::BadRequest("bad".to_string());
    assert_eq!(format!("{}", e), "Bad request: bad");
}
