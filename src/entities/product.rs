use serde_json::{json, Value};

use crate::annotations::*;
use crate::entity::ODataEntity;
use crate::NAMESPACE;

#[derive(Debug)]
pub struct ProductEntity;

impl ODataEntity for ProductEntity {
    fn set_name(&self) -> &'static str {
        "Products"
    }
    fn key_field(&self) -> &'static str {
        "ProductID"
    }
    fn type_name(&self) -> &'static str {
        "Product"
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![
            json!({"ProductID": "P001", "ProductName": "Laptop Pro 15",    "Category": "Electronics",
                   "Supplier": "TechCorp GmbH",      "Status": "A", "StatusCriticality": 3,
                   "Price": "1299.99", "Currency": "EUR",
                   "UnitsInStock": 42,  "Rating": 5, "CreatedAt": "2023-11-14T22:13:20Z",
                   "Description": "Leistungsstarkes Business-Notebook mit 15 Zoll Display."}),
            json!({"ProductID": "P002", "ProductName": "Wireless Mouse",   "Category": "Accessories",
                   "Supplier": "PeriphTech AG",      "Status": "A", "StatusCriticality": 3,
                   "Price": "29.95",  "Currency": "EUR",
                   "UnitsInStock": 210, "Rating": 4, "CreatedAt": "2023-11-26T09:46:40Z",
                   "Description": "Ergonomische kabellose Maus mit USB-C Empfaenger."}),
            json!({"ProductID": "P003", "ProductName": "USB-C Hub 7-Port", "Category": "Accessories",
                   "Supplier": "ConnectWorld Ltd.",  "Status": "A", "StatusCriticality": 3,
                   "Price": "49.90",  "Currency": "EUR",
                   "UnitsInStock": 88,  "Rating": 4, "CreatedAt": "2023-12-07T21:20:00Z",
                   "Description": "7-Port USB-C Hub mit HDMI, SD-Card und Ethernet."}),
            json!({"ProductID": "P004", "ProductName": "4K Monitor 27\"",  "Category": "Electronics",
                   "Supplier": "DisplayMax SE",      "Status": "B", "StatusCriticality": 2,
                   "Price": "549.00", "Currency": "EUR",
                   "UnitsInStock": 15,  "Rating": 5, "CreatedAt": "2023-12-19T08:53:20Z",
                   "Description": "27 Zoll 4K UHD IPS Monitor mit USB-C Stromversorgung."}),
            json!({"ProductID": "P005", "ProductName": "Desk Lamp LED",    "Category": "Office",
                   "Supplier": "LightDesign KG",     "Status": "A", "StatusCriticality": 3,
                   "Price": "39.99",  "Currency": "EUR",
                   "UnitsInStock": 0,   "Rating": 3, "CreatedAt": "2023-12-30T20:26:40Z",
                   "Description": "Dimmbare LED-Schreibtischlampe mit Farbtemperaturregelung."}),
        ]
    }

    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        static FIELDS: &[FieldDef] = &[
            FieldDef { name: "ProductID",         label: "Produkt-ID",    edm_type: "Edm.String",          max_length: Some(10),  precision: None,     scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "ProductName",       label: "Produktname",   edm_type: "Edm.String",          max_length: Some(80),  precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Category",          label: "Kategorie",     edm_type: "Edm.String",          max_length: Some(40),  precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Supplier",          label: "Lieferant",     edm_type: "Edm.String",          max_length: Some(80),  precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Status",            label: "Status",        edm_type: "Edm.String",          max_length: Some(1),   precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "StatusCriticality", label: "Kritikalitaet", edm_type: "Edm.Byte",            max_length: None,      precision: None,     scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "Price",             label: "Preis",         edm_type: "Edm.Decimal",         max_length: None,      precision: Some(15), scale: Some(2), immutable: false, semantic_object: None },
            FieldDef { name: "Currency",          label: "Waehrung",      edm_type: "Edm.String",          max_length: Some(3),   precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "UnitsInStock",      label: "Lagerbestand",  edm_type: "Edm.Int32",           max_length: None,      precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "Rating",            label: "Bewertung",     edm_type: "Edm.Byte",            max_length: None,      precision: None,     scale: None, immutable: false, semantic_object: None },
            FieldDef { name: "CreatedAt",         label: "Erstellt am",   edm_type: "Edm.DateTimeOffset",  max_length: None,      precision: None,     scale: None, immutable: true,  semantic_object: None },
            FieldDef { name: "Description",       label: "Beschreibung",  edm_type: "Edm.String",          max_length: Some(500), precision: None,     scale: None, immutable: false, semantic_object: None },
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"Products\" EntityType=\"{ns}.Product\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"Products\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &["Category", "Status", "Supplier"],
            line_item: &[
                LineItemField { name: "ProductID",    importance: Some("High"), criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "ProductName",  importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Category",     importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Supplier",     importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Price",        importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "UnitsInStock", importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Rating",       importance: None, criticality_path: None, navigation_path: None, semantic_object: None },
                LineItemField { name: "Status",       importance: None, criticality_path: Some("StatusCriticality"), navigation_path: None, semantic_object: None },
            ],
            header_info: HeaderInfoDef {
                type_name: "Produkt",
                type_name_plural: "Produkte",
                title_path: "ProductName",
                description_path: "ProductID",
            },
            header_facets: &[
                HeaderFacetDef { data_point_qualifier: "Price",  label: "Preis" },
                HeaderFacetDef { data_point_qualifier: "Stock",  label: "Lagerbestand" },
                HeaderFacetDef { data_point_qualifier: "Rating", label: "Bewertung" },
            ],
            data_points: &[
                DataPointDef { qualifier: "Price",  value_path: "Price",        title: "Preis",        max_value: None,    visualization: None },
                DataPointDef { qualifier: "Stock",  value_path: "UnitsInStock", title: "Lagerbestand", max_value: None,    visualization: None },
                DataPointDef { qualifier: "Rating", value_path: "Rating",       title: "Bewertung",    max_value: Some(5), visualization: Some("Rating") },
            ],
            facet_sections: &[
                FacetSectionDef { label: "Allgemeine Informationen", id: "GeneralInfo", field_group_qualifier: "General", field_group_label: "Produktdetails" },
                FacetSectionDef { label: "Preis &amp; Bestand",     id: "PriceStock",  field_group_qualifier: "Pricing", field_group_label: "Preisdetails" },
            ],
            field_groups: &[
                FieldGroupDef { qualifier: "General", fields: &["ProductID", "ProductName", "Category", "Supplier", "Status", "CreatedAt", "Description"] },
                FieldGroupDef { qualifier: "Pricing", fields: &["Price", "Currency", "UnitsInStock", "Rating"] },
            ],
            table_facets: &[],
        };
        Some(&DEF)
    }
}
