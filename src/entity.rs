use std::collections::HashMap;
use std::fmt::Debug;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::annotations::{AnnotationsDef, FieldDef, NavigationPropertyDef};
use crate::model::ResolvedEntity;
use crate::spec::EntitySpec;

/// Fixed namespace UUID for deterministic value list IDs (UUID v5).
const VALUE_LIST_NS: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

/// Generates a deterministic UUID v5 for a value list based on its name.
pub fn value_list_id(list_name: &str) -> String {
    Uuid::new_v5(&VALUE_LIST_NS, list_name.as_bytes()).to_string()
}

/// Base trait for an OData entity.
///
/// Adding a new entity:
///   1. Create a new struct, implement the ODataEntity trait
///   2. Implement set_name, key_field, type_name, mock_data, entity_type,
///      entity_set, annotations_def (and optionally expand_record)
///   3. Register the instance in AppStateBuilder via .entity()
pub trait ODataEntity: Sync + Debug {
    /// Name of the EntitySet (e.g. "Products", "Orders")
    fn set_name(&self) -> &'static str;
    /// Name of the key field – always "ID" (Edm.Guid).
    fn key_field(&self) -> &'static str {
        "ID"
    }
    /// Name of the entity type (e.g. "Product", "Order")
    fn type_name(&self) -> &'static str;
    /// Mock data as a JSON array
    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }
    /// Unified field definitions – a single list for EntityType AND annotations.
    fn fields_def(&self) -> Option<&'static [FieldDef]> {
        None
    }
    /// NavigationProperty-Definitionen (optional).
    fn navigation_properties(&self) -> &'static [NavigationPropertyDef] {
        &[]
    }
    /// Parent EntitySet for compositions (e.g. "Orders" for OrderItems).
    fn parent_set_name(&self) -> Option<&'static str> {
        None
    }

    // ── Layer 1 spec integration ────────────────────────────────────

    /// Layer 1 entity specification (optional).
    /// When provided, the resolve→generate pipeline can produce entity_type,
    /// entity_set, and annotations automatically from specs + relationships.
    fn entity_spec(&self) -> Option<EntitySpec> {
        None
    }

    /// Customize the resolved entity before XML generation.
    /// Called after the resolver produces a ResolvedEntity from specs + relationships.
    /// Override to add entity-specific tweaks (e.g., extra nav properties, custom facets).
    fn tweak_resolved(&self, _resolved: &mut ResolvedEntity) {}

    /// Default values for new entities (e.g. Currency="EUR", Status="A").
    /// Applied before type defaults when creating a new draft entity.
    fn default_values(&self) -> Option<Value> {
        None
    }
    /// Update computed fields (e.g. TypeName = HeaderTypeName + "Type").
    /// Called after create_entity and patch_entity.
    fn compute_fields(&self, _record: &mut Value) {}
    /// Automatically create child entities on parent creation.
    /// Returns (child_set_name, child_data) pairs.
    /// May mutate parent_record (e.g. set FK reference to the child).
    fn auto_create_children(&self, _parent_record: &mut Value) -> Vec<(String, Value)> {
        vec![]
    }
    /// Primary text field displayed instead of the key.
    /// Default: HeaderInfo.title_path (if available).
    /// Generates Common.Text + UI.TextArrangement on the key field.
    fn title_field(&self) -> Option<&'static str> {
        self.annotations_def().map(|d| d.header_info.title_path)
    }
    /// EDMX EntitySet-XML
    fn entity_set(&self) -> String;
    /// Declarative annotation definition (optional).
    fn annotations_def(&self) -> Option<&'static AnnotationsDef> {
        None
    }
    /// Apply $expand logic to a single record (optional).
    fn expand_record(
        &self,
        _record: &mut Value,
        _nav_properties: &[&str],
        _entities: &[&dyn ODataEntity],
        _data_store: &HashMap<String, Vec<Value>>,
    ) {
    }

    /// Tile title for the FLP (default: type_name_plural from HeaderInfo).
    fn tile_title(&self) -> &str {
        self.annotations_def()
            .map(|d| d.header_info.type_name_plural)
            .unwrap_or(self.set_name())
    }

    /// Optional apps.json entry for the FLP.
    /// Default: None – hardcoded entities are configured via the static apps.json.
    /// GenericEntity provides the tile configuration from the JSON file.
    fn apps_json_entry(&self) -> Option<(String, Value)> {
        None
    }

    /// Manifest crossNavigation inbound key (e.g. "Products-display").
    fn manifest_inbound_key(&self) -> String {
        format!("{}-display", self.set_name())
    }

    /// Manifest crossNavigation inbound entry.
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

    /// Manifest routing: returns the routes for this EntitySet.
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

    /// Manifest routing: returns the targets (ListReport + ObjectPage)
    /// for this EntitySet.
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
