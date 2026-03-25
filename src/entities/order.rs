use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

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
            json!({"OrderID": "O001", "ProductID": "P001", "CustomerName": "Müller GmbH",
                   "Quantity": 3, "TotalAmount": "3899.97", "Currency": "EUR",
                   "Status": "C", "StatusCriticality": 3,
                   "OrderDate": "2024-01-10T08:30:00Z", "DeliveryDate": "2024-01-17T14:00:00Z",
                   "Note": "Express-Lieferung gewünscht."}),
            json!({"OrderID": "O002", "ProductID": "P002", "CustomerName": "Schmidt AG",
                   "Quantity": 10, "TotalAmount": "299.50", "Currency": "EUR",
                   "Status": "C", "StatusCriticality": 3,
                   "OrderDate": "2024-01-15T10:00:00Z", "DeliveryDate": "2024-01-20T09:00:00Z",
                   "Note": ""}),
            json!({"OrderID": "O003", "ProductID": "P004", "CustomerName": "Weber & Söhne KG",
                   "Quantity": 2, "TotalAmount": "1098.00", "Currency": "EUR",
                   "Status": "P", "StatusCriticality": 2,
                   "OrderDate": "2024-02-01T14:20:00Z", "DeliveryDate": null,
                   "Note": "Lieferung an Filiale Nord."}),
            json!({"OrderID": "O004", "ProductID": "P003", "CustomerName": "Müller GmbH",
                   "Quantity": 5, "TotalAmount": "249.50", "Currency": "EUR",
                   "Status": "P", "StatusCriticality": 2,
                   "OrderDate": "2024-02-05T09:15:00Z", "DeliveryDate": null,
                   "Note": ""}),
            json!({"OrderID": "O005", "ProductID": "P005", "CustomerName": "Becker IT Services",
                   "Quantity": 1, "TotalAmount": "39.99", "Currency": "EUR",
                   "Status": "X", "StatusCriticality": 1,
                   "OrderDate": "2024-02-10T16:45:00Z", "DeliveryDate": null,
                   "Note": "Storniert – Produkt nicht lieferbar."}),
        ]
    }

    fn expand_record(&self, record: &mut Value, nav_properties: &[&str], entities: &[&dyn ODataEntity]) {
        if !nav_properties.contains(&"Product") {
            return;
        }
        let found_entity = entities.iter().find(|e| e.set_name() == "Products");
        if let Some(entity) = found_entity {
            let pid = record
                .get("ProductID")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(pid) = pid {
                let data = entity.mock_data();
                let found = data
                    .into_iter()
                    .find(|p| p.get(entity.key_field()).and_then(|v| v.as_str()) == Some(&pid));
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("Product".to_string(), found.unwrap_or(Value::Null));
                }
            }
        }
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "OrderID",           label: "Bestell-Nr.",   edm_type: "Edm.String",          max_length: Some(10),  precision: None,     scale: None },
            FieldDef { name: "ProductID",         label: "Produkt-ID",    edm_type: "Edm.String",          max_length: Some(10),  precision: None,     scale: None },
            FieldDef { name: "CustomerName",      label: "Kunde",         edm_type: "Edm.String",          max_length: Some(80),  precision: None,     scale: None },
            FieldDef { name: "Quantity",          label: "Menge",         edm_type: "Edm.Int32",           max_length: None,      precision: None,     scale: None },
            FieldDef { name: "TotalAmount",       label: "Gesamtbetrag",  edm_type: "Edm.Decimal",         max_length: None,      precision: Some(15), scale: Some(2) },
            FieldDef { name: "Currency",          label: "Waehrung",      edm_type: "Edm.String",          max_length: Some(3),   precision: None,     scale: None },
            FieldDef { name: "Status",            label: "Status",        edm_type: "Edm.String",          max_length: Some(1),   precision: None,     scale: None },
            FieldDef { name: "StatusCriticality", label: "Kritikalitaet", edm_type: "Edm.Byte",            max_length: None,      precision: None,     scale: None },
            FieldDef { name: "OrderDate",         label: "Bestelldatum",  edm_type: "Edm.DateTimeOffset",  max_length: None,      precision: None,     scale: None },
            FieldDef { name: "DeliveryDate",      label: "Lieferdatum",   edm_type: "Edm.DateTimeOffset",  max_length: None,      precision: None,     scale: None },
            FieldDef { name: "Note",              label: "Notiz",         edm_type: "Edm.String",          max_length: Some(500), precision: None,     scale: None },
        ];
        Some(FIELDS)
    }

    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        static NAV: &[NavigationPropertyDef] = &[
            NavigationPropertyDef { name: "Product", target_type: "Product" },
        ];
        NAV
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"Orders\" EntityType=\"{ns}.Order\"><NavigationPropertyBinding Path=\"Product\" Target=\"Products\"/></EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &["Status", "CustomerName", "ProductID"],
            line_item: &[
                LineItemField { name: "OrderID",      importance: Some("High"), criticality_path: None, navigation_path: None },
                LineItemField { name: "ProductID",    importance: None, criticality_path: None, navigation_path: Some("Product") },
                LineItemField { name: "CustomerName", importance: None, criticality_path: None, navigation_path: None },
                LineItemField { name: "Quantity",     importance: None, criticality_path: None, navigation_path: None },
                LineItemField { name: "TotalAmount",  importance: None, criticality_path: None, navigation_path: None },
                LineItemField { name: "OrderDate",    importance: None, criticality_path: None, navigation_path: None },
                LineItemField { name: "Status",       importance: None, criticality_path: Some("StatusCriticality"), navigation_path: None },
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
                FieldGroupDef { qualifier: "OrderInfo", fields: &["OrderID", "ProductID", "CustomerName", "Quantity", "TotalAmount", "Currency", "Status"] },
                FieldGroupDef { qualifier: "Delivery",  fields: &["OrderDate", "DeliveryDate", "Note"] },
            ],
        };
        Some(&DEF)
    }
}
