use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct OrderEntity;

impl ODataEntity for OrderEntity {
    fn set_name(&self) -> &'static str {
        "Orders"
    }
    fn key_field(&self) -> &'static str {
        "OrderID"
    }
    fn type_name(&self) -> &'static str {
        "Order"
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![
            json!({"OrderID": "O001", "CustomerName": "Müller GmbH",
                   "Quantity": 3, "TotalAmount": "3899.97", "Currency": "EUR",
                   "Status": "C", "StatusCriticality": 3,
                   "OrderDate": "2024-01-10T08:30:00Z", "DeliveryDate": "2024-01-17T14:00:00Z",
                   "Note": "Express-Lieferung gewünscht."}),
            json!({"OrderID": "O002", "CustomerName": "Schmidt AG",
                   "Quantity": 10, "TotalAmount": "299.50", "Currency": "EUR",
                   "Status": "C", "StatusCriticality": 3,
                   "OrderDate": "2024-01-15T10:00:00Z", "DeliveryDate": "2024-01-20T09:00:00Z",
                   "Note": ""}),
            json!({"OrderID": "O003", "CustomerName": "Weber & Söhne KG",
                   "Quantity": 2, "TotalAmount": "1098.00", "Currency": "EUR",
                   "Status": "P", "StatusCriticality": 2,
                   "OrderDate": "2024-02-01T14:20:00Z", "DeliveryDate": null,
                   "Note": "Lieferung an Filiale Nord."}),
            json!({"OrderID": "O004", "CustomerName": "Müller GmbH",
                   "Quantity": 5, "TotalAmount": "249.50", "Currency": "EUR",
                   "Status": "P", "StatusCriticality": 2,
                   "OrderDate": "2024-02-05T09:15:00Z", "DeliveryDate": null,
                   "Note": ""}),
            json!({"OrderID": "O005", "CustomerName": "Becker IT Services",
                   "Quantity": 1, "TotalAmount": "39.99", "Currency": "EUR",
                   "Status": "X", "StatusCriticality": 1,
                   "OrderDate": "2024-02-10T16:45:00Z", "DeliveryDate": null,
                   "Note": "Storniert – Produkt nicht lieferbar."}),
        ]
    }

    fn expand_record(&self, record: &mut Value, nav_properties: &[&str], entities: &[&dyn ODataEntity], data_store: &std::collections::HashMap<String, Vec<Value>>) {
        // Items-Expansion: OrderItems filtern nach OrderID
        if nav_properties.contains(&"Items") {
            let found_entity = entities.iter().find(|e| e.set_name() == "OrderItems");
            if let Some(entity) = found_entity {
                let order_id = record
                    .get("OrderID")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(oid) = order_id {
                    let data = data_store.get(entity.set_name())
                        .cloned()
                        .unwrap_or_else(|| entity.mock_data());
                    let items: Vec<Value> = data
                        .into_iter()
                        .filter(|item| item.get("OrderID").and_then(|v| v.as_str()) == Some(&oid))
                        .collect();
                    if let Some(obj) = record.as_object_mut() {
                        obj.insert("Items".to_string(), Value::Array(items));
                    }
                }
            }
        }
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "OrderID",           label: "Bestell-Nr.",   edm_type: "Edm.String",          max_length: Some(10),  precision: None,     scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "CustomerName",      label: "Kunde",         edm_type: "Edm.String",          max_length: Some(80),  precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "Quantity",          label: "Menge",         edm_type: "Edm.Int32",           max_length: None,      precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "TotalAmount",       label: "Gesamtbetrag",  edm_type: "Edm.Decimal",         max_length: None,      precision: Some(15), scale: Some(2), immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "Currency",          label: "Waehrung",      edm_type: "Edm.String",          max_length: Some(3),   precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "Status",            label: "Status",        edm_type: "Edm.String",          max_length: Some(1),   precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "StatusCriticality", label: "Kritikalitaet", edm_type: "Edm.Byte",            max_length: None,      precision: None,     scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "OrderDate",         label: "Bestelldatum",  edm_type: "Edm.DateTimeOffset",  max_length: None,      precision: None,     scale: None, immutable: true,  semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "DeliveryDate",      label: "Lieferdatum",   edm_type: "Edm.DateTimeOffset",  max_length: None,      precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
            FieldDef { name: "Note",              label: "Notiz",         edm_type: "Edm.String",          max_length: Some(500), precision: None,     scale: None, immutable: false, semantic_object: None, value_source: None , value_list: None},
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef { name: "Items", target_type: "OrderItem", is_collection: true, foreign_key: None },
        ];
        NAV
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"Orders\" EntityType=\"{ns}.Order\">\n\
             <NavigationPropertyBinding Path=\"Items\" Target=\"OrderItems\"/>\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"Orders\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &["Status", "CustomerName"],
            line_item: &[
                LineItemField { name: "OrderID",      label: None, importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "CustomerName", label: None, importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Quantity",     label: None, importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "TotalAmount",  label: None, importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "OrderDate",    label: None, importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Status",       label: None, importance: None, criticality_path: Some("StatusCriticality"), navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Bestellung",
                type_name_plural: "Bestellungen",
                title_path: "OrderID",
                description_path: "CustomerName",
            },
            header_facets: &[
                HeaderFacetDef { data_point_qualifier: "TotalAmount", label: "Gesamtbetrag" },
                HeaderFacetDef { data_point_qualifier: "Quantity",    label: "Menge" },
            ],
            data_points: &[
                DataPointDef { qualifier: "TotalAmount", value_path: "TotalAmount", title: "Gesamtbetrag", max_value: None, visualization: None },
                DataPointDef { qualifier: "Quantity",    value_path: "Quantity",    title: "Menge",        max_value: None, visualization: None },
            ],
            facet_sections: &[
                FacetSectionDef { label: "Bestelldetails", id: "OrderDetails", field_group_qualifier: "OrderInfo", field_group_label: "Informationen" },
                FacetSectionDef { label: "Lieferung",      id: "Delivery",     field_group_qualifier: "Delivery",  field_group_label: "Lieferdetails" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "OrderInfo", fields: &["OrderID", "CustomerName", "Quantity", "TotalAmount", "Currency", "Status"] },
                FieldGroupDef { qualifier: "Delivery",  fields: &["OrderDate", "DeliveryDate", "Note"] },
            ],
            table_facets: &[
                TableFacetDef { label: "Positionen", id: "ItemsSection", navigation_property: "Items" },
            ],
        };
        Some(&DEF)
    }

    /// Eigene Routen mit Items-Sub-ObjectPage.
    fn manifest_routes(&self) -> Vec<Value> {
        vec![
            json!({
                "pattern": "Orders:?query:",
                "name": "OrdersList",
                "target": "OrdersList"
            }),
            json!({
                "pattern": "Orders({key}):?query:",
                "name": "OrdersObjectPage",
                "target": ["OrdersList", "OrdersObjectPage"]
            }),
            json!({
                "pattern": "Orders({key})/Items({key2}):?query:",
                "name": "OrderItemsObjectPage",
                "target": ["OrdersList", "OrdersObjectPage", "OrderItemsObjectPage"]
            }),
        ]
    }

    /// Eigene Targets mit Items-Sub-ObjectPage.
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        vec![
            (
                "OrdersList".to_string(),
                json!({
                    "type": "Component",
                    "id": "OrdersList",
                    "name": "sap.fe.templates.ListReport",
                    "options": {
                        "settings": {
                            "contextPath": "/Orders",
                            "variantManagement": "Page",
                            "initialLoad": "Enabled",
                            "navigation": {
                                "Orders": {
                                    "detail": {
                                        "route": "OrdersObjectPage"
                                    }
                                }
                            }
                        }
                    },
                    "controlAggregation": "beginColumnPages",
                    "contextPattern": ""
                }),
            ),
            (
                "OrdersObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "OrdersObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": {
                        "settings": {
                            "contextPath": "/Orders",
                            "navigation": {
                                "Items": {
                                    "detail": {
                                        "route": "OrderItemsObjectPage"
                                    }
                                }
                            }
                        }
                    },
                    "controlAggregation": "midColumnPages",
                    "contextPattern": "/Orders({key})"
                }),
            ),
            (
                "OrderItemsObjectPage".to_string(),
                json!({
                    "type": "Component",
                    "id": "OrderItemsObjectPage",
                    "name": "sap.fe.templates.ObjectPage",
                    "options": {
                        "settings": {
                            "contextPath": "/Orders/Items"
                        }
                    },
                    "controlAggregation": "endColumnPages",
                    "contextPattern": "/Orders({key})/Items({key2})"
                }),
            ),
        ]
    }
}
