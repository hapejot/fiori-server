use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

const BASE_PATH: &str = "/odata/v4/ProductsService";
const NAMESPACE: &str = "ProductsService";

// ── Webapp directory (sibling to the executable's working dir) ──────────
fn webapp_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("webapp")
}

// ═══════════════════════════════════════════════════════════════════════
// Annotation-Structs – datengetriebene UI-Annotation-Erzeugung
//
// Die Structs beschreiben deklarativ, welche Felder in SelectionFields,
// LineItem, HeaderFacets, DataPoints, Facets und FieldGroups erscheinen.
// build_annotations_xml() erzeugt daraus das komplette EDMX-XML.
// ═══════════════════════════════════════════════════════════════════════

/// Einheitliche Feld-Definition fuer EntityType-Properties UND Annotations.
struct FieldDef {
    name: &'static str,
    label: &'static str,
    edm_type: &'static str,
    max_length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
}

/// LineItem-Referenz mit Annotation-spezifischen Attributen.
struct LineItemField {
    name: &'static str,
    importance: Option<&'static str>,
    criticality_path: Option<&'static str>,
    /// Navigation-Property-Pfad – erzeugt UI.DataFieldWithNavigationPath.
    navigation_path: Option<&'static str>,
}

/// NavigationProperty-Definition im EntityType.
struct NavigationPropertyDef {
    name: &'static str,
    target_type: &'static str,
}

/// DataPoint fuer den Object-Page-Header.
struct DataPointDef {
    qualifier: &'static str,
    value_path: &'static str,
    title: &'static str,
    max_value: Option<u32>,
    visualization: Option<&'static str>,
}

/// ReferenceFacet im HeaderFacets-Block – verweist auf einen DataPoint.
struct HeaderFacetDef {
    data_point_qualifier: &'static str,
    label: &'static str,
}

/// Eine Gruppe von Feldern (z.B. "General", "Pricing").
struct FieldGroupDef {
    qualifier: &'static str,
    fields: &'static [&'static str],
}

/// Ein CollectionFacet auf der Object Page, verweist auf eine FieldGroup.
struct FacetSectionDef {
    label: &'static str,
    id: &'static str,
    field_group_qualifier: &'static str,
    field_group_label: &'static str,
}

/// Kopfzeile der Object Page.
struct HeaderInfoDef {
    type_name: &'static str,
    type_name_plural: &'static str,
    title_path: &'static str,
    description_path: &'static str,
}

/// Komplette Annotation-Definition fuer eine Entitaet.
struct AnnotationsDef {
    selection_fields: &'static [&'static str],
    line_item: &'static [LineItemField],
    header_info: HeaderInfoDef,
    header_facets: &'static [HeaderFacetDef],
    data_points: &'static [DataPointDef],
    facet_sections: &'static [FacetSectionDef],
    field_groups: &'static [FieldGroupDef],
}

/// Erzeugt das Annotations-XML fuer eine Entitaet aus ihrer AnnotationsDef.
fn build_annotations_xml(
    entity_type_name: &str,
    def: &AnnotationsDef,
    fields: &[FieldDef],
) -> String {
    let mut x = format!(
        "      <Annotations Target=\"{}.{}\">\n",
        NAMESPACE, entity_type_name
    );

    // ── SelectionFields ──
    x.push_str("<Annotation Term=\"UI.SelectionFields\">\n");
    x.push_str("<Collection>\n");
    for f in def.selection_fields {
        x.push_str(&format!("            <PropertyPath>{}</PropertyPath>\n", f));
    }
    x.push_str("          </Collection>\n");
    x.push_str("        </Annotation>\n");

    // ── LineItem ──
    x.push_str("\n        <!-- LineItem (Tabellenspalten) -->\n");
    x.push_str("        <Annotation Term=\"UI.LineItem\">\n");
    x.push_str("          <Collection>\n");
    for f in def.line_item {
        let label = fields
            .iter()
            .find(|fd| fd.name == f.name)
            .map(|fd| fd.label)
            .unwrap_or(f.name);
        let record_type = if f.navigation_path.is_some() {
            "UI.DataFieldWithNavigationPath"
        } else {
            "UI.DataField"
        };
        x.push_str(&format!("            <Record Type=\"{}\">\n", record_type));
        x.push_str(&format!(
            "              <PropertyValue Property=\"Value\" Path=\"{}\"/>\n",
            f.name
        ));
        x.push_str(&format!(
            "              <PropertyValue Property=\"Label\" String=\"{}\"/>\n",
            label
        ));
        if let Some(nav) = f.navigation_path {
            x.push_str(&format!(
                "              <PropertyValue Property=\"Target\" NavigationPropertyPath=\"{}\"/>\n",
                nav
            ));
        }
        if let Some(imp) = f.importance {
            x.push_str(&format!(
                "              <PropertyValue Property=\"![@UI.Importance]\" EnumMember=\"UI.ImportanceType/{}\"/>\n",
                imp
            ));
        }
        if let Some(crit) = f.criticality_path {
            x.push_str(&format!(
                "              <PropertyValue Property=\"Criticality\" Path=\"{}\"/>\n",
                crit
            ));
        }
        x.push_str("            </Record>\n");
    }
    x.push_str("          </Collection>\n");
    x.push_str("        </Annotation>\n");

    // ── HeaderInfo ──
    x.push_str("\n        <!-- HeaderInfo -->\n");
    x.push_str("        <Annotation Term=\"UI.HeaderInfo\">\n");
    x.push_str("          <Record Type=\"UI.HeaderInfoType\">\n");
    x.push_str(&format!(
        "            <PropertyValue Property=\"TypeName\" String=\"{}\"/>\n",
        def.header_info.type_name
    ));
    x.push_str(&format!(
        "            <PropertyValue Property=\"TypeNamePlural\" String=\"{}\"/>\n",
        def.header_info.type_name_plural
    ));
    x.push_str("            <PropertyValue Property=\"Title\">\n");
    x.push_str("              <Record Type=\"UI.DataField\">\n");
    x.push_str(&format!(
        "                <PropertyValue Property=\"Value\" Path=\"{}\"/>\n",
        def.header_info.title_path
    ));
    x.push_str("              </Record>\n");
    x.push_str("            </PropertyValue>\n");
    x.push_str("            <PropertyValue Property=\"Description\">\n");
    x.push_str("              <Record Type=\"UI.DataField\">\n");
    x.push_str(&format!(
        "                <PropertyValue Property=\"Value\" Path=\"{}\"/>\n",
        def.header_info.description_path
    ));
    x.push_str("              </Record>\n");
    x.push_str("            </PropertyValue>\n");
    x.push_str("          </Record>\n");
    x.push_str("        </Annotation>\n");

    // ── HeaderFacets ──
    x.push_str("\n        <!-- HeaderFacets -->\n");
    x.push_str("        <Annotation Term=\"UI.HeaderFacets\">\n");
    x.push_str("          <Collection>\n");
    for hf in def.header_facets {
        x.push_str("            <Record Type=\"UI.ReferenceFacet\">\n");
        x.push_str(&format!(
            "              <PropertyValue Property=\"Target\" AnnotationPath=\"@UI.DataPoint#{}\"/>\n",
            hf.data_point_qualifier
        ));
        x.push_str(&format!(
            "              <PropertyValue Property=\"Label\" String=\"{}\"/>\n",
            hf.label
        ));
        x.push_str("            </Record>\n");
    }
    x.push_str("          </Collection>\n");
    x.push_str("        </Annotation>\n");

    // ── DataPoints ──
    x.push_str("\n        <!-- DataPoints -->\n");
    for dp in def.data_points {
        x.push_str(&format!(
            "        <Annotation Term=\"UI.DataPoint\" Qualifier=\"{}\">\n",
            dp.qualifier
        ));
        x.push_str("          <Record Type=\"UI.DataPointType\">\n");
        x.push_str(&format!(
            "            <PropertyValue Property=\"Value\" Path=\"{}\"/>\n",
            dp.value_path
        ));
        x.push_str(&format!(
            "            <PropertyValue Property=\"Title\" String=\"{}\"/>\n",
            dp.title
        ));
        if let Some(max) = dp.max_value {
            x.push_str(&format!(
                "            <PropertyValue Property=\"MaximumValue\" Int=\"{}\"/>\n",
                max
            ));
        }
        if let Some(vis) = dp.visualization {
            x.push_str(&format!(
                "            <PropertyValue Property=\"Visualization\" EnumMember=\"UI.VisualizationType/{}\"/>\n",
                vis
            ));
        }
        x.push_str("          </Record>\n");
        x.push_str("        </Annotation>\n");
    }

    // ── Facets (Object Page Sections) ──
    x.push_str("\n        <!-- Facets (Object Page Sections) -->\n");
    x.push_str("        <Annotation Term=\"UI.Facets\">\n");
    x.push_str("          <Collection>\n");
    for sec in def.facet_sections {
        x.push_str("            <Record Type=\"UI.CollectionFacet\">\n");
        x.push_str(&format!(
            "              <PropertyValue Property=\"Label\" String=\"{}\"/>\n",
            sec.label
        ));
        x.push_str(&format!(
            "              <PropertyValue Property=\"ID\" String=\"{}\"/>\n",
            sec.id
        ));
        x.push_str("              <PropertyValue Property=\"Facets\">\n");
        x.push_str("                <Collection>\n");
        x.push_str("                  <Record Type=\"UI.ReferenceFacet\">\n");
        x.push_str(&format!(
            "                    <PropertyValue Property=\"Target\" AnnotationPath=\"@UI.FieldGroup#{}\"/>\n",
            sec.field_group_qualifier
        ));
        x.push_str(&format!(
            "                    <PropertyValue Property=\"Label\" String=\"{}\"/>\n",
            sec.field_group_label
        ));
        x.push_str("                  </Record>\n");
        x.push_str("                </Collection>\n");
        x.push_str("              </PropertyValue>\n");
        x.push_str("            </Record>\n");
    }
    x.push_str("          </Collection>\n");
    x.push_str("        </Annotation>\n");

    // ── FieldGroups ──
    for fg in def.field_groups {
        x.push_str(&format!(
            "\n        <Annotation Term=\"UI.FieldGroup\" Qualifier=\"{}\">\n",
            fg.qualifier
        ));
        x.push_str("          <Record Type=\"UI.FieldGroupType\">\n");
        x.push_str("            <PropertyValue Property=\"Data\">\n");
        x.push_str("              <Collection>\n");
        for name in fg.fields {
            let label = fields
                .iter()
                .find(|fd| fd.name == *name)
                .map(|fd| fd.label)
                .unwrap_or(name);
            x.push_str("                <Record Type=\"UI.DataField\">\n");
            x.push_str(&format!(
                "                  <PropertyValue Property=\"Value\" Path=\"{}\"/>\n",
                name
            ));
            x.push_str(&format!(
                "                  <PropertyValue Property=\"Label\" String=\"{}\"/>\n",
                label
            ));
            x.push_str("                </Record>\n");
        }
        x.push_str("              </Collection>\n");
        x.push_str("            </PropertyValue>\n");
        x.push_str("          </Record>\n");
        x.push_str("        </Annotation>\n");
    }

    x.push_str("\n      </Annotations>");
    x
}

/// Erzeugt das EntityType-XML aus Typ-Name, Schluesselfeld und Property-Definitionen.
fn build_entity_type_xml(type_name: &str, key_field: &str, props: &[FieldDef]) -> String {
    let mut x = format!("      <EntityType Name=\"{}\">\n", type_name);
    x.push_str("        <Key>\n");
    x.push_str(&format!(
        "          <PropertyRef Name=\"{}\"/>\n",
        key_field
    ));
    x.push_str("        </Key>\n");
    for p in props {
        let mut attr = format!("Type=\"{}\"", p.edm_type);
        if p.name == key_field {
            attr.push_str("         Nullable=\"false\"");
        }
        if let Some(ml) = p.max_length {
            attr.push_str(&format!(" MaxLength=\"{}\"", ml));
        }
        if let Some(prec) = p.precision {
            attr.push_str(&format!(" Precision=\"{}\"", prec));
        }
        if let Some(sc) = p.scale {
            attr.push_str(&format!(" Scale=\"{}\"", sc));
        }
        // Padding fuer lesbare Ausrichtung
        let pad = if p.name.len() < 18 {
            " ".repeat(18 - p.name.len())
        } else {
            " ".to_string()
        };
        x.push_str(&format!(
            "        <Property Name=\"{}\"{}{}/>\n",
            p.name, pad, attr
        ));
    }
    x.push_str("      </EntityType>");
    x
}

/// Haengt NavigationProperty-Elemente an einen EntityType-XML-String an.
fn append_navigation_properties(xml: &mut String, nav_props: &[NavigationPropertyDef]) {
    for np in nav_props {
        xml.insert_str(
            xml.rfind("</EntityType>").unwrap(),
            &format!(
                "        <NavigationProperty Name=\"{}\" Type=\"{}.{}\"/>\n",
                np.name, NAMESPACE, np.target_type
            ),
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// OData Entity – Trait + Implementierungen
//
// Neue Entitaet hinzufuegen:
//   1. Neues Struct anlegen, ODataEntity-Trait implementieren
//   2. set_name, key_field, type_name, mock_data, entity_type,
//      entity_set, annotations_def (und ggf. expand_record) implementieren
//   3. Instanz in ENTITIES-Slice eintragen
// ═══════════════════════════════════════════════════════════════════════

/// Basisklasse (Trait) fuer eine OData-Entitaet.
trait ODataEntity: Sync {
    /// Name des EntitySets (z.B. "Products", "Orders")
    fn set_name(&self) -> &'static str;
    /// Name des Schluesselfelds (z.B. "ProductID")
    fn key_field(&self) -> &'static str;
    /// Name des Entity-Typs (z.B. "Product", "Order")
    fn type_name(&self) -> &'static str;
    /// Mock-Daten als JSON-Array
    fn mock_data(&self) -> Vec<Value>;
    /// Einheitliche Feld-Definitionen – eine Liste fuer EntityType UND Annotations.
    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        None
    }
    /// NavigationProperty-Definitionen (optional).
    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        &[]
    }
    /// EDMX EntityType-XML – wird automatisch aus fields_def() erzeugt
    /// oder kann manuell ueberschrieben werden.
    fn entity_type(&self) -> String {
        match self.fields_def() {
            Some(fields) => {
                let mut xml = build_entity_type_xml(self.type_name(), self.key_field(), fields);
                append_navigation_properties(&mut xml, self.navigation_properties());
                xml
            }
            None => String::new(),
        }
    }
    /// EDMX EntitySet-XML
    fn entity_set(&self) -> String;
    /// Deklarative Annotation-Definition (optional).
    /// Wird von der Default-Impl von annotations() verwendet.
    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        None
    }
    /// EDMX Annotations-XML – wird automatisch aus annotations_def() erzeugt.
    fn annotations(&self) -> String {
        match (self.annotations_def(), self.fields_def()) {
            (Some(def), Some(fields)) => build_annotations_xml(self.type_name(), def, fields),
            (Some(def), None) => build_annotations_xml(self.type_name(), def, &[]),
            _ => String::new(),
        }
    }
    /// $expand-Logik auf einen einzelnen Datensatz anwenden (optional).
    fn expand_record(&self, _record: &mut Value, _nav_properties: &[&str], _entities: &[&dyn ODataEntity]) {}

    /// Tile-Titel fuer den FLP Sandbox Launchpad (Default: type_name_plural aus HeaderInfo).
    fn tile_title(&self) -> &str {
        self.annotations_def()
            .map(|d| d.header_info.type_name_plural)
            .unwrap_or(self.set_name())
    }

    /// Manifest crossNavigation inbound Schluessel (z.B. "Products-display").
    fn manifest_inbound_key(&self) -> String {
        format!("{}-display", self.set_name())
    }

    /// Manifest crossNavigation inbound-Eintrag.
    fn manifest_inbound(&self) -> (String, Value) {
        (
            self.manifest_inbound_key(),
            json!({
                "semanticObject": self.set_name(),
                "action": "display",
                "signature": {
                    "parameters": {},
                    "additionalParameters": "allowed"
                }
            }),
        )
    }

    /// Manifest-Routing: Liefert die Routen (routes) fuer dieses EntitySet.
    /// Default-Impl erzeugt ListReport- und ObjectPage-Routen aus set_name().
    fn manifest_routes(&self) -> Vec<Value> {
        let name = self.set_name();
        vec![
            json!({
                "pattern": format!("{}:?query:", name),
                "name": format!("{}List", name),
                "target": format!("{}List", name)
            }),
            json!({
                "pattern": format!("{}({{key}}):?query:", name),
                "name": format!("{}ObjectPage", name),
                "target": [format!("{}List", name), format!("{}ObjectPage", name)]
            }),
        ]
    }

    /// Manifest-Routing: Liefert die Targets (ListReport + ObjectPage)
    /// fuer dieses EntitySet. Default-Impl erzeugt Standard-Fiori-Elements-Targets.
    fn manifest_targets(&self) -> Vec<(String, Value)> {
        let name = self.set_name();
        vec![
            (
                format!("{}List", name),
                json!({
                    "type": "Component",
                    "id": format!("{}List", name),
                    "name": "sap.fe.templates.ListReport",
                    "options": {
                        "settings": {
                            "contextPath": format!("/{}", name),
                            "variantManagement": "Page",
                            "initialLoad": "Enabled",
                            "navigation": {
                                (name): {
                                    "detail": {
                                        "route": format!("{}ObjectPage", name)
                                    }
                                }
                            }
                        }
                    }
                }),
            ),
            (
                format!("{}ObjectPage", name),
                json!({
                    "type": "Component",
                    "id": format!("{}ObjectPage", name),
                    "name": "sap.fe.templates.ObjectPage",
                    "options": {
                        "settings": {
                            "contextPath": format!("/{}", name)
                        }
                    }
                }),
            ),
        ]
    }
}

// ── ProductEntity ───────────────────────────────────────────────────
struct ProductEntity;

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
            FieldDef {
                name: "ProductID",
                label: "Produkt-ID",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "ProductName",
                label: "Produktname",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Category",
                label: "Kategorie",
                edm_type: "Edm.String",
                max_length: Some(40),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Supplier",
                label: "Lieferant",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Status",
                label: "Status",
                edm_type: "Edm.String",
                max_length: Some(1),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "StatusCriticality",
                label: "Kritikalitaet",
                edm_type: "Edm.Byte",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Price",
                label: "Preis",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(15),
                scale: Some(2),
            },
            FieldDef {
                name: "Currency",
                label: "Waehrung",
                edm_type: "Edm.String",
                max_length: Some(3),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "UnitsInStock",
                label: "Lagerbestand",
                edm_type: "Edm.Int32",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Rating",
                label: "Bewertung",
                edm_type: "Edm.Byte",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "CreatedAt",
                label: "Erstellt am",
                edm_type: "Edm.DateTimeOffset",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Description",
                label: "Beschreibung",
                edm_type: "Edm.String",
                max_length: Some(500),
                precision: None,
                scale: None,
            },
        ];
        Some(FIELDS)
    }

    fn entity_set(&self) -> String {
        format!(
            r#"<EntitySet Name="Products" EntityType="{}.Product"/>"#,
            NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &["Category", "Status", "Supplier"],
            line_item: &[
                LineItemField {
                    name: "ProductID",
                    importance: Some("High"),
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "ProductName",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Category",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Supplier",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Price",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "UnitsInStock",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Rating",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Status",
                    importance: None,
                    criticality_path: Some("StatusCriticality"),
                    navigation_path: None,
                },
            ],
            header_info: HeaderInfoDef {
                type_name: "Produkt",
                type_name_plural: "Produkte",
                title_path: "ProductName",
                description_path: "ProductID",
            },
            header_facets: &[
                HeaderFacetDef {
                    data_point_qualifier: "Price",
                    label: "Preis",
                },
                HeaderFacetDef {
                    data_point_qualifier: "Stock",
                    label: "Lagerbestand",
                },
                HeaderFacetDef {
                    data_point_qualifier: "Rating",
                    label: "Bewertung",
                },
            ],
            data_points: &[
                DataPointDef {
                    qualifier: "Price",
                    value_path: "Price",
                    title: "Preis",
                    max_value: None,
                    visualization: None,
                },
                DataPointDef {
                    qualifier: "Stock",
                    value_path: "UnitsInStock",
                    title: "Lagerbestand",
                    max_value: None,
                    visualization: None,
                },
                DataPointDef {
                    qualifier: "Rating",
                    value_path: "Rating",
                    title: "Bewertung",
                    max_value: Some(5),
                    visualization: Some("Rating"),
                },
            ],
            facet_sections: &[
                FacetSectionDef {
                    label: "Allgemeine Informationen",
                    id: "GeneralInfo",
                    field_group_qualifier: "General",
                    field_group_label: "Produktdetails",
                },
                FacetSectionDef {
                    label: "Preis &amp; Bestand",
                    id: "PriceStock",
                    field_group_qualifier: "Pricing",
                    field_group_label: "Preisdetails",
                },
            ],
            field_groups: &[
                FieldGroupDef {
                    qualifier: "General",
                    fields: &[
                        "ProductID",
                        "ProductName",
                        "Category",
                        "Supplier",
                        "Status",
                        "CreatedAt",
                        "Description",
                    ],
                },
                FieldGroupDef {
                    qualifier: "Pricing",
                    fields: &["Price", "Currency", "UnitsInStock", "Rating"],
                },
            ],
        };
        Some(&DEF)
    }
}

// ── OrderEntity ─────────────────────────────────────────────────────
struct OrderEntity;

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
            FieldDef {
                name: "OrderID",
                label: "Bestell-Nr.",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "ProductID",
                label: "Produkt-ID",
                edm_type: "Edm.String",
                max_length: Some(10),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "CustomerName",
                label: "Kunde",
                edm_type: "Edm.String",
                max_length: Some(80),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Quantity",
                label: "Menge",
                edm_type: "Edm.Int32",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "TotalAmount",
                label: "Gesamtbetrag",
                edm_type: "Edm.Decimal",
                max_length: None,
                precision: Some(15),
                scale: Some(2),
            },
            FieldDef {
                name: "Currency",
                label: "Waehrung",
                edm_type: "Edm.String",
                max_length: Some(3),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Status",
                label: "Status",
                edm_type: "Edm.String",
                max_length: Some(1),
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "StatusCriticality",
                label: "Kritikalitaet",
                edm_type: "Edm.Byte",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "OrderDate",
                label: "Bestelldatum",
                edm_type: "Edm.DateTimeOffset",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "DeliveryDate",
                label: "Lieferdatum",
                edm_type: "Edm.DateTimeOffset",
                max_length: None,
                precision: None,
                scale: None,
            },
            FieldDef {
                name: "Note",
                label: "Notiz",
                edm_type: "Edm.String",
                max_length: Some(500),
                precision: None,
                scale: None,
            },
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
            "        <EntitySet Name=\"Orders\" EntityType=\"{ns}.Order\">\n\
             \x20         <NavigationPropertyBinding Path=\"Product\" Target=\"Products\"/>\n\
             \x20       </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        static DEF: AnnotationsDef = AnnotationsDef {
            selection_fields: &["Status", "CustomerName", "ProductID"],
            line_item: &[
                LineItemField {
                    name: "OrderID",
                    importance: Some("High"),
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "ProductID",
                    importance: None,
                    criticality_path: None,
                    navigation_path: Some("Product"),
                },
                LineItemField {
                    name: "CustomerName",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Quantity",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "TotalAmount",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "OrderDate",
                    importance: None,
                    criticality_path: None,
                    navigation_path: None,
                },
                LineItemField {
                    name: "Status",
                    importance: None,
                    criticality_path: Some("StatusCriticality"),
                    navigation_path: None,
                },
            ],
            header_info: HeaderInfoDef {
                type_name: "Bestellung",
                type_name_plural: "Bestellungen",
                title_path: "OrderID",
                description_path: "CustomerName",
            },
            header_facets: &[
                HeaderFacetDef {
                    data_point_qualifier: "TotalAmount",
                    label: "Gesamtbetrag",
                },
                HeaderFacetDef {
                    data_point_qualifier: "Quantity",
                    label: "Menge",
                },
            ],
            data_points: &[
                DataPointDef {
                    qualifier: "TotalAmount",
                    value_path: "TotalAmount",
                    title: "Gesamtbetrag",
                    max_value: None,
                    visualization: None,
                },
                DataPointDef {
                    qualifier: "Quantity",
                    value_path: "Quantity",
                    title: "Menge",
                    max_value: None,
                    visualization: None,
                },
            ],
            facet_sections: &[
                FacetSectionDef {
                    label: "Bestelldetails",
                    id: "OrderDetails",
                    field_group_qualifier: "OrderInfo",
                    field_group_label: "Informationen",
                },
                FacetSectionDef {
                    label: "Lieferung",
                    id: "Delivery",
                    field_group_qualifier: "Delivery",
                    field_group_label: "Lieferdetails",
                },
            ],
            field_groups: &[
                FieldGroupDef {
                    qualifier: "OrderInfo",
                    fields: &[
                        "OrderID",
                        "ProductID",
                        "CustomerName",
                        "Quantity",
                        "TotalAmount",
                        "Currency",
                        "Status",
                    ],
                },
                FieldGroupDef {
                    qualifier: "Delivery",
                    fields: &["OrderDate", "DeliveryDate", "Note"],
                },
            ],
        };
        Some(&DEF)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Entity-Registry
// ═══════════════════════════════════════════════════════════════════════

/// Alle Entitaeten registrieren – neues Struct hier einfuegen.
static ENTITIES: &[&dyn ODataEntity] = &[&ProductEntity, &OrderEntity];

/// Gesamtzustand der Applikation – haelt vorberechnete Artefakte
/// (Metadata-XML, manifest.json, FLP-Sandbox-HTML) und die Entity-Registry.
struct AppState {
    entities: &'static [&'static dyn ODataEntity],
    metadata_xml: String,
    manifest_json: String,
    flp_sandbox_html: String,
}

impl AppState {
    fn new(entities: &'static [&'static dyn ODataEntity]) -> Self {
        let metadata_xml = build_metadata_xml(entities);
        let manifest_json =
            serde_json::to_string_pretty(&build_manifest_json(entities)).unwrap_or_default();
        let flp_sandbox_html = build_flp_sandbox_html(entities);
        Self {
            entities,
            metadata_xml,
            manifest_json,
            flp_sandbox_html,
        }
    }

    fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities.iter().find(|e| e.set_name() == set_name).copied()
    }
}

/// Extrahiert den EntitySet-Namen aus dem Request-Pfad.
fn extract_set_name(path: &str) -> Option<&str> {
    let after_base = path.strip_prefix(BASE_PATH)?.trim_start_matches('/');
    let set_part = after_base.split('/').next().unwrap_or("");
    let set_name = set_part.split('(').next().unwrap_or(set_part);
    if set_name.is_empty() {
        None
    } else {
        Some(set_name)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// EDMX Metadata Builder
// ═══════════════════════════════════════════════════════════════════════

/// Baut das komplette EDMX-Dokument aus allen registrierten Entitaeten.
fn build_metadata_xml(entities: &[&dyn ODataEntity]) -> String {
    let entity_types: String = entities
        .iter()
        .map(|e| e.entity_type())
        .collect::<Vec<_>>()
        .join("\n");
    let entity_sets: String = entities
        .iter()
        .map(|e| e.entity_set())
        .collect::<Vec<_>>()
        .join("\n");
    let annotations: String = entities
        .iter()
        .map(|e| e.annotations())
        .filter(|a| !a.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<edmx:Edmx Version="4.0" xmlns:edmx="http://docs.oasis-open.org/odata/ns/edmx">

  <edmx:Reference Uri="https://oasis-tcs.github.io/odata-vocabularies/vocabularies/Org.OData.Capabilities.V1.xml">
    <edmx:Include Namespace="Org.OData.Capabilities.V1" Alias="Capabilities"/>
  </edmx:Reference>
  <edmx:Reference Uri="https://sap.github.io/odata-vocabularies/vocabularies/UI.xml">
    <edmx:Include Namespace="com.sap.vocabularies.UI.v1" Alias="UI"/>
  </edmx:Reference>
  <edmx:Reference Uri="https://sap.github.io/odata-vocabularies/vocabularies/Common.xml">
    <edmx:Include Namespace="com.sap.vocabularies.Common.v1" Alias="Common"/>
  </edmx:Reference>

  <edmx:DataServices>
    <Schema Namespace="{ns}" xmlns="http://docs.oasis-open.org/odata/ns/edm">

{entity_types}

      <EntityContainer Name="EntityContainer">
{entity_sets}
      </EntityContainer>

{annotations}

    </Schema>
  </edmx:DataServices>
</edmx:Edmx>"#,
        ns = NAMESPACE,
        entity_types = entity_types,
        entity_sets = entity_sets,
        annotations = annotations,
    )
}

// ═══════════════════════════════════════════════════════════════════════
// Manifest Builder – dynamisches manifest.json aus Entity-Registry
// ═══════════════════════════════════════════════════════════════════════

/// Baut das komplette manifest.json dynamisch aus allen registrierten Entitaeten.
fn build_manifest_json(entities: &[&dyn ODataEntity]) -> Value {
    let mut routes = Vec::new();
    let mut targets = serde_json::Map::new();
    let mut inbounds = serde_json::Map::new();

    for entity in entities.iter() {
        routes.extend(entity.manifest_routes());
        for (key, val) in entity.manifest_targets() {
            targets.insert(key, val);
        }
        let (inbound_key, inbound_val) = entity.manifest_inbound();
        inbounds.insert(inbound_key, inbound_val);
    }

    json!({
        "_version": "1.65.0",
        "sap.app": {
            "id": "products.demo",
            "type": "application",
            "applicationVersion": {
                "version": "1.0.0"
            },
            "title": "Produkte",
            "description": "Fiori Elements List Report + Object Page",
            "crossNavigation": {
                "inbounds": inbounds
            },
            "dataSources": {
                "mainService": {
                    "uri": format!("{}/", BASE_PATH),
                    "type": "OData",
                    "settings": {
                        "odataVersion": "4.0"
                    }
                }
            }
        },
        "sap.ui5": {
            "flexEnabled": false,
            "dependencies": {
                "minUI5Version": "1.120.0",
                "libs": {
                    "sap.ui.core": {},
                    "sap.fe.templates": {},
                    "sap.f": {}
                }
            },
            "models": {
                "": {
                    "dataSource": "mainService",
                    "settings": {
                        "operationMode": "Server",
                        "autoExpandSelect": true,
                        "earlyRequests": true
                    }
                }
            },
            "routing": {
                "config": {
                    "flexibleColumnLayout": {
                        "defaultTwoColumnLayoutType": "TwoColumnsMidExpanded",
                        "defaultThreeColumnLayoutType": "ThreeColumnsMidExpanded"
                    }
                },
                "routes": routes,
                "targets": targets
            }
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════
// FLP Sandbox Builder – dynamische flpSandbox.html aus Entity-Registry
// ═══════════════════════════════════════════════════════════════════════

/// Baut die flpSandbox.html dynamisch mit Kacheln fuer jede registrierte Entitaet.
fn build_flp_sandbox_html(entities: &[&dyn ODataEntity]) -> String {
    let mut apps = String::new();
    for (i, entity) in entities.iter().enumerate() {
        if i > 0 {
            apps.push_str(",\n");
        }
        apps.push_str(&format!(
            r#"					"{key}": {{
						additionalInformation: "SAPUI5.Component=products.demo",
						applicationType: "URL",
						url: "../",
						title: "{title}",
						description: "Fiori Elements Demo"
					}}"#,
            key = entity.manifest_inbound_key(),
            title = entity.tile_title(),
        ));
    }

    format!(
        r##"<!doctype html>
<html>
	<head>
		<meta http-equiv="X-UA-Compatible" content="IE=edge" />
		<meta http-equiv="Content-Type" content="text/html;charset=UTF-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1.0" />
		<title>Fiori Launchpad Sandbox</title>
		<link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
		<script type="text/javascript">
			window["sap-ushell-config"] = {{
				defaultRenderer: "fiori2",
				renderers: {{
					fiori2: {{
						componentData: {{
							config: {{
								enableSearch: false
							}}
						}}
					}}
				}},
				services: {{
					AppState: {{
						config: {{
							transient: false
						}}
					}}
				}},
				applications: {{
{apps}
				}}
			}};
		</script>

		<script src="https://ui5.sap.com/1.120.0/test-resources/sap/ushell/bootstrap/sandbox.js" id="sap-ushell-bootstrap"></script>

		<!-- Bootstrap the UI5 core library -->
		<script
			id="sap-ui-bootstrap"
			src="https://ui5.sap.com/1.120.0/resources/sap-ui-core.js"
			data-sap-ui-libs="sap.m, sap.ushell, sap.fe.templates"
			data-sap-ui-async="true"
			data-sap-ui-preload="async"
			data-sap-ui-theme="sap_horizon"
			data-sap-ui-bindingSyntax="complex"
			data-sap-ui-compatVersion="edge"
			data-sap-ui-language="de"
			data-sap-ui-resourceroots='{{
				"products.demo": "../"
			}}'
			data-sap-ui-flexibilityServices='[{{"connector": "SessionStorageConnector"}}]'
		></script>

		<script>
			sap.ui.require(["sap/ui/core/Core"], function(Core) {{
				Core.ready(function() {{
					sap.ushell.Container.createRenderer("fiori2", true).then(function(oRenderer) {{
						oRenderer.placeAt("content");
					}});
				}});
			}});
		</script>
	</head>

	<!-- UI Content -->
	<body class="sapUiBody" id="content"></body>
</html>"##,
        apps = apps
    )
}

// ═══════════════════════════════════════════════════════════════════════
// Query helpers
// ═══════════════════════════════════════════════════════════════════════
fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if query.is_empty() {
        return map;
    }
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(
                urlencoding::decode(k).unwrap_or_default().into_owned(),
                urlencoding::decode(v).unwrap_or_default().into_owned(),
            );
        }
    }
    map
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    if let (Some(a_f), Some(b_f)) = (value_as_f64(a), value_as_f64(b)) {
        a_f.partial_cmp(&b_f).unwrap_or(std::cmp::Ordering::Equal)
    } else {
        let a_s = a.as_str().unwrap_or("");
        let b_s = b.as_str().unwrap_or("");
        a_s.cmp(b_s)
    }
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn match_filter(record: &Value, expr: &str) -> bool {
    static FILTER_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)(\w+)\s+(eq|ne|gt|ge|lt|le)\s+(.+)").unwrap());
    static AND_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)\s+and\s+").unwrap());

    let parts: Vec<&str> = AND_RE.split(expr).collect();

    for part in parts {
        let part = part.trim();
        if let Some(caps) = FILTER_RE.captures(part) {
            let field = &caps[1];
            let op = caps[2].to_lowercase();
            let raw_val = caps[3].trim();

            let record_obj = match record.as_object() {
                Some(o) => o,
                None => return false,
            };
            let record_val = match record_obj.get(field) {
                Some(v) if !v.is_null() => v,
                _ => return false,
            };

            let filter_val: Value = if raw_val.starts_with('\'') && raw_val.ends_with('\'') {
                Value::String(raw_val[1..raw_val.len() - 1].to_string())
            } else if let Ok(i) = raw_val.parse::<i64>() {
                Value::Number(i.into())
            } else if let Ok(f) = raw_val.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::String(raw_val.to_string()))
            } else {
                Value::String(raw_val.to_string())
            };

            let cmp = compare_values(record_val, &filter_val);
            let ok = match op.as_str() {
                "eq" => cmp == std::cmp::Ordering::Equal,
                "ne" => cmp != std::cmp::Ordering::Equal,
                "gt" => cmp == std::cmp::Ordering::Greater,
                "ge" => cmp != std::cmp::Ordering::Less,
                "lt" => cmp == std::cmp::Ordering::Less,
                "le" => cmp != std::cmp::Ordering::Greater,
                _ => true,
            };
            if !ok {
                return false;
            }
        }
    }
    true
}

/// Fuehrt eine OData-Abfrage auf den Mock-Daten einer Entitaet aus
/// ($filter, $orderby, $skip, $top, $expand, $select, $count).
fn query_collection(entity: &dyn ODataEntity, qs: &HashMap<String, String>, entities: &[&dyn ODataEntity]) -> Value {
    let mut results = entity.mock_data();

    // $filter
    if let Some(filter_expr) = qs.get("$filter") {
        if !filter_expr.is_empty() {
            results.retain(|r| match_filter(r, filter_expr));
        }
    }

    // $orderby
    if let Some(orderby) = qs.get("$orderby") {
        if !orderby.is_empty() {
            let parts: Vec<&str> = orderby.split_whitespace().collect();
            let field = parts[0];
            let desc = parts
                .get(1)
                .map(|s| s.eq_ignore_ascii_case("desc"))
                .unwrap_or(false);
            results.sort_by(|a, b| {
                let va = a.get(field).unwrap_or(&Value::Null);
                let vb = b.get(field).unwrap_or(&Value::Null);
                let cmp = compare_values(va, vb);
                if desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
    }

    let total = results.len();

    // $skip / $top
    let skip: usize = qs.get("$skip").and_then(|s| s.parse().ok()).unwrap_or(0);
    let top: usize = qs
        .get("$top")
        .and_then(|s| s.parse().ok())
        .unwrap_or(results.len());
    results = results.into_iter().skip(skip).take(top).collect();

    // $expand
    if let Some(expand) = qs.get("$expand") {
        if !expand.is_empty() {
            let nav_names: Vec<&str> = expand.split(',').map(|s| s.trim()).collect();
            for r in &mut results {
                entity.expand_record(r, &nav_names, entities);
            }
        }
    }

    // $select
    if let Some(select) = qs.get("$select") {
        if !select.is_empty() {
            let fields: Vec<&str> = select.split(',').map(|s| s.trim()).collect();
            results = results
                .into_iter()
                .map(|r| {
                    if let Some(obj) = r.as_object() {
                        let filtered: serde_json::Map<String, Value> = obj
                            .iter()
                            .filter(|(k, _)| fields.contains(&k.as_str()))
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        Value::Object(filtered)
                    } else {
                        r
                    }
                })
                .collect();
        }
    }

    let mut body = json!({
        "@odata.context": format!("{}/$metadata#{}", BASE_PATH, entity.set_name()),
        "value": results
    });

    if qs
        .get("$count")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        body["@odata.count"] = json!(total);
    }

    body
}

// ═══════════════════════════════════════════════════════════════════════
// HTTP Handlers (generisch ueber Entity-Registry)
// ═══════════════════════════════════════════════════════════════════════
fn cors_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Accept, Authorization, OData-Version, OData-MaxVersion, X-Requested-With"),
        ("Access-Control-Expose-Headers", "OData-Version"),
    ]
}

fn json_response(data: Value) -> HttpResponse {
    let body = serde_json::to_string_pretty(&data).unwrap_or_default();
    let mut builder = HttpResponse::Ok();
    builder.insert_header((
        "Content-Type",
        "application/json;odata.metadata=minimal;charset=utf-8",
    ));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(body)
}

fn error_response(code: u16, message: &str) -> HttpResponse {
    let body = json!({"error": {"code": code.to_string(), "message": message}});
    let mut builder = match code {
        404 => HttpResponse::NotFound(),
        405 => HttpResponse::MethodNotAllowed(),
        400 => HttpResponse::BadRequest(),
        403 => HttpResponse::Forbidden(),
        _ => HttpResponse::InternalServerError(),
    };
    builder.insert_header(("Content-Type", "application/json;charset=utf-8"));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.json(body)
}

async fn options_handler() -> HttpResponse {
    let mut builder = HttpResponse::Ok();
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.finish()
}

async fn metadata_handler(data: web::Data<AppState>) -> HttpResponse {
    let mut builder = HttpResponse::Ok();
    builder.insert_header(("Content-Type", "application/xml;charset=utf-8"));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(data.metadata_xml.clone())
}

/// Service-Dokument – wird dynamisch aus der Entity-Registry erzeugt.
async fn service_document(data: web::Data<AppState>) -> HttpResponse {
    let sets: Vec<Value> = data.entities
        .iter()
        .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
        .collect();
    json_response(json!({
        "@odata.context": format!("{}/$metadata", BASE_PATH),
        "value": sets
    }))
}

/// Generischer Collection-Handler fuer beliebige EntitySets.
async fn collection_handler(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(set_name) = extract_set_name(req.path()) {
        if let Some(entity) = data.find_entity(set_name) {
            let qs = parse_query_string(req.query_string());
            return json_response(query_collection(entity, &qs, data.entities));
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer $count-Handler fuer beliebige EntitySets.
async fn count_handler(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(set_name) = extract_set_name(req.path()) {
        if let Some(entity) = data.find_entity(set_name) {
            let mut builder = HttpResponse::Ok();
            builder.insert_header(("Content-Type", "text/plain;charset=utf-8"));
            builder.insert_header(("OData-Version", "4.0"));
            for (k, v) in cors_headers() {
                builder.insert_header((k, v));
            }
            return builder.body(entity.mock_data().len().to_string());
        }
    }
    error_response(404, "Entity set not found")
}

/// Generischer Single-Entity-Handler: /SetName('key')
async fn single_entity_handler(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let path = req.path();
    let qs = parse_query_string(req.query_string());

    for entity in state.entities.iter() {
        let prefix = format!("{}/{}('", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            if let Some(key_value) = rest.strip_suffix("')") {
                let data = entity.mock_data();
                if let Some(record) = data
                    .iter()
                    .find(|r| r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value))
                {
                    let mut result = record.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(
                            "@odata.context".to_string(),
                            json!(format!(
                                "{}/$metadata#{}/$entity",
                                BASE_PATH,
                                entity.set_name()
                            )),
                        );
                    }
                    // $expand
                    if let Some(expand) = qs.get("$expand") {
                        if !expand.is_empty() {
                            let nav_names: Vec<&str> =
                                expand.split(',').map(|s| s.trim()).collect();
                            entity.expand_record(&mut result, &nav_names, state.entities);
                        }
                    }
                    return json_response(result);
                }
                return error_response(
                    404,
                    &format!(
                        "Entity with {}='{}' not found.",
                        entity.key_field(),
                        key_value
                    ),
                );
            }
        }
    }
    error_response(404, "Entity not found.")
}

// ── $batch handler ──────────────────────────────────────────────────
async fn batch_handler(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let batch_boundary = content_type
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("boundary=")
        })
        .unwrap_or("");

    if batch_boundary.is_empty() {
        return error_response(400, "Missing batch boundary");
    }

    let raw_body = String::from_utf8_lossy(&body);
    let mut response_parts = Vec::new();
    let separator = format!("--{}", batch_boundary);

    for segment in raw_body.split(&separator) {
        let segment = segment.trim();
        if segment.is_empty() || segment == "--" {
            continue;
        }

        if segment.contains("multipart/mixed") {
            let cs_boundary = segment
                .lines()
                .find_map(|line| {
                    if line.contains("boundary=") {
                        line.split(';').find_map(|tok| {
                            let tok = tok.trim();
                            tok.strip_prefix("boundary=")
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            if !cs_boundary.is_empty() {
                let cs_resp = format!("--{}--\r\n", cs_boundary);
                let part_resp = format!(
                    "Content-Type: multipart/mixed; boundary={}\r\nContent-Length: {}\r\n\r\n{}",
                    cs_boundary,
                    cs_resp.len(),
                    cs_resp
                );
                response_parts.push(part_resp);
            }
            continue;
        }

        let lines: Vec<&str> = segment.lines().collect();
        let request_line = lines.iter().find(|l| l.starts_with("GET "));
        if let Some(request_line) = request_line {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let rel_url = parts.get(1).copied().unwrap_or("");
            let resp_json = handle_batch_get(rel_url, data.entities);
            let resp_body = serde_json::to_string(&resp_json).unwrap_or_default();

            let part_resp = format!(
                "Content-Type: application/http\r\n\
                 Content-Transfer-Encoding: binary\r\n\
                 \r\n\
                 HTTP/1.1 200 OK\r\n\
                 Content-Type: application/json;odata.metadata=minimal;charset=utf-8\r\n\
                 OData-Version: 4.0\r\n\
                 Content-Length: {}\r\n\
                 \r\n\
                 {}",
                resp_body.len(),
                resp_body
            );
            response_parts.push(part_resp);
        }
    }

    let resp_boundary = format!("batch_resp_{}", std::process::id());
    let mut body_parts = Vec::new();
    for rp in &response_parts {
        body_parts.push(format!("--{}\r\n{}", resp_boundary, rp));
    }
    body_parts.push(format!("--{}--\r\n", resp_boundary));
    let full_body = body_parts.join("\r\n");

    let mut builder = HttpResponse::Ok();
    builder.insert_header((
        "Content-Type",
        format!("multipart/mixed; boundary={}", resp_boundary),
    ));
    builder.insert_header(("OData-Version", "4.0"));
    for (k, v) in cors_headers() {
        builder.insert_header((k, v));
    }
    builder.body(full_body)
}

/// Generischer Batch-GET – loest Pfade ueber die Entity-Registry auf.
fn handle_batch_get(rel_url: &str, entities: &[&dyn ODataEntity]) -> Value {
    let full_path = if rel_url.starts_with('/') {
        rel_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, rel_url)
    };

    let (path_part, query_part) = full_path.split_once('?').unwrap_or((&full_path, ""));
    let path = path_part.trim_end_matches('/');
    let qs = parse_query_string(query_part);

    // Service root
    if path == BASE_PATH {
        let sets: Vec<Value> = entities
            .iter()
            .map(|e| json!({"name": e.set_name(), "url": e.set_name()}))
            .collect();
        return json!({
            "@odata.context": format!("{}/$metadata", BASE_PATH),
            "value": sets
        });
    }

    // Iterate entities for collection, $count, and single-entity routes
    for entity in entities.iter() {
        let set_path = format!("{}/{}", BASE_PATH, entity.set_name());
        let count_path = format!("{}/$count", set_path);

        // Collection
        if path == set_path {
            return query_collection(*entity, &qs, entities);
        }

        // $count
        if path == count_path {
            return json!({"value": entity.mock_data().len()});
        }

        // Single entity: /SetName('key')
        let prefix = format!("{}('", set_path);
        if let Some(rest) = path.strip_prefix(&prefix) {
            if let Some(key_value) = rest.strip_suffix("')") {
                let data = entity.mock_data();
                if let Some(record) = data
                    .iter()
                    .find(|r| r.get(entity.key_field()).and_then(|v| v.as_str()) == Some(key_value))
                {
                    let mut result = record.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(
                            "@odata.context".to_string(),
                            json!(format!(
                                "{}/$metadata#{}/$entity",
                                BASE_PATH,
                                entity.set_name()
                            )),
                        );
                    }
                    if let Some(expand) = qs.get("$expand") {
                        if !expand.is_empty() {
                            let nav_names: Vec<&str> =
                                expand.split(',').map(|s| s.trim()).collect();
                            entity.expand_record(&mut result, &nav_names, entities);
                        }
                    }
                    return result;
                }
                return json!({"error": {"code": "404", "message": "Not found"}});
            }
        }
    }

    json!({"error": {"code": "404", "message": format!("Unknown: {}", rel_url)}})
}

// ── Static file serving ─────────────────────────────────────────────
async fn static_files(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let raw_path = urlencoding::decode(req.path())
        .unwrap_or_default()
        .into_owned();
    let mut relative = raw_path.trim_start_matches('/').to_string();

    for prefix in &["products/demo/", "products.demo/"] {
        if relative.starts_with(prefix) {
            relative = relative[prefix.len()..].to_string();
            break;
        }
    }
    if relative.is_empty() {
        relative = "index.html".to_string();
    }

    // manifest.json wird dynamisch aus der Entity-Registry generiert
    if relative == "manifest.json" {
        let mut builder = HttpResponse::Ok();
        builder.insert_header(("Content-Type", "application/json;charset=utf-8"));
        for (k, v) in cors_headers() {
            builder.insert_header((k, v));
        }
        return builder.body(data.manifest_json.clone());
    }

    // flpSandbox.html wird dynamisch generiert (Kacheln pro Entitaet)
    if relative == "test/flpSandbox.html" {
        let mut builder = HttpResponse::Ok();
        builder.insert_header(("Content-Type", "text/html;charset=utf-8"));
        for (k, v) in cors_headers() {
            builder.insert_header((k, v));
        }
        return builder.body(data.flp_sandbox_html.clone());
    }

    let wa_dir = webapp_dir();
    if !wa_dir.exists() {
        return error_response(404, "webapp directory not found");
    }

    let candidate = wa_dir.join(&relative);
    // Path traversal protection
    let canonical = match candidate.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            if Path::new(&relative).extension().is_none() {
                let index = wa_dir.join("index.html");
                if index.exists() {
                    return serve_file(&index);
                }
            }
            return error_response(404, &format!("Resource not found: {}", raw_path));
        }
    };
    let wa_canonical = match wa_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return error_response(403, "Access denied."),
    };
    if !canonical.starts_with(&wa_canonical) {
        return error_response(403, "Access denied.");
    }

    let target = if canonical.is_dir() {
        canonical.join("index.html")
    } else {
        canonical
    };

    if target.exists() && target.is_file() {
        return serve_file(&target);
    }

    if Path::new(&relative).extension().is_none() {
        let index = wa_dir.join("index.html");
        if index.exists() {
            return serve_file(&index);
        }
    }

    error_response(404, &format!("Resource not found: {}", raw_path))
}

fn serve_file(path: &Path) -> HttpResponse {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    match std::fs::read(path) {
        Ok(bytes) => HttpResponse::Ok()
            .insert_header(("Content-Type", mime.to_string()))
            .body(bytes),
        Err(_) => error_response(500, "Failed to read file"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = "0.0.0.0";
    let port = 8000u16;

    println!("{}", "=".repeat(60));
    println!("{}", "=".repeat(60));
    println!("  Web App      : http://localhost:{}/", port);
    println!("  Service Root : http://localhost:{}{}", port, BASE_PATH);
    println!(
        "  $metadata    : http://localhost:{}{}/$metadata",
        port, BASE_PATH
    );
    println!(
        "  manifest     : http://localhost:{}/manifest.json (dynamisch)",
        port
    );
    println!(
        "  Products     : http://localhost:{}{}/Products",
        port, BASE_PATH
    );
    println!(
        "  Single Item  : http://localhost:{}{}/Products('P001')",
        port, BASE_PATH
    );
    println!("{}", "=".repeat(60));
    println!("  Druecke Ctrl+C zum Beenden\n");

    let app_state = web::Data::new(AppState::new(ENTITIES));

    HttpServer::new(move || {
        let base = BASE_PATH;

        let mut app = App::new()
            .app_data(app_state.clone())
            .route(
                &format!("{}/$metadata", base),
                web::get().to(metadata_handler),
            )
            .route(
                &format!("{}/$metadata", base),
                web::method(actix_web::http::Method::OPTIONS).to(options_handler),
            )
            .route(&format!("{}/", base), web::get().to(service_document))
            .route(base, web::get().to(service_document))
            .route(&format!("{}/$batch", base), web::post().to(batch_handler))
            .route(
                &format!("{}/$batch", base),
                web::method(actix_web::http::Method::OPTIONS).to(options_handler),
            );

        // Routen fuer jedes registrierte EntitySet dynamisch erzeugen
        for entity in app_state.entities.iter() {
            let set = entity.set_name();
            app = app
                .route(
                    &format!("{}/{}", base, set),
                    web::get().to(collection_handler),
                )
                .route(
                    &format!("{}/{}/$count", base, set),
                    web::get().to(count_handler),
                );
        }

        app.default_service(web::route().to(catch_all))
    })
    .bind((host, port))?
    .run()
    .await
}

// ── Favicon: Dänischer Leuchtturm (SVG) ────────────────────────────
fn favicon_svg() -> &'static str {
    // Rot-weiss abwechselnd gestreifter Leuchtturm (Rubjerg Knude / Lyngvig Stil)
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#1a3a5c"/>
      <stop offset="100%" stop-color="#4a90c4"/>
    </linearGradient>
    <clipPath id="tower">
      <polygon points="26,56 22,22 42,22 38,56"/>
    </clipPath>
  </defs>
  <!-- Himmel -->
  <rect width="64" height="64" rx="12" fill="url(#sky)"/>
  <!-- Duene / Sand -->
  <ellipse cx="32" cy="60" rx="38" ry="10" fill="#d4a84b"/>
  <!-- Turm: abwechselnd rot/weiss, geclippt auf Turmform -->
  <g clip-path="url(#tower)">
    <rect x="20" y="22" width="24" height="34" fill="#ffffff"/>
    <rect x="20" y="22" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="32" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="42" width="24" height="5"  fill="#c0392b"/>
    <rect x="20" y="52" width="24" height="4"  fill="#c0392b"/>
  </g>
  <!-- Galerie (Balkon) -->
  <rect x="19" y="19" width="26" height="4" rx="1" fill="#2c3e50"/>
  <!-- Laterne (Glashaus) -->
  <rect x="25" y="11" width="14" height="9" rx="2" fill="#f9e784" opacity="0.9"/>
  <rect x="25" y="11" width="14" height="9" rx="2" fill="none" stroke="#2c3e50" stroke-width="1"/>
  <!-- Dach -->
  <polygon points="24,11 32,5 40,11" fill="#2c3e50"/>
  <!-- Lichtstrahl -->
  <polygon points="39,15 58,6 58,12 39,17" fill="#f9e784" opacity="0.35"/>
  <polygon points="25,15 6,6 6,12 25,17" fill="#f9e784" opacity="0.25"/>
  <!-- Tuer -->
  <rect x="29" y="49" width="6" height="7" rx="3" fill="#2c3e50"/>
</svg>"##
}

fn favicon_response() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Content-Type", "image/svg+xml"))
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .body(favicon_svg())
}

async fn catch_all(req: HttpRequest, _body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let path = req.path();

    if req.method() == actix_web::http::Method::OPTIONS {
        return options_handler().await;
    }

    if path == "/favicon.ico" || path == "/favicon.svg" {
        return favicon_response();
    }

    // Single entity: /BASE_PATH/SetName('key') – generisch ueber Registry
    for entity in data.entities.iter() {
        let prefix = format!("{}/{}", BASE_PATH, entity.set_name());
        if let Some(rest) = path.strip_prefix(&prefix) {
            if rest.starts_with("('") && rest.ends_with("')") {
                return single_entity_handler(req, data).await;
            }
        }
    }

    static_files(req, data).await
}
