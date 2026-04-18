use serde_json::{json, Value};

use crate::entity::ODataEntity;
use crate::spec::{self, EntitySpec};
use crate::NAMESPACE;

#[derive(Debug)]
pub struct EntityRelationshipEntity;

impl ODataEntity for EntityRelationshipEntity {
    fn set_name(&self) -> &'static str {
        "EntityRelationships"
    }
    fn type_name(&self) -> &'static str {
        "EntityRelationship"
    }

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(spec::meta_package::entity_relationships())
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"EntityRelationships\" EntityType=\"{ns}.EntityRelationship\">\n\
             <NavigationPropertyBinding Path=\"SiblingEntity\" Target=\"EntityRelationships\"/>\n\
             <NavigationPropertyBinding Path=\"DraftAdministrativeData\" Target=\"DraftAdministrativeData\"/>\n\
             </EntitySet>",
            ns = NAMESPACE
        )
    }

    fn mock_data(&self) -> Vec<Value> {
        vec![]
    }

    fn apps_json_entry(&self) -> Option<(String, Value)> {
        Some(("EntityRelationships-display".to_string(), json!({
            "title": "Relationships",
            "description": "Entity-Beziehungen",
            "icon": "sap-icon://connected",
            "semanticObject": "EntityRelationships",
            "action": "display"
        })))
    }
}
