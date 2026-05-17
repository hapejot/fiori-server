use std::sync::Arc;

use simple_fiori_server::{
    entities::EntityFacetEntity,
    entity::{ODataEntity, ODataEntityImp},
    runtime::data_store::*,
    spec::{FieldDef, NavigationPropertyDef},
};
use serde_json::{json, Value};


// ── InMemoryDataStore tests ─────────────────────────────────────

/// Minimal test entity for unit tests.
#[derive(Debug)]
struct TestProductEntity;

impl ODataEntityImp for TestProductEntity {
    fn set_name(&self) -> &'static str {
        "Products"
    }
    fn type_name(&self) -> &'static str {
        "Product"
    }
    fn initial_data(&self) -> Vec<Value> {
        vec![
            json!({"ID": "P001", "ProductName": "Laptop", "Price": "1299.99", "Status": "A"}),
            json!({"ID": "P002", "ProductName": "Mouse", "Price": "29.99", "Status": "A"}),
            json!({"ID": "P003", "ProductName": "Monitor", "Price": "499.99", "Status": "D"}),
        ]
    }
    fn entity_set(&self) -> String {
        String::new()
    }
    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef {
                name: "ID",
                label: "ID",
                edm_type: "Edm.Guid",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
                computed: true,
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
                name: "ProductName",
                label: "Name",
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
                searchable: false,
                show_in_list: false,
                list_sort_order: None,
                list_importance: None,
                list_criticality_path: None,
                form_group: None,
            },
            FieldDef {
                name: "Price",
                label: "Price",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(10),
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
                form_group: None,
            },
            FieldDef {
                name: "Status",
                label: "Status",
                edm_type: "Edm.String",
                max_length: Some(1),
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
        Some(FIELDS)
    }
}

#[derive(Debug)]
struct TestOrderEntity;

impl ODataEntityImp for TestOrderEntity {
    fn set_name(&self) -> &'static str {
        "Orders"
    }
    fn type_name(&self) -> &'static str {
        "Order"
    }
    fn initial_data(&self) -> Vec<Value> {
        vec![
            json!({"ID": "O001", "CustomerName": "Alice", "TotalAmount": "100.00"}),
            json!({"ID": "O002", "CustomerName": "Bob", "TotalAmount": "200.00"}),
        ]
    }
    fn entity_set(&self) -> String {
        String::new()
    }
    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[NavigationPropertyDef {
            name: "Items",
            target_type: "OrderItem",
            is_collection: true,
            foreign_key: Some("OrderID"),
        }];
        NAV
    }
}

#[derive(Debug)]
struct TestOrderItemEntity;

impl ODataEntityImp for TestOrderItemEntity {
    fn set_name(&self) -> &'static str {
        "OrderItems"
    }
    fn type_name(&self) -> &'static str {
        "OrderItem"
    }
    fn initial_data(&self) -> Vec<Value> {
        vec![
            json!({"ID": "I001", "OrderID": "O001", "ProductID": "P001", "Quantity": 2}),
            json!({"ID": "I002", "OrderID": "O001", "ProductID": "P002", "Quantity": 5}),
            json!({"ID": "I003", "OrderID": "O002", "ProductID": "P001", "Quantity": 1}),
        ]
    }
    fn entity_set(&self) -> String {
        String::new()
    }
    fn parent_set_name(&self) -> Option<&'static str> {
        Some("Orders")
    }
}

fn create_test_store() -> InMemoryDataStore {
    // Use a temp dir that doesn't exist so it falls back to mock_data
    let data_dir = std::env::temp_dir().join("fiori-test-nonexistent");
    let entities: Vec<ODataEntity> = vec![
        ODataEntity::new(Arc::new(TestProductEntity)),
        ODataEntity::new(Arc::new(TestOrderEntity)),
        ODataEntity::new(Arc::new(TestOrderItemEntity)),
        ODataEntity::new(Arc::new(EntityFacetEntity)),
    ];
    InMemoryDataStore::new(data_dir, entities)
}

#[test]
fn store_get_collection_returns_all() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    let result = store.get_collection("Products", &q, None, None).unwrap();
    let values = result;
    assert_eq!(values.len(), 3);
}

#[test]
fn store_get_collection_with_filter() {
    let store = create_test_store();
    let q = ODataQuery::parse("$filter=Status eq 'A'");
    let result = store.get_collection("Products", &q, None, None).unwrap();
    let values = result;
    assert_eq!(values.len(), 2);
}

#[test]
fn store_get_collection_with_top_skip() {
    let store = create_test_store();
    let q = ODataQuery::parse("$top=1&$skip=1");
    let result: Vec<Value> = store.get_collection("Products", &q, None,  None).unwrap();
    let values = result;
    assert_eq!(values.len(), 1);
}

#[test]
fn store_get_collection_with_orderby() {
    let store = create_test_store();
    let q = ODataQuery::parse("$orderby=Price desc");
    let result = store.get_collection("Products", &q, None, None).unwrap();
    let values = result;
    // Laptop (1299.99) should be first
    assert_eq!(
        values[0].get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );
}

#[test]
fn store_get_collection_with_count() {
    // lets redesign the interface. the count can be calculated at the top level.

    // let store = create_test_store();
    // let q = ODataQuery::parse("$count=true");
    // let result = store.get_collection("Products", &q, None, None).unwrap();
    // assert_eq!(result.get("@odata.count").unwrap().as_i64().unwrap(), 3);
}

#[test]
fn store_get_collection_not_found() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    let result = store.get_collection("NonExistent", &q, None, None);
    assert!(result.is_err());
}

#[test]
fn store_get_collection_sub_collection() {
    // implementation is unclear to me: How is the parent identified? Via filter on nav property? 

    // let store = create_test_store();
    // let parent = ParentKey::new("Orders", EntityKey::single("ID", "O001"));
    // let q = ODataQuery::empty();
    // let result = store
    //     .get_collection("OrderItems", &q, Some(&parent))
    //     .unwrap();
    // let values = result.get("value").unwrap().as_array().unwrap();
    // assert_eq!(values.len(), 2); // I001 and I002 belong to O001
}

#[test]
fn store_count_all() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    assert_eq!(store.count("Products", &q, None), 3);
}

#[test]
fn store_count_sub_collection() {
    // we can't count what we have not exactly specified.

    // let store = create_test_store();
    // let parent = ParentKey::new("Orders", EntityKey::single("ID", "O001"));
    // let q = ODataQuery::empty();
    // assert_eq!(store.count("OrderItems", &q, Some(&parent)), 2);
}

#[test]
fn store_read_entity() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let q = ODataQuery::empty();
    let result = store.read_record("Products", &key, &q).unwrap();
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );
    assert!(result.get("@odata.context").is_some());
}

#[test]
fn store_read_entity_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let q = ODataQuery::empty();
    let result = store.read_record("Products", &key, &q);
    assert!(result.is_err());
}

#[test]
fn store_read_entity_parsed_key() {
    let store = create_test_store();
    let key = EntityKey::parse("'P002'");
    let q = ODataQuery::empty();
    let result = store.read_record("Products", &key, &q).unwrap();
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Mouse"
    );
}

#[test]
fn store_create_entity() {
    let store = create_test_store();
    let data = json!({"ProductName": "Keyboard", "Price": "79.99", "Status": "A"});
    let result = store.create_entity("Products", &data, None).unwrap();
    assert!(result.get("ID").is_some()); // auto-generated
    assert!(result.get("@odata.context").is_some());


}

#[test]
fn store_create_sub_item() {
    let store = create_test_store();
    let parent = ParentKey::new("Orders", EntityKey::single("ID", "O002"));
    let data = json!({"ProductID": "P003", "Quantity": 3});
    let result = store
        .create_entity("OrderItems", &data, Some(&parent))
        .unwrap();
    assert_eq!(result.get("OrderID").unwrap().as_str().unwrap(), "O002");
    assert!(result.get("ID").is_some());
}

#[test]
fn store_patch_entity() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let patch = json!({"ProductName": "Laptop Pro Max"});
    let result = store.patch_entity("Products", &key, &patch).unwrap();
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop Pro Max"
    );
}

#[test]
fn store_patch_entity_immutable_field_ignored() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let patch = json!({"ID": "HACKED", "ProductName": "Changed"});
    let result = store.patch_entity("Products", &key, &patch).unwrap();
    // ID is computed, should not change
    assert_eq!(result.get("ID").unwrap().as_str().unwrap(), "P001");
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Changed"
    );
}

#[test]
fn store_patch_entity_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let patch = json!({"ProductName": "X"});
    let result = store.patch_entity("Products", &key, &patch);
    assert!(result.is_err());
}

#[test]
fn store_delete_entity() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    store.delete_entity("Products", &key).unwrap();

    let q = ODataQuery::empty();
    let active = store.read_record("Products", &key, &q);
    assert!(active.is_err()); // deleted record should not be found
}

#[test]
fn store_delete_entity_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let result = store.delete_entity("Products", &key);
    assert!(result.is_err());
}


// ── Property access ─────────────────────────────────────────────

#[test]
fn store_get_property() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let val = store.get_property("Products", &key, "ProductName").unwrap();
    assert_eq!(val.as_str().unwrap(), "Laptop");
}

#[test]
fn store_get_property_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let result = store.get_property("Products", &key, "NonExistentField");
    assert!(result.is_err());
}

// ── get_records ─────────────────────────────────────────────────

#[test]
fn store_get_records() {
    let store = create_test_store();
    let records = store.get_records("Products");
    assert_eq!(records.len(), 3);
}

#[test]
fn store_get_records_nonexistent() {
    let store = create_test_store();
    let records = store.get_records("NonExistent");
    assert!(records.is_empty());
}

// ── Commit ──────────────────────────────────────────────────────

#[test]
fn store_commit_writes_json_files() {
    let tmp_dir = std::env::temp_dir().join(format!("fiori-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let entities: Vec<ODataEntity> = vec![ODataEntity::new(Arc::new(TestProductEntity))];
    let store = InMemoryDataStore::new(tmp_dir.clone(), entities);

    let key = EntityKey::single("ID", "P001");
    store
        .patch_entity("Products", &key, &json!({"ProductName": "Laptop V2"}))
        .unwrap();

    // Commit
    store.commit();

    // Verify JSON file
    let json_path = tmp_dir.join("Products.json");
    let content = std::fs::read_to_string(&json_path).unwrap();
    let records: Vec<Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(records.len(), 3);

    // Baseline records don't have draft flags (they were never stored)
    for r in &records {
        assert!(r.get("IsActiveEntity").is_none());
        assert!(r.get("HasActiveEntity").is_none());
        assert!(r.get("HasDraftEntity").is_none());
    }

    // Verify update was saved
    let laptop = records
        .iter()
        .find(|r| r.get("ID").unwrap().as_str() == Some("P001"))
        .unwrap();
    assert_eq!(
        laptop.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop V2"
    );

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();
}

