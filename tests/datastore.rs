use fake_fiori_server::runtime::data_store::*;
use serde_json::json;

impl InMemoryDataStore {
    fn record_count(&self, set_name: &str) -> usize {
        self.store
            .read()
            .unwrap()
            .get(set_name)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}


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
    assert!(q.filter.is_none());
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
    assert_eq!(q.filter, Some("Status eq 'A'".to_string()));
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
    assert_eq!(q.filter, Some("Status eq 'A'".to_string()));
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
        filter: Some("Status eq 'A'".to_string()),
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

// ── StoreError tests ────────────────────────────────────────────

#[test]
fn store_error_display() {
    let e = StoreError::NotFound("test".to_string());
    assert_eq!(format!("{}", e), "Not found: test");
    let e = StoreError::BadRequest("bad".to_string());
    assert_eq!(format!("{}", e), "Bad request: bad");
}

// ── InMemoryDataStore tests ─────────────────────────────────────


/// Minimal test entity for unit tests.
#[derive(Debug)]
struct TestProductEntity;

impl ODataEntity for TestProductEntity {
    fn set_name(&self) -> &'static str {
        "Products"
    }
    fn type_name(&self) -> &'static str {
        "Product"
    }
    fn mock_data(&self) -> Vec<Value> {
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

impl ODataEntity for TestOrderEntity {
    fn set_name(&self) -> &'static str {
        "Orders"
    }
    fn type_name(&self) -> &'static str {
        "Order"
    }
    fn mock_data(&self) -> Vec<Value> {
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

impl ODataEntity for TestOrderItemEntity {
    fn set_name(&self) -> &'static str {
        "OrderItems"
    }
    fn type_name(&self) -> &'static str {
        "OrderItem"
    }
    fn mock_data(&self) -> Vec<Value> {
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
    let entities: Vec<&'static dyn ODataEntity> = vec![
        &TestProductEntity,
        &TestOrderEntity,
        &TestOrderItemEntity,
        &EntityFacetEntity,
    ];
    InMemoryDataStore::new(data_dir, entities)
}

#[test]
fn store_get_collection_returns_all() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    let result = store.get_collection("Products", &q, None).unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 3);
}

#[test]
fn store_get_collection_with_filter() {
    let store = create_test_store();
    let q = ODataQuery::parse("$filter=Status eq 'A'");
    let result = store.get_collection("Products", &q, None).unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2);
}

#[test]
fn store_get_collection_with_top_skip() {
    let store = create_test_store();
    let q = ODataQuery::parse("$top=1&$skip=1");
    let result = store.get_collection("Products", &q, None).unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 1);
}

#[test]
fn store_get_collection_with_orderby() {
    let store = create_test_store();
    let q = ODataQuery::parse("$orderby=Price desc");
    let result = store.get_collection("Products", &q, None).unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    // Laptop (1299.99) should be first
    assert_eq!(
        values[0].get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );
}

#[test]
fn store_get_collection_with_count() {
    let store = create_test_store();
    let q = ODataQuery::parse("$count=true");
    let result = store.get_collection("Products", &q, None).unwrap();
    assert_eq!(result.get("@odata.count").unwrap().as_i64().unwrap(), 3);
}

#[test]
fn store_get_collection_not_found() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    let result = store.get_collection("NonExistent", &q, None);
    assert!(result.is_err());
}

#[test]
fn store_get_collection_sub_collection() {
    let store = create_test_store();
    let parent = ParentKey::new("Orders", EntityKey::single("ID", "O001"));
    let q = ODataQuery::empty();
    let result = store
        .get_collection("OrderItems", &q, Some(&parent))
        .unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2); // I001 and I002 belong to O001
}

#[test]
fn store_count_all() {
    let store = create_test_store();
    let q = ODataQuery::empty();
    assert_eq!(store.count("Products", &q, None), 3);
}

#[test]
fn store_count_sub_collection() {
    let store = create_test_store();
    let parent = ParentKey::new("Orders", EntityKey::single("ID", "O001"));
    let q = ODataQuery::empty();
    assert_eq!(store.count("OrderItems", &q, Some(&parent)), 2);
}

#[test]
fn store_read_entity() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let q = ODataQuery::empty();
    let result = store.read_entity("Products", &key, &q).unwrap();
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
    let result = store.read_entity("Products", &key, &q);
    assert!(result.is_err());
}

#[test]
fn store_read_entity_parsed_key() {
    let store = create_test_store();
    let key = EntityKey::parse("'P002'");
    let q = ODataQuery::empty();
    let result = store.read_entity("Products", &key, &q).unwrap();
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
    assert_eq!(
        result.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        false
    ); // draft
    assert!(result.get("@odata.context").is_some());

    // New entity is in changeset, baseline count stays the same
    let q = ODataQuery::empty();
    assert_eq!(store.count("Products", &q, None), 3);

    // But the new entity should be readable as a draft
    let new_key_value = result.get("ID").unwrap().as_str().unwrap();
    let draft_key = EntityKey::composite(&[("ID", new_key_value), ("IsActiveEntity", "false")]);
    let draft = store.read_entity("Products", &draft_key, &q).unwrap();
    assert_eq!(
        draft.get("ProductName").unwrap().as_str().unwrap(),
        "Keyboard"
    );
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
    // First create a draft to patch (drafts are editable)
    let key = EntityKey::single("ID", "P001");
    let edit_result = store.draft_edit("Products", &key).unwrap();
    assert_eq!(
        edit_result
            .get("IsActiveEntity")
            .unwrap()
            .as_bool()
            .unwrap(),
        false
    );

    // Patch the draft
    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    let patch = json!({"ProductName": "Laptop Pro Max"});
    let result = store.patch_entity("Products", &draft_key, &patch).unwrap();
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop Pro Max"
    );
}

#[test]
fn store_patch_entity_immutable_field_ignored() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    store.draft_edit("Products", &key).unwrap();

    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    let patch = json!({"ID": "HACKED", "ProductName": "Changed"});
    let result = store.patch_entity("Products", &draft_key, &patch).unwrap();
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
    // Create a draft then delete it
    let key = EntityKey::single("ID", "P001");
    store.draft_edit("Products", &key).unwrap();

    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    store.delete_entity("Products", &draft_key).unwrap();

    // Draft should be gone, active should have HasDraftEntity=false
    let active_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "true")]);
    let q = ODataQuery::empty();
    let active = store.read_entity("Products", &active_key, &q).unwrap();
    assert_eq!(
        active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
        false
    );

    // Draft should not exist
    let draft_read = store.read_entity("Products", &draft_key, &q);
    assert!(draft_read.is_err());
}

#[test]
fn store_delete_entity_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let result = store.delete_entity("Products", &key);
    assert!(result.is_err());
}

// ── Draft lifecycle tests ───────────────────────────────────────

#[test]
fn store_draft_edit_creates_draft() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    let draft = store.draft_edit("Products", &key).unwrap();

    assert_eq!(
        draft.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        false
    );
    assert_eq!(
        draft.get("HasActiveEntity").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        draft.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );

    // Active entity should now have HasDraftEntity=true
    let active_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "true")]);
    let q = ODataQuery::empty();
    let active = store.read_entity("Products", &active_key, &q).unwrap();
    assert_eq!(
        active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
        true
    );
}

#[test]
fn store_draft_edit_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let result = store.draft_edit("Products", &key);
    assert!(result.is_err());
}

#[test]
fn store_draft_activate_updates_active() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");

    let n = store.record_count("Products");
    assert_eq!(n, 3); // ensure initial count

    // Edit → creates changeset entry (baseline unchanged)
    store.draft_edit("Products", &key).unwrap();

    // Baseline count stays the same (draft is in changeset, not store)
    assert_eq!(n, store.record_count("Products"));

    // Patch draft
    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    let patch = json!({"ProductName": "Laptop Pro 16"});
    store.patch_entity("Products", &draft_key, &patch).unwrap();

    // Activate — merges changeset into baseline
    let activated = store.draft_activate("Products", &key).unwrap();
    assert_eq!(
        activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        activated.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop Pro 16"
    );
    assert_eq!(
        activated.get("HasDraftEntity").unwrap().as_bool().unwrap(),
        false
    );

    assert_eq!(n, store.record_count("Products"));

    // Draft should be gone (no changeset)
    let q = ODataQuery::empty();
    let draft_read = store.read_entity("Products", &draft_key, &q);
    assert!(draft_read.is_err());
}

#[test]
fn store_draft_activate_new_entity() {
    let store = create_test_store();
    // Create a brand new entity (no active counterpart)
    let data = json!({"ProductName": "New Product", "Price": "9.99"});
    let created = store.create_entity("Products", &data, None).unwrap();
    let new_key_value = created.get("ID").unwrap().as_str().unwrap();

    let new_key = EntityKey::single("ID", new_key_value);
    let activated = store.draft_activate("Products", &new_key).unwrap();
    assert_eq!(
        activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        activated.get("ProductName").unwrap().as_str().unwrap(),
        "New Product"
    );
}

#[test]
fn store_draft_prepare() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P001");
    store.draft_edit("Products", &key).unwrap();

    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    let result = store.draft_prepare("Products", &draft_key).unwrap();
    assert_eq!(
        result.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );
    assert!(result.get("@odata.context").is_some());
}

#[test]
fn store_draft_prepare_not_found() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "P999");
    let result = store.draft_prepare("Products", &key);
    assert!(result.is_err());
}

// ── Draft with children ─────────────────────────────────────────

#[test]
fn store_draft_edit_copies_children() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "O001");
    store.draft_edit("Orders", &key).unwrap();

    // Check that child drafts were created
    let parent = ParentKey::new(
        "Orders",
        EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "false")]),
    );
    let q = ODataQuery::empty();
    let children = store
        .get_collection("OrderItems", &q, Some(&parent))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2); // I001 and I002 as drafts
    for v in values {
        assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), false);
    }
}

#[test]
fn store_draft_activate_activates_children() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "O001");
    store.draft_edit("Orders", &key).unwrap();

    // Activate parent
    store.draft_activate("Orders", &key).unwrap();

    // Children should be active again
    let parent = ParentKey::new(
        "Orders",
        EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "true")]),
    );
    let q = ODataQuery::empty();
    let children = store
        .get_collection("OrderItems", &q, Some(&parent))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2);
    for v in values {
        assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), true);
    }
}

#[test]
fn store_delete_draft_removes_children() {
    let store = create_test_store();
    let key = EntityKey::single("ID", "O001");
    store.draft_edit("Orders", &key).unwrap();

    // Delete draft (discard)
    let draft_key = EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "false")]);
    store.delete_entity("Orders", &draft_key).unwrap();

    // Draft children should be gone
    let parent_draft = ParentKey::new("Orders", draft_key.clone());
    let q = ODataQuery::empty();
    let children = store
        .get_collection("OrderItems", &q, Some(&parent_draft))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 0);

    // Active children should still be there
    let parent_active = ParentKey::new(
        "Orders",
        EntityKey::composite(&[("ID", "O001"), ("IsActiveEntity", "true")]),
    );
    let active_children = store
        .get_collection("OrderItems", &q, Some(&parent_active))
        .unwrap();
    let active_values = active_children.get("value").unwrap().as_array().unwrap();
    assert_eq!(active_values.len(), 2);
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

    let entities: Vec<&'static dyn ODataEntity> = vec![&TestProductEntity];
    let store = InMemoryDataStore::new(tmp_dir.clone(), entities);

    // Patch a product via draft lifecycle
    let key = EntityKey::single("ID", "P001");
    store.draft_edit("Products", &key).unwrap();
    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    store
        .patch_entity("Products", &draft_key, &json!({"ProductName": "Laptop V2"}))
        .unwrap();
    store.draft_activate("Products", &key).unwrap();

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

// ── Full lifecycle integration test ─────────────────────────────

#[test]
fn store_full_draft_lifecycle() {
    let store = create_test_store();
    let q = ODataQuery::empty();

    // 1. Read active entity
    let key = EntityKey::single("ID", "P001");
    let active = store.read_entity("Products", &key, &q).unwrap();
    assert_eq!(
        active.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop"
    );

    // 2. Draft edit
    let draft = store.draft_edit("Products", &key).unwrap();
    assert_eq!(
        draft.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        false
    );

    // 3. Patch draft
    let draft_key = EntityKey::composite(&[("ID", "P001"), ("IsActiveEntity", "false")]);
    store
        .patch_entity(
            "Products",
            &draft_key,
            &json!({"ProductName": "Laptop 2026"}),
        )
        .unwrap();

    // 4. Draft prepare
    let prepared = store.draft_prepare("Products", &draft_key).unwrap();
    assert_eq!(
        prepared.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop 2026"
    );

    // 5. Activate
    let activated = store.draft_activate("Products", &key).unwrap();
    assert_eq!(
        activated.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop 2026"
    );
    assert_eq!(
        activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        true
    );

    // 6. Verify active entity is updated
    let final_read = store.read_entity("Products", &key, &q).unwrap();
    assert_eq!(
        final_read.get("ProductName").unwrap().as_str().unwrap(),
        "Laptop 2026"
    );

    // 7. Verify no draft remains
    let draft_read = store.read_entity("Products", &draft_key, &q);
    assert!(draft_read.is_err());
}

#[test]
fn store_draft_discard_lifecycle() {
    let store = create_test_store();
    let q = ODataQuery::empty();

    // 1. Draft edit
    let key = EntityKey::single("ID", "P002");
    store.draft_edit("Products", &key).unwrap();

    // 2. Patch draft
    let draft_key = EntityKey::composite(&[("ID", "P002"), ("IsActiveEntity", "false")]);
    store
        .patch_entity("Products", &draft_key, &json!({"ProductName": "Changed"}))
        .unwrap();

    // 3. Discard (delete draft)
    store.delete_entity("Products", &draft_key).unwrap();

    // 4. Active should be unchanged
    let active = store.read_entity("Products", &key, &q).unwrap();
    assert_eq!(
        active.get("ProductName").unwrap().as_str().unwrap(),
        "Mouse"
    );
    assert_eq!(
        active.get("HasDraftEntity").unwrap().as_bool().unwrap(),
        false
    );
}

// ── FieldValueList draft tests (custom FK: ListID) ──────────────

#[derive(Debug)]
struct TestValueListEntity;

impl ODataEntity for TestValueListEntity {
    fn set_name(&self) -> &'static str {
        "FieldValueLists"
    }
    fn type_name(&self) -> &'static str {
        "FieldValueList"
    }
    fn mock_data(&self) -> Vec<Value> {
        vec![
            json!({"ID": "VL-001", "ListName": "EdmTypes", "Description": "OData EDM Datentypen"}),
            json!({"ID": "VL-002", "ListName": "StatusCodes", "Description": "Status"}),
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
                name: "ListName",
                label: "Listenname",
                edm_type: "Edm.String",
                max_length: Some(40),
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
                name: "Description",
                label: "Beschreibung",
                edm_type: "Edm.String",
                max_length: Some(120),
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
    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[NavigationPropertyDef {
            name: "Items",
            target_type: "FieldValueListItem",
            is_collection: true,
            foreign_key: Some("ListID"),
        }];
        NAV
    }
}

#[derive(Debug)]
struct TestValueListItemEntity;

impl ODataEntity for TestValueListItemEntity {
    fn set_name(&self) -> &'static str {
        "FieldValueListItems"
    }
    fn type_name(&self) -> &'static str {
        "FieldValueListItem"
    }
    fn mock_data(&self) -> Vec<Value> {
        vec![
            json!({"ID": "ITEM-001", "ListID": "VL-001", "Code": "Edm.String",  "Description": "Zeichenkette", "SortOrder": 0}),
            json!({"ID": "ITEM-002", "ListID": "VL-001", "Code": "Edm.Int32",   "Description": "Ganzzahl",     "SortOrder": 1}),
            json!({"ID": "ITEM-003", "ListID": "VL-002", "Code": "Active",      "Description": "Aktiv",        "SortOrder": 0}),
        ]
    }
    fn entity_set(&self) -> String {
        String::new()
    }
    fn parent_set_name(&self) -> Option<&'static str> {
        Some("FieldValueLists")
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
                name: "ListID",
                label: "Listen-ID",
                edm_type: "Edm.Guid",
                max_length: None,
                precision: None,
                scale: None,
                immutable: true,
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
                name: "Code",
                label: "Code",
                edm_type: "Edm.String",
                max_length: Some(40),
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
                name: "Description",
                label: "Beschreibung",
                edm_type: "Edm.String",
                max_length: Some(120),
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
                name: "SortOrder",
                label: "Reihenfolge",
                edm_type: "Edm.Int32",
                max_length: None,
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

fn create_vl_store() -> InMemoryDataStore {
    let data_dir = std::env::temp_dir().join("fiori-test-vl-nonexistent");
    let entities: Vec<&'static dyn ODataEntity> =
        vec![&TestValueListEntity, &TestValueListItemEntity];
    InMemoryDataStore::new(data_dir, entities)
}

#[test]
fn vl_read_items_via_parent() {
    let store = create_vl_store();
    let parent = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
    );
    let q = ODataQuery::empty();
    let result = store
        .get_collection("FieldValueListItems", &q, Some(&parent))
        .unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2); // ITEM-001 and ITEM-002 belong to VL-001
}

#[test]
fn vl_read_items_other_parent() {
    let store = create_vl_store();
    let parent = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-002"), ("IsActiveEntity", "true")]),
    );
    let q = ODataQuery::empty();
    let result = store
        .get_collection("FieldValueListItems", &q, Some(&parent))
        .unwrap();
    let values = result.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 1); // ITEM-003 belongs to VL-002
}

#[test]
fn vl_draft_edit_copies_children_with_custom_fk() {
    let store = create_vl_store();
    let key = EntityKey::single("ID", "VL-001");
    store.draft_edit("FieldValueLists", &key).unwrap();

    // Draft children should exist, filtered by ListID (not ID)
    let parent_draft = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
    );
    let q = ODataQuery::empty();
    let children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_draft))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 2);
    for v in values {
        assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), false);
        assert_eq!(v.get("ListID").unwrap().as_str().unwrap(), "VL-001");
    }
}

#[test]
fn vl_create_item_sets_list_id_not_id() {
    let store = create_vl_store();
    let key = EntityKey::single("ID", "VL-001");
    store.draft_edit("FieldValueLists", &key).unwrap();

    // Create a new child item via sub-collection POST
    let parent = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
    );
    let data = json!({"Code": "Edm.Boolean", "Description": "Wahrheitswert", "SortOrder": 2});
    let created = store
        .create_entity("FieldValueListItems", &data, Some(&parent))
        .unwrap();

    // ListID should be set to the parent's key (VL-001)
    assert_eq!(created.get("ListID").unwrap().as_str().unwrap(), "VL-001");
    // ID should be auto-generated and NOT be the parent's key
    let item_id = created.get("ID").unwrap().as_str().unwrap();
    assert_ne!(
        item_id, "VL-001",
        "Child ID must not be overwritten with parent key"
    );
    assert!(
        uuid::Uuid::parse_str(item_id).is_ok(),
        "Edm.Guid key should be a valid UUID, got: {}",
        item_id
    );
}

#[test]
fn vl_create_item_visible_in_subcollection() {
    let store = create_vl_store();
    let key = EntityKey::single("ID", "VL-001");
    store.draft_edit("FieldValueLists", &key).unwrap();

    let parent_draft = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
    );
    let data = json!({"Code": "Edm.Date", "Description": "Datum", "SortOrder": 3});
    store
        .create_entity("FieldValueListItems", &data, Some(&parent_draft))
        .unwrap();

    // Should now have 3 draft items (2 copied + 1 new)
    let q = ODataQuery::empty();
    let children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_draft))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 3);
}

#[test]
fn vl_patch_child_item() {
    let store = create_vl_store();
    let key = EntityKey::single("ID", "VL-001");
    store.draft_edit("FieldValueLists", &key).unwrap();

    // Patch the first draft child
    let draft_item_key = EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "false")]);
    let patch = json!({"Description": "Zeichenkette (aktualisiert)"});
    let result = store
        .patch_entity("FieldValueListItems", &draft_item_key, &patch)
        .unwrap();
    assert_eq!(
        result.get("Description").unwrap().as_str().unwrap(),
        "Zeichenkette (aktualisiert)"
    );
    // ListID should remain unchanged
    assert_eq!(result.get("ListID").unwrap().as_str().unwrap(), "VL-001");
}

#[test]
fn vl_activate_with_new_child() {
    let store = create_vl_store();
    let q = ODataQuery::empty();
    let key = EntityKey::single("ID", "VL-001");

    // 1. Edit → draft
    store.draft_edit("FieldValueLists", &key).unwrap();

    // 2. Create new child
    let parent_draft = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
    );
    let data = json!({"Code": "Edm.Guid", "Description": "GUID", "SortOrder": 9});
    store
        .create_entity("FieldValueListItems", &data, Some(&parent_draft))
        .unwrap();

    // 3. Activate parent
    let activated = store.draft_activate("FieldValueLists", &key).unwrap();
    assert_eq!(
        activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        true
    );

    // 4. Active children should include the new item
    let parent_active = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
    );
    let children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_active))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 3); // 2 original + 1 new
    for v in values {
        assert_eq!(v.get("IsActiveEntity").unwrap().as_bool().unwrap(), true);
        assert_eq!(v.get("ListID").unwrap().as_str().unwrap(), "VL-001");
    }

    // 5. No draft children remain
    let children_draft = store
        .get_collection("FieldValueListItems", &q, Some(&parent_draft))
        .unwrap();
    let draft_values = children_draft.get("value").unwrap().as_array().unwrap();
    assert_eq!(draft_values.len(), 0);
}

#[test]
fn vl_activate_with_patched_child() {
    let store = create_vl_store();
    let q = ODataQuery::empty();
    let key = EntityKey::single("ID", "VL-001");

    // Edit, patch child, activate
    store.draft_edit("FieldValueLists", &key).unwrap();
    let draft_item_key = EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "false")]);
    store
        .patch_entity(
            "FieldValueListItems",
            &draft_item_key,
            &json!({"Description": "String (updated)"}),
        )
        .unwrap();
    store.draft_activate("FieldValueLists", &key).unwrap();

    // Read active child
    let active_item_key = EntityKey::composite(&[("ID", "ITEM-001"), ("IsActiveEntity", "true")]);
    let item = store
        .read_entity("FieldValueListItems", &active_item_key, &q)
        .unwrap();
    assert_eq!(
        item.get("Description").unwrap().as_str().unwrap(),
        "String (updated)"
    );
}

#[test]
fn vl_discard_draft_removes_children() {
    let store = create_vl_store();
    let q = ODataQuery::empty();
    let key = EntityKey::single("ID", "VL-001");

    // Edit → add new item → discard
    store.draft_edit("FieldValueLists", &key).unwrap();
    let parent_draft = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]),
    );
    store
        .create_entity(
            "FieldValueListItems",
            &json!({"Code": "Edm.Byte", "Description": "Byte", "SortOrder": 10}),
            Some(&parent_draft),
        )
        .unwrap();

    // Discard draft
    let draft_key = EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "false")]);
    store.delete_entity("FieldValueLists", &draft_key).unwrap();

    // Draft children gone
    let children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_draft))
        .unwrap();
    assert_eq!(children.get("value").unwrap().as_array().unwrap().len(), 0);

    // Active children unchanged
    let parent_active = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-001"), ("IsActiveEntity", "true")]),
    );
    let active = store
        .get_collection("FieldValueListItems", &q, Some(&parent_active))
        .unwrap();
    assert_eq!(active.get("value").unwrap().as_array().unwrap().len(), 2);
}

#[test]
fn vl_other_list_unaffected_by_draft() {
    let store = create_vl_store();
    let q = ODataQuery::empty();
    let key = EntityKey::single("ID", "VL-001");

    // Edit VL-001 (EdmTypes) → should NOT create drafts for VL-002's children
    store.draft_edit("FieldValueLists", &key).unwrap();

    // VL-002's active items remain unchanged
    let parent_vl2 = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", "VL-002"), ("IsActiveEntity", "true")]),
    );
    let children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_vl2))
        .unwrap();
    let values = children.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].get("Code").unwrap().as_str().unwrap(), "Active");
}

#[test]
fn vl_full_lifecycle_create_list_add_items_activate() {
    let store = create_vl_store();
    let q = ODataQuery::empty();

    // 1. Create a brand new FieldValueList
    let list_data = json!({"ListName": "Priorities", "Description": "Prioritaeten"});
    let created = store
        .create_entity("FieldValueLists", &list_data, None)
        .unwrap();
    let new_list_id = created.get("ID").unwrap().as_str().unwrap().to_string();
    assert_eq!(
        created.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        false
    );

    // 2. Add items to it
    let parent = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", &new_list_id), ("IsActiveEntity", "false")]),
    );
    store
        .create_entity(
            "FieldValueListItems",
            &json!({"Code": "HIGH", "Description": "Hoch", "SortOrder": 0}),
            Some(&parent),
        )
        .unwrap();
    store
        .create_entity(
            "FieldValueListItems",
            &json!({"Code": "MED",  "Description": "Mittel", "SortOrder": 1}),
            Some(&parent),
        )
        .unwrap();
    store
        .create_entity(
            "FieldValueListItems",
            &json!({"Code": "LOW",  "Description": "Niedrig", "SortOrder": 2}),
            Some(&parent),
        )
        .unwrap();

    // 3. Verify draft items
    let draft_children = store
        .get_collection("FieldValueListItems", &q, Some(&parent))
        .unwrap();
    assert_eq!(
        draft_children
            .get("value")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );

    // 4. Activate the list
    let new_key = EntityKey::single("ID", &new_list_id);
    let activated = store.draft_activate("FieldValueLists", &new_key).unwrap();
    assert_eq!(
        activated.get("IsActiveEntity").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        activated.get("ListName").unwrap().as_str().unwrap(),
        "Priorities"
    );

    // 5. Active children should all be there
    let parent_active = ParentKey::new(
        "FieldValueLists",
        EntityKey::composite(&[("ID", &new_list_id), ("IsActiveEntity", "true")]),
    );
    let active_children = store
        .get_collection("FieldValueListItems", &q, Some(&parent_active))
        .unwrap();
    let items = active_children.get("value").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 3);
    let codes: Vec<&str> = items
        .iter()
        .map(|v| v.get("Code").unwrap().as_str().unwrap())
        .collect();
    assert!(codes.contains(&"HIGH"));
    assert!(codes.contains(&"MED"));
    assert!(codes.contains(&"LOW"));
}

#[test]
fn retrieve_facettes_without_duplicates() {
    let store = create_test_store();
    for x in store.entities.try_read().unwrap().iter() {
        println!("Entity: {}", x.set_name());
    }

    store.create_entity("EntityFacets", &json!({"ConfigID": "4553b09f-ab02-4fc9-9653-0dbf32d4cda4", "FieldGroupLabel": "Label", "FieldGroupQualifier": "Qualifier", "ID": "ID"}), None).unwrap();
    let mut q = ODataQuery::empty();
    q.count = true;
    q.orderby = Some(OrderByClause {
        field: "FieldGroupQualifier".to_string(),
        descending: false,
    });
    q.filter = Some("ConfigID eq 4553b09f-ab02-4fc9-9653-0dbf32d4cda4".into());
    q.skip = Some(0);
    q.top = Some(100);
    let col = store.get_collection("EntityFacets", &q, None).unwrap();
    let values = col.get("value").unwrap().as_array().unwrap();
    assert_eq!(values.len(), 1); // Only one facet expected for the given ConfigID
}
