use crate::entity::ODataEntity;
use crate::BASE_PATH;

/// Geparste Entity-Key-Informationen (Schluesselwert + IsActiveEntity).
#[derive(Debug)]
pub struct EntityKeyInfo {
    pub key_value: String,
    pub is_active: bool,
}

#[derive(Debug)]
/// Aufgeloester OData-Pfad – alle Varianten, die der Server unterstuetzt.
pub enum ODataPath<'a> {
    /// Service-Root: /odata/v4/Service
    ServiceRoot,
    /// Collection: /odata/v4/Service/Products
    Collection { entity: &'a dyn ODataEntity },
    /// $count:     /odata/v4/Service/Products/$count
    Count { entity: &'a dyn ODataEntity },
    /// Single Entity: /odata/v4/Service/Products('P001')
    Entity {
        entity: &'a dyn ODataEntity,
        key: EntityKeyInfo,
    },
    /// Sub-Collection: /odata/v4/Service/Orders('O001')/Items
    SubCollection {
        parent_entity: &'a dyn ODataEntity,
        parent_key: EntityKeyInfo,
        child_entity: &'a dyn ODataEntity,
    },
    /// Bound Action:  /odata/v4/Service/Products('P001')/Ns.draftEdit
    Action {
        entity: &'a dyn ODataEntity,
        key: EntityKeyInfo,
        action: String,
    },
    /// Property Access: /odata/v4/Service/Orders('O001')/OrderID
    PropertyAccess {
        entity: &'a dyn ODataEntity,
        key: EntityKeyInfo,
        property: String,
    },
    /// Nicht erkannter Pfad
    Unknown,
}

#[derive(Debug)]
/// Ergebnis von resolve_odata_path: aufgeloester Pfad + abgetrennter Query-String.
pub struct ParsedODataUrl<'a> {
    pub path: ODataPath<'a>,
    pub query_string: String,
}

/// Loest eine rohe URL (relativ oder absolut, mit oder ohne Query-String)
/// in einen strukturierten ODataPath auf.
///
/// Beispiele:
///   - `Products`                           → Collection
///   - `Products/$count`                    → Count
///   - `Products('P001')`                   → Entity
///   - `Products('P001')/Ns.draftEdit`      → Action { action: "draftEdit" }
///   - `/odata/v4/Service/Products` → Collection (absolut)
///   - `Products?$filter=...`               → Collection + query_string
pub fn resolve_odata_path<'a>(
    raw_url: &str,
    entities: &'a [&dyn ODataEntity],
) -> ParsedODataUrl<'a> {
    // URL-Dekodierung (z.B. %27 → ')
    let decoded_url = urlencoding::decode(raw_url).unwrap_or_default();
    let raw_url = decoded_url.as_ref();

    // Relativ → absolut normalisieren
    let full = if raw_url.starts_with('/') {
        raw_url.to_string()
    } else {
        format!("{}/{}", BASE_PATH, raw_url)
    };

    // Query-String abtrennen
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
                path: ODataPath::Collection { entity: *entity },
                query_string: query_part.to_string(),
            };
        }

        // $count
        if path == count_path {
            return ParsedODataUrl {
                path: ODataPath::Count { entity: *entity },
                query_string: query_part.to_string(),
            };
        }

        // Entity oder Action:  /SetPath(key...) oder /SetPath(key...)/Ns.action
        let set_prefix = format!("{}(", set_path);
        if let Some(rest) = path.strip_prefix(&set_prefix) {
            // Action or SubCollection: suche ")/" als Trenner
            if let Some(paren_end) = rest.find(")/") {
                let key_str = &rest[..paren_end];
                let after_paren = &rest[paren_end + 2..];
                if let Some(key) = parse_key_content(key_str, entity.key_field()) {
                    // Erst pruefen ob after_paren mit einem NavigationProperty beginnt
                    let first_segment = after_paren
                        .split(|c: char| c == '(' || c == '/')
                        .next()
                        .unwrap_or(after_paren);
                    let nav_props = entity.navigation_properties();
                    if let Some(nav_def) = nav_props.iter().find(|np| np.name == first_segment) {
                        if let Some(child) = entities.iter().find(|e| e.type_name() == nav_def.target_type) {
                            // Kind-Entity mit Key: Items(ItemID='I002',IsActiveEntity=true)
                            if let Some(child_key_start) = after_paren.find('(') {
                                let child_rest = &after_paren[child_key_start + 1..];
                                // Kind-Entity mit Action: Items(key)/Ns.action
                                if let Some(cp_end) = child_rest.find(")/") {
                                    let child_key_str = &child_rest[..cp_end];
                                    let child_after = &child_rest[cp_end + 2..];
                                    if let Some(child_key) = parse_key_content(child_key_str, child.key_field()) {
                                        if child_after.contains('.') {
                                            let action = child_after.rsplit('.').next().unwrap_or(child_after).to_string();
                                            return ParsedODataUrl {
                                                path: ODataPath::Action {
                                                    entity: *child,
                                                    key: child_key,
                                                    action,
                                                },
                                                query_string: query_part.to_string(),
                                            };
                                        }
                                    }
                                }
                                // Kind-Entity ohne Action: Items(key)
                                if let Some(child_key_str) = child_rest.strip_suffix(')') {
                                    if let Some(child_key) = parse_key_content(child_key_str, child.key_field()) {
                                        return ParsedODataUrl {
                                            path: ODataPath::Entity {
                                                entity: *child,
                                                key: child_key,
                                            },
                                            query_string: query_part.to_string(),
                                        };
                                    }
                                }
                            }
                            // Einfache Sub-Collection ohne Kind-Key: Items
                            return ParsedODataUrl {
                                path: ODataPath::SubCollection {
                                    parent_entity: *entity,
                                    parent_key: key,
                                    child_entity: *child,
                                },
                                query_string: query_part.to_string(),
                            };
                        }
                    }
                    // Bound Action am Parent: Ns.actionName (enthaelt '.')
                    if after_paren.contains('.') {
                        let action = after_paren
                            .rsplit('.')
                            .next()
                            .unwrap_or(after_paren)
                            .to_string();
                        return ParsedODataUrl {
                            path: ODataPath::Action {
                                entity: *entity,
                                key,
                                action,
                            },
                            query_string: query_part.to_string(),
                        };
                    }
                    // Property Access: Entity(key)/PropertyName
                    return ParsedODataUrl {
                        path: ODataPath::PropertyAccess {
                            entity: *entity,
                            key,
                            property: after_paren.to_string(),
                        },
                        query_string: query_part.to_string(),
                    };
                }
            }

            // Single Entity: abschliessendes ')' abschneiden
            if let Some(key_str) = rest.strip_suffix(')') {
                if let Some(key) = parse_key_content(key_str, entity.key_field()) {
                    return ParsedODataUrl {
                        path: ODataPath::Entity {
                            entity: *entity,
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

/// Parst den Inhalt ZWISCHEN den Klammern eines OData-Keys.
///
/// Akzeptiert:
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
