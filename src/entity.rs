use std::collections::HashMap;
use std::fmt::Debug;

use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;

use crate::annotations::*;

/// Fester Namespace-UUID fuer deterministische Value-List-IDs (UUID v5).
const VALUE_LIST_NS: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

/// Erzeugt eine deterministische UUID v5 fuer eine Werteliste anhand ihres Namens.
pub fn value_list_id(list_name: &str) -> String {
    Uuid::new_v5(&VALUE_LIST_NS, list_name.as_bytes()).to_string()
}

/// Basisklasse (Trait) fuer eine OData-Entitaet.
///
/// Neue Entitaet hinzufuegen:
///   1. Neues Struct anlegen, ODataEntity-Trait implementieren
///   2. set_name, key_field, type_name, mock_data, entity_type,
///      entity_set, annotations_def (und ggf. expand_record) implementieren
///   3. Instanz im AppStateBuilder via .entity() registrieren
pub trait ODataEntity: Sync + Debug {
    /// Name des EntitySets (z.B. "Products", "Orders")
    fn set_name(&self) -> &'static str;
    /// Name des Schluesselfelds – immer "ID" (Edm.Guid).
    fn key_field(&self) -> &'static str {
        "ID"
    }
    /// Name des Entity-Typs (z.B. "Product", "Order")
    fn type_name(&self) -> &'static str;
    /// Mock-Daten als JSON-Array
    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }
    /// Einheitliche Feld-Definitionen – eine Liste fuer EntityType UND Annotations.
    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        None
    }
    /// NavigationProperty-Definitionen (optional).
    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        &[]
    }
    /// Eltern-EntitySet bei Kompositionen (z.B. "Orders" fuer OrderItems).
    fn parent_set_name(&self) -> Option<&'static str> {
        None
    }
    /// Standardwerte fuer neue Entitaeten (z.B. Currency="EUR", Status="A").
    /// Werden beim Erstellen einer neuen Draft-Entitaet vor den Typ-Defaults angewendet.
    fn default_values(&self) -> Option<Value> {
        None
    }
    /// Berechnete Felder aktualisieren (z.B. TypeName = HeaderTypeName + "Type").
    /// Wird nach create_entity und patch_entity aufgerufen.
    fn compute_fields(&self, _record: &mut Value) {}
    /// Automatische Kind-Entitaeten bei Neu-Anlage erzeugen.
    /// Gibt (child_set_name, child_data)-Paare zurueck.
    /// Darf parent_record mutieren (z.B. FK-Referenz auf das Kind).
    fn auto_create_children(&self, _parent_record: &mut Value) -> Vec<(String, Value)> {
        vec![]
    }
    /// Primaeres Textfeld, das anstelle des Schluessels angezeigt wird.
    /// Default: HeaderInfo.title_path (falls vorhanden).
    /// Erzeugt Common.Text + UI.TextArrangement auf dem Schluesselfeld.
    fn title_field(&self) -> Option<&'static str> {
        self.annotations_def().map(|d| d.header_info.title_path)
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
    /// Zusaetzliche Annotations-XML (z.B. UI.Identification fuer Custom Actions).
    /// Wird am Ende von annotations() angehaengt.
    fn extra_annotations_xml(&self) -> String {
        String::new()
    }
    /// Zusaetzliche gebundene OData-Actions (z.B. publishConfig).
    /// Wird neben den Draft-Actions ins Schema geschrieben.
    fn custom_actions_xml(&self) -> String {
        String::new()
    }
    /// EDMX Annotations-XML – wird automatisch aus annotations_def() erzeugt.
    fn annotations(&self) -> String {
        let mut xml = match (self.annotations_def(), self.fields_def()) {
            (Some(def), Some(fields)) => {
                info!("building annotations for {} with {} fields", self.set_name(), fields.len());
                build_annotations_xml(self.type_name(), def, fields)
            }
            (Some(def), None) => {
                info!("building annotations for {} without fields", self.set_name());
                build_annotations_xml(self.type_name(), def, &[])
            }
            _ => String::new(),
        };
        // UpdateRestrictions + Immutable-Annotations
        if let Some(fields) = self.fields_def() {
            let is_draft_root = self.parent_set_name().is_none();
            xml.push_str(&build_capabilities_annotations(
                self.set_name(),
                self.type_name(),
                self.key_field(),
                self.title_field(),
                fields,
                is_draft_root,
            ));
        }
        // Zusaetzliche entitaetsspezifische Annotations
        xml.push_str(&self.extra_annotations_xml());
        xml
    }
    /// $expand-Logik auf einen einzelnen Datensatz anwenden (optional).
    fn expand_record(
        &self,
        _record: &mut Value,
        _nav_properties: &[&str],
        _entities: &[&dyn ODataEntity],
        _data_store: &HashMap<String, Vec<Value>>,
    ) {
    }

    /// Tile-Titel fuer das FLP (Default: type_name_plural aus HeaderInfo).
    fn tile_title(&self) -> &str {
        self.annotations_def()
            .map(|d| d.header_info.type_name_plural)
            .unwrap_or(self.set_name())
    }

    /// Optionaler apps.json-Eintrag fuer das FLP.
    /// Default: None – hardkodierte Entitaeten werden ueber die statische apps.json konfiguriert.
    /// GenericEntity liefert die Kachel-Konfiguration aus der JSON-Datei.
    fn apps_json_entry(&self) -> Option<(String, Value)> {
        None
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
    /// fuer dieses EntitySet.
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
                },
                "controlAggregation": "beginColumnPages",
                "contextPattern": ""
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
                    },
                    "controlAggregation": "midColumnPages",
                    "contextPattern": format!("/{}({{key}})", name)
                }),
            ),
        ]
    }
}
