//! OData URL parsing and route resolution.
//!
//! Converts raw request URLs into structured [`ODataPath`] variants that the
//! handlers use for dispatching entity, collection, action, and sub-collection
//! operations.

use crate::entity::ODataEntity;
use crate::model::ResolvedEntity;
use crate::BASE_PATH;

/// Parsed entity key information (key value + IsActiveEntity).
#[derive(Debug)]
pub struct EntityKeyInfo {
    pub key_value: String,
    pub is_active: bool,
}

/// Resolved OData path variants supported by the server.
#[derive(Debug)]
pub enum ODataPath {
    /// Service-Root: /odata/v4/Service
    ServiceRoot,
    /// Collection: /odata/v4/Service/Products
    Collection { entity: ODataEntity },
    /// $count:     /odata/v4/Service/Products/$count
    Count { entity: ODataEntity },
    /// Single Entity: /odata/v4/Service/Products('P001')
    Entity {
        entity: ODataEntity,
        key: EntityKeyInfo,
    },
    /// Sub-Collection: /odata/v4/Service/Orders('O001')/Items
    SubCollection {
        parent_entity: ODataEntity,
        parent_key: EntityKeyInfo,
        child_entity: ODataEntity,
    },
    /// Bound Action:  /odata/v4/Service/Products('P001')/Ns.draftEdit
    Action {
        entity: ODataEntity,
        key: EntityKeyInfo,
        action: String,
    },
    /// Property Access: /odata/v4/Service/Orders('O001')/OrderID
    PropertyAccess {
        entity: ODataEntity,
        key: EntityKeyInfo,
        property: String,
    },
    /// Unrecognized path
    Unknown,
}

/// Result of OData URL parsing: resolved path plus detached query string.
#[derive(Debug)]
pub struct ParsedODataUrl {
    pub path: ODataPath,
    pub query_string: String,
}

/// Resolves a raw URL into a structured [`ODataPath`].
///
/// This helper delegates to [`resolve_odata_path_with_resolved`] with an empty
/// resolved-model input, which is sufficient for legacy trait-defined
/// navigation properties.
///
/// Examples:
///   - `Products`                           → Collection
///   - `Products/$count`                    → Count
///   - `Products('P001')`                   → Entity
///   - `Products('P001')/Ns.draftEdit`      → Action { action: "draftEdit" }
///   - `/odata/v4/Service/Products`         → Collection (absolute)
///   - `Products?$filter=...`               → Collection + query_string
pub fn resolve_odata_path<'a>(raw_url: &str, entities: &'a [ODataEntity]) -> ParsedODataUrl {
    resolve_odata_path_with_resolved(raw_url, entities, &[])
}

/// Resolves a raw URL using both trait-level and resolved-model navigation metadata.
///
/// This is required for spec-driven entities where navigation properties are derived
/// from `Relationship` declarations and may not be present on `ODataEntity::navigation_properties()`.
pub fn resolve_odata_path_with_resolved<'a>(
    raw_url: &str,
    entities: &'a [ODataEntity],
    resolved_entities: &[ResolvedEntity],
) -> ParsedODataUrl {
    // URL decoding (e.g. %27 → ')
    let decoded_url = urlencoding::decode(raw_url).unwrap_or_default();
    let raw_url = decoded_url.as_ref();

    // Normalize relative → absolute
    let full = if raw_url.starts_with('/') {
        raw_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, raw_url)
    };

    // Separate query string
    let (path_part, query_part) = full.split_once('?').unwrap_or((&full, ""));
    let path = path_part.trim_end_matches('/');

    // Service-Root
    if path == BASE_PATH {
        return ParsedODataUrl {
            path: ODataPath::ServiceRoot,
            query_string: query_part.to_string(),
        };
    }

    for entity in entities {
        let set_path = format!("{}/{}", BASE_PATH, entity.set_name());
        let count_path = format!("{}/$count", set_path);

        // Collection
        if path == set_path {
            return ParsedODataUrl {
                path: ODataPath::Collection {
                    entity: entity.clone(),
                },
                query_string: query_part.to_string(),
            };
        }

        // $count
        if path == count_path {
            return ParsedODataUrl {
                path: ODataPath::Count {
                    entity: entity.clone(),
                },
                query_string: query_part.to_string(),
            };
        }

        // Entity or action:  /SetPath(key...) or /SetPath(key...)/Ns.action
        let set_prefix = format!("{}(", set_path);
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            // Action or SubCollection: search for ")/" as separator
            if let Some(paren_end) = rest.find(")/") {
                let key_str = &rest[..paren_end];
                let after_paren = &rest[paren_end + 2..];
                if let Some(key) = parse_key_content(key_str, entity.key_field()) {
                    // First check if after_paren starts with a NavigationProperty
                    let first_segment = after_paren
                        .split(|c: char| c == '(' || c == '/')
                        .next()
                        .unwrap_or(after_paren);
                    let nav_props = entity.navigation_properties();
                    let child_from_trait = nav_props
                        .iter()
                        .find(|np| np.name == first_segment)
                        .and_then(|nav_def| {
                            entities
                                .iter()
                                .find(|e| e.type_name() == nav_def.target_type)
                                .cloned()
                        });

                    let child_from_resolved = resolved_entities
                        .iter()
                        .find(|re| re.set_name == entity.set_name())
                        .and_then(|re| re.nav_properties.iter().find(|np| np.name == first_segment))
                        .and_then(|np| {
                            entities
                                .iter()
                                .find(|e| e.set_name() == np.target_set)
                                .cloned()
                                .or_else(|| {
                                    entities
                                        .iter()
                                        .find(|e| e.type_name() == np.target_type)
                                        .cloned()
                                })
                        });

                    if let Some(child) = child_from_trait.or(child_from_resolved) {
                        // Child entity with key: Items(ItemID='I002',IsActiveEntity=true)
                        if let Some(child_key_start) = after_paren.find('(') {
                            let child_rest = &after_paren[child_key_start + 1..];
                            // Child entity with action: Items(key)/Ns.action
                            if let Some(cp_end) = child_rest.find(")/") {
                                let child_key_str = &child_rest[..cp_end];
                                let child_after = &child_rest[cp_end + 2..];
                                if let Some(child_key) =
                                    parse_key_content(child_key_str, child.key_field())
                                {
                                    if child_after.contains('.') {
                                        let action = child_after
                                            .rsplit('.')
                                            .next()
                                            .unwrap_or(child_after)
                                            .to_string();
                                        return ParsedODataUrl {
                                            path: ODataPath::Action {
                                                entity: child,
                                                key: child_key,
                                                action,
                                            },
                                            query_string: query_part.to_string(),
                                        };
                                    }
                                }
                            }
                            // Child entity without action: Items(key)
                            if let Some(child_key_str) = child_rest.strip_suffix(')') {
                                if let Some(child_key) =
                                    parse_key_content(child_key_str, child.key_field())
                                {
                                    return ParsedODataUrl {
                                        path: ODataPath::Entity {
                                            entity: child,
                                            key: child_key,
                                        },
                                        query_string: query_part.to_string(),
                                    };
                                }
                            }
                        }
                        // Simple sub-collection without child key: Items
                        return ParsedODataUrl {
                            path: ODataPath::SubCollection {
                                parent_entity: entity.clone(),
                                parent_key: key,
                                child_entity: child.clone(),
                            },
                            query_string: query_part.to_string(),
                        };
                    }
                    // Bound action on parent: Ns.actionName (contains '.')
                    if after_paren.contains('.') {
                        let action = after_paren
                            .rsplit('.')
                            .next()
                            .unwrap_or(after_paren)
                            .to_string();
                        return ParsedODataUrl {
                            path: ODataPath::Action {
                                entity: entity.clone(),
                                key,
                                action,
                            },
                            query_string: query_part.to_string(),
                        };
                    }
                    // Property Access: Entity(key)/PropertyName
                    return ParsedODataUrl {
                        path: ODataPath::PropertyAccess {
                            entity: entity.clone(),
                            key,
                            property: after_paren.to_string(),
                        },
                        query_string: query_part.to_string(),
                    };
                }
            }

            // Single entity: strip trailing ')'
            if let Some(key_str) = rest.strip_suffix(')') {
                if let Some(key) = parse_key_content(key_str, entity.key_field()) {
                    return ParsedODataUrl {
                        path: ODataPath::Entity {
                            entity: entity.clone(),
                            key,
                        },
                        query_string: query_part.to_string(),
                    };
                }
            }
        }
    }

    ParsedODataUrl {
        path: ODataPath::Unknown,
        query_string: query_part.to_string(),
    }
}

/// Parses key content inside OData key parentheses.
///
/// Accepts:
///   - `'P001'`                                 → simple key
///   - `ProductID='P001',IsActiveEntity=true`   → composite key
fn parse_key_content(key_str: &str, key_field: &str) -> Option<EntityKeyInfo> {
    // Simple key: 'value'
    if key_str.starts_with('\'') && key_str.ends_with('\'') {
        let value = key_str[1..key_str.len() - 1].to_string();
        return Some(EntityKeyInfo {
            key_value: value,
            is_active: true,
        });
    }

    // Composite key: Key='val',IsActiveEntity=true
    let mut key_value = String::new();
    let mut is_active = true;
    for part in key_str.split(',') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if k == key_field {
                key_value = v.trim_matches('\'').to_string();
            } else if k == "IsActiveEntity" {
                is_active = v.eq_ignore_ascii_case("true");
            }
        }
    }
    if !key_value.is_empty() {
        return Some(EntityKeyInfo {
            key_value,
            is_active,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::entity::ODataEntityImp;
    use crate::model::{ResolvedEntity, ResolvedNavProperty};

    #[derive(Debug)]
    struct OrdersEntity;

    impl ODataEntityImp for OrdersEntity {
        fn set_name(&self) -> &'static str {
            "Orders"
        }

        fn type_name(&self) -> &'static str {
            "Order"
        }

        fn entity_set(&self) -> String {
            String::new()
        }
    }

    #[derive(Debug)]
    struct OrderItemsEntity;

    impl ODataEntityImp for OrderItemsEntity {
        fn set_name(&self) -> &'static str {
            "OrderItems"
        }

        fn type_name(&self) -> &'static str {
            "OrderItem"
        }

        fn entity_set(&self) -> String {
            String::new()
        }
    }

    #[test]
    fn resolves_subcollection_from_resolved_nav_properties() {
        let orders = ODataEntity::new(Arc::new(OrdersEntity));
        let items = ODataEntity::new(Arc::new(OrderItemsEntity));
        let entities: [ODataEntity; 2] = [orders, items];

        // Simulate relationship-derived navs: Orders has nav "Items" to OrderItems.
        // Trait-level navigation_properties() is intentionally empty here.
        let resolved_entities = vec![ResolvedEntity {
            set_name: "Orders".into(),
            type_name: "Order".into(),
            type_name_plural: "Orders".into(),
            key_field: "ID".into(),
            title_field: "Name".into(),
            description_field: None,
            parent_set_name: None,
            properties: vec![],
            nav_properties: vec![ResolvedNavProperty {
                name: "Items".into(),
                target_type: "OrderItem".into(),
                target_set: "OrderItems".into(),
                is_collection: true,
                foreign_key: Some("OrderID".into()),
                relationship: "Order_Items".into(),
                is_composition: true,
            }],
            data_points: vec![],
            header_facets: vec![],
            facet_sections: vec![],
            table_facets: vec![],
            selection_fields: vec![],
            package: None,
            extra_annotations_xml: String::new(),
            custom_actions_xml: String::new(),
        }];

        let parsed = resolve_odata_path_with_resolved(
            "/odata/v4/Service/Orders('11111111-1111-1111-1111-111111111111')/Items",
            &entities,
            &resolved_entities,
        );

        match parsed.path {
            ODataPath::SubCollection {
                parent_entity,
                child_entity,
                ..
            } => {
                assert_eq!(parent_entity.set_name(), "Orders");
                assert_eq!(child_entity.set_name(), "OrderItems");
            }
            _ => panic!("expected SubCollection path"),
        }
    }
}
