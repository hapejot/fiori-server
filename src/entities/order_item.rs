use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct OrderItemEntity;

impl ODataEntity for OrderItemEntity {
    fn set_name(&self) -> &'static str {
        "OrderItems"
    }
    fn key_field(&self) -> &'static str {
        "ItemID"
    }
    fn type_name(&self) -> &'static str {
        "OrderItem"
    }

    /// Eltern-EntitySet fuer Composition (Order → OrderItems).
    fn parent_set_name(&self) -> Option<&'static str> {
        Some("Orders")
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![
            // Order O001
            json!({"ItemID": "I001", "OrderID": "O001", "ProductID": "P001", "ProductName": "Laptop Pro 15",
                   "Quantity": 2, "UnitPrice": "1299.99", "Currency": "EUR", "NetAmount": "2599.98"}),
            json!({"ItemID": "I002", "OrderID": "O001", "ProductID": "P002", "ProductName": "Wireless Mouse",
                   "Quantity": 2, "UnitPrice": "29.95", "Currency": "EUR", "NetAmount": "59.90"}),
            // Order O002
            json!({"ItemID": "I003", "OrderID": "O002", "ProductID": "P002", "ProductName": "Wireless Mouse",
                   "Quantity": 10, "UnitPrice": "29.95", "Currency": "EUR", "NetAmount": "299.50"}),
            // Order O003
            json!({"ItemID": "I004", "OrderID": "O003", "ProductID": "P004", "ProductName": "4K Monitor 27\"",
                   "Quantity": 2, "UnitPrice": "549.00", "Currency": "EUR", "NetAmount": "1098.00"}),
            // Order O004
            json!({"ItemID": "I005", "OrderID": "O004", "ProductID": "P003", "ProductName": "USB-C Hub 7-Port",
                   "Quantity": 5, "UnitPrice": "49.90", "Currency": "EUR", "NetAmount": "249.50"}),
            // Order O005
            json!({"ItemID": "I006", "OrderID": "O005", "ProductID": "P005", "ProductName": "Desk Lamp LED",
                   "Quantity": 1, "UnitPrice": "39.99", "Currency": "EUR", "NetAmount": "39.99"}),
        ]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ItemID",      label: "Pos.-Nr.",      edm_type: "Edm.String",  max_length: Some(10),  precision: None,      scale: None,    immutable: true,  semantic_object: None },
            FieldDef { name: "OrderID",     label: "Bestell-Nr.",   edm_type: "Edm.String",  max_length: Some(10),  precision: None,      scale: None,    immutable: true,  semantic_object: None },
            FieldDef { name: "ProductID",   label: "Produkt-ID",    edm_type: "Edm.String",  max_length: Some(10),  precision: None,      scale: None,    immutable: false,  semantic_object: Some("Products") },
            FieldDef { name: "ProductName", label: "Produktname",   edm_type: "Edm.String",  max_length: Some(80),  precision: None,      scale: None,    immutable: false,  semantic_object: None },
            FieldDef { name: "Quantity",    label: "Menge",         edm_type: "Edm.Int32",   max_length: None,      precision: None,      scale: None,    immutable: false, semantic_object: None },
            FieldDef { name: "UnitPrice",   label: "Einzelpreis",   edm_type: "Edm.Decimal", max_length: None,      precision: Some(15),  scale: Some(2), immutable: false, semantic_object: None },
            FieldDef { name: "Currency",    label: "Waehrung",      edm_type: "Edm.String",  max_length: Some(3),   precision: None,      scale: None,    immutable: false, semantic_object: None },
            FieldDef { name: "NetAmount",   label: "Nettobetrag",   edm_type: "Edm.Decimal", max_length: None,      precision: Some(15),  scale: Some(2), immutable: false, semantic_object: None },
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef { name: "Product", target_type: "Product", is_collection: false },
        ];
        NAV
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"OrderItems\" EntityType=\"{ns}.OrderItem\">\n\
             <NavigationPropertyBinding Path=\"Product\" Target=\"Products\"/>\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"OrderItems\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &[],
            line_item: &[
                LineItemField { name: "ItemID",      importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "ProductID",   importance: None, criticality_path: None, navigation_path: None, semantic_object: Some("Products") },
                LineItemField { name: "ProductName", importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Quantity",    importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "UnitPrice",   importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "NetAmount",   importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Position",
                type_name_plural: "Positionen",
                title_path: "ItemID",
                description_path: "ProductName",
            },
            header_facets: &[
                HeaderFacetDef { data_point_qualifier: "NetAmount", label: "Nettobetrag" },
            ],
            data_points: &[
                DataPointDef { qualifier: "NetAmount", value_path: "NetAmount", title: "Nettobetrag", max_value: None, visualization: None },
            ],
            facet_sections: &[
                FacetSectionDef { label: "Positionsdetails", id: "ItemDetails", field_group_qualifier: "ItemInfo", field_group_label: "Informationen" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "ItemInfo", fields: &["ItemID", "OrderID", "ProductID", "ProductName", "Quantity", "UnitPrice", "Currency", "NetAmount"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }

    // OrderItems sind Kompositionen — kein eigener Tile, keine eigene Route.
    fn manifest_inbound(&self) -> (String, serde_json::Value) {
        // Kein eigener Inbound — wird nie direkt navigiert
        ("_OrderItems-stub".to_string(), json!(null))
    }
    fn manifest_routes(&self) -> Vec<Value> {
        vec![]
    }
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![]
    }
}
