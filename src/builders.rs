use serde_json::{json, Value};

use crate::annotations::{build_draft_actions_xml, build_draft_admin_type_xml};
use crate::entity::ODataEntity;
use crate::settings::Settings;
use crate::{BASE_PATH, NAMESPACE};

/// Baut das komplette EDMX-Dokument aus allen registrierten Entitaeten.
pub fn build_metadata_xml(entities: &[&dyn ODataEntity]) -> String {
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

    // DraftAdministrativeData EntityType
    let draft_admin_type = build_draft_admin_type_xml();

    // Bound draft actions fuer jede Entitaet
    let draft_actions: String = entities
        .iter()
        .map(|e| {
            let mut actions = build_draft_actions_xml(e.type_name());
            let custom = e.custom_actions_xml();
            if !custom.is_empty() {
                actions.push_str(&custom);
            }
            actions
        })
        .collect::<Vec<_>>()
        .join("\n");

    println!("entity types: {}", entity_types.len());
    println!("entity sets: {}", entity_sets.len());
    println!("annotations: {}", annotations.len());

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<edmx:Edmx Version="4.0" xmlns:edmx="http://docs.oasis-open.org/odata/ns/edmx">

  <edmx:Reference Uri="https://oasis-tcs.github.io/odata-vocabularies/vocabularies/Org.OData.Capabilities.V1.xml">
    <edmx:Include Namespace="Org.OData.Capabilities.V1" Alias="Capabilities"/>
  </edmx:Reference>
  <edmx:Reference Uri="https://oasis-tcs.github.io/odata-vocabularies/vocabularies/Org.OData.Core.V1.xml">
    <edmx:Include Namespace="Org.OData.Core.V1" Alias="Core"/>
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
{draft_admin_type}
{draft_actions}
      <EntityContainer Name="EntityContainer">
{entity_sets}
        <EntitySet Name="DraftAdministrativeData" EntityType="{ns}.DraftAdministrativeData"/>
      </EntityContainer>
{annotations}
    </Schema>
  </edmx:DataServices>
</edmx:Edmx>"#,
        ns = NAMESPACE,
        entity_types = entity_types,
        draft_admin_type = draft_admin_type,
        draft_actions = draft_actions,
        entity_sets = entity_sets,
        annotations = annotations,
    )
}

/// Baut das komplette manifest.json dynamisch aus allen registrierten Entitaeten.
/// `default_entity_idx` bestimmt, welche Entitaet die Default-Route (leerer Hash) bekommt.
pub fn build_manifest_json(entities: &[&dyn ODataEntity], settings: &Settings) -> Value {
    build_manifest_json_with_default(entities, settings, 0)
}

/// Wie `build_manifest_json`, aber mit waehlbarer Default-Entitaet.
pub fn build_manifest_json_with_default(
    entities: &[&dyn ODataEntity],
    settings: &Settings,
    default_entity_idx: usize,
) -> Value {
    let mut routes = Vec::new();
    let mut targets = serde_json::Map::new();
    let mut inbounds = serde_json::Map::new();

    for (idx, entity) in entities.iter().enumerate() {
        let entity_routes = entity.manifest_routes();
        // The chosen default entity gets an additional default route (empty pattern)
        // so the app has a landing page when opened with no inner hash.
        if idx == default_entity_idx {
            if let Some(first_route) = entity_routes.first() {
                if let Some(target) = first_route.get("target") {
                    routes.push(json!({
                        "pattern": ":?query:",
                        "name": "default",
                        "target": target
                    }));
                }
            }
        }
        routes.extend(entity_routes);
        for (key, val) in entity.manifest_targets() {
            targets.insert(key, val);
        }
        let (inbound_key, inbound_val) = entity.manifest_inbound();
        inbounds.insert(inbound_key, inbound_val);
    }

    // Entitaet-spezifische App-ID: z.B. "products.app", "orders.app"
    let default_entity = entities[default_entity_idx];
    let app_id = format!("{}.app", default_entity.set_name().to_lowercase());
    let app_title = default_entity.tile_title();

    build_manifest_value(&app_id, &app_title, routes, targets, inbounds, settings)
}

/// Baut ein manifest.json fuer eine einzelne Entitaet (CDM-Modus).
/// Nur Routen/Targets/Inbounds der Entitaet und ihrer Kompositions-Kinder
/// werden aufgenommen — so erkennt die UShell Cross-App-Navigation korrekt.
pub fn build_entity_manifest(
    entities: &[&dyn ODataEntity],
    settings: &Settings,
    entity_idx: usize,
) -> Value {
    let target_entity = entities[entity_idx];
    let target_set = target_entity.set_name();
    let app_id = format!("{}.app", target_set.to_lowercase());
    let app_title = target_entity.tile_title();

    let mut routes = Vec::new();
    let mut targets = serde_json::Map::new();
    let mut inbounds = serde_json::Map::new();

    for (idx, entity) in entities.iter().enumerate() {
        // Only include the target entity itself and its composition children.
        let dominated_by_target = entity.parent_set_name() == Some(target_set);
        if idx != entity_idx && !dominated_by_target {
            continue;
        }

        let entity_routes = entity.manifest_routes();
        if idx == entity_idx {
            // Default route for the main entity
            if let Some(first_route) = entity_routes.first() {
                if let Some(target) = first_route.get("target") {
                    routes.push(json!({
                        "pattern": ":?query:",
                        "name": "default",
                        "target": target
                    }));
                }
            }
        }
        routes.extend(entity_routes);
        for (key, val) in entity.manifest_targets() {
            targets.insert(key, val);
        }
        let (inbound_key, inbound_val) = entity.manifest_inbound();
        if !inbound_val.is_null() {
            inbounds.insert(inbound_key, inbound_val);
        }
    }

    build_manifest_value(&app_id, &app_title, routes, targets, inbounds, settings)
}

fn build_manifest_value(
    app_id: &str,
    app_title: &str,
    routes: Vec<Value>,
    targets: serde_json::Map<String, Value>,
    inbounds: serde_json::Map<String, Value>,
    settings: &Settings,
) -> Value {
    json!({
        "_version": "1.65.0",
        "sap.app": {
            "id": app_id,
            "type": "application",
            "i18n": "i18n/i18n.properties",
            "applicationVersion": {
                "version": "1.0.0"
            },
            "title": app_title,
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
            "flexEnabled": true,
            "rootView": {
                "viewName": "sap.fe.core.rootView.Fcl",
                "type": "XML",
                "id": "appRootView"
            },
            "dependencies": {
                "minUI5Version": settings.ui5_version,
                "libs": {
                    "sap.m": {},
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
                },
                "i18n": {
                    "type": "sap.ui.model.resource.ResourceModel",
                    "settings": {
                        "bundleUrl": "i18n/i18n.properties"
                    }
                }
            },
            "routing": {
                "config": {
                    "routerClass": "sap.f.routing.Router",
                    "controlAggregation": "beginColumnPages",
                    "controlId": "appContent",
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

/// Baut das CDM 3.1 Site-Dokument aus allen registrierten Entitaeten.
/// Wird von der UShell im CDM-Modus ueber /cdm/site.json geladen.
pub fn build_cdm_site_json(entities: &[&dyn ODataEntity]) -> Value {
    let mut applications = serde_json::Map::new();
    let mut visualizations = serde_json::Map::new();
    let mut viz_refs = serde_json::Map::new();
    let mut viz_order = Vec::new();

    for entity in entities {
        let entry = match entity.apps_json_entry() {
            Some((_, v)) => v,
            None => continue,
        };

        let set_name = entity.set_name();
        let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or(set_name);
        let description = entry.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let icon = entry.get("icon").and_then(|v| v.as_str()).unwrap_or("sap-icon://sys-help");
        let semantic_object = entry.get("semanticObject").and_then(|v| v.as_str()).unwrap_or(set_name);
        let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("display");

        let app_key = format!("{}-{}", semantic_object, action);
        let app_id = format!("{}.app", set_name.to_lowercase());
        let viz_key = format!("{}-viz", app_key);

        // Application entry (CDM format)
        // The CDM applications map key MUST equal sap.app.id — CSTR resolves
        // appId from sap.app.id, and the sap-ui-app-id-hint on the navigation
        // hash must match this key for readApplications.getInboundTarget() to
        // find the application.
        let (inbound_key, inbound_val) = entity.manifest_inbound();
        let mut inbounds = serde_json::Map::new();
        inbounds.insert(inbound_key.clone(), inbound_val);

        applications.insert(app_id.clone(), json!({
            "sap.app": {
                "id": app_id,
                "title": title,
                "subTitle": description,
                "crossNavigation": {
                    "inbounds": inbounds
                }
            },
            "sap.ui5": {
                "componentName": app_id
            },
            "sap.platform.runtime": {
                "componentProperties": {
                    "url": format!("./apps/{}/", set_name)
                }
            },
            "sap.flp": {
                "type": "application"
            },
            "sap.ui": {
                "technology": "UI5",
                "deviceTypes": {
                    "desktop": true,
                    "tablet": true,
                    "phone": true
                }
            }
        }));

        // Visualization entry
        visualizations.insert(viz_key.clone(), json!({
            "vizType": "sap.ushell.StaticAppLauncher",
            "businessApp": app_id,
            "target": {
                "semanticObject": semantic_object,
                "action": action
            },
            "vizConfig": {
                "sap.flp": {
                    "target": {
                        "appId": app_id,
                        "inboundId": inbound_key
                    }
                },
                "sap.app": {
                    "title": title,
                    "subTitle": description,
                    "icon": icon,
                    "info": ""
                }
            }
        }));

        // Viz reference for the home page section
        viz_refs.insert(viz_key.clone(), json!({
            "id": viz_key,
            "vizId": viz_key
        }));
        viz_order.push(viz_key);
    }

    // Build the single home page with one section containing all viz refs
    let page = json!({
        "identification": {
            "id": "home-page",
            "title": "Home"
        },
        "payload": {
            "layout": {
                "sectionOrder": ["apps-section"]
            },
            "sections": {
                "apps-section": {
                    "id": "apps-section",
                    "title": "Applications",
                    "default": true,
                    "visible": true,
                    "preset": true,
                    "locked": false,
                    "layout": {
                        "vizOrder": viz_order
                    },
                    "viz": viz_refs
                }
            }
        }
    });

    json!({
        "_version": "3.1.0",
        "site": {
            "identification": {
                "id": "local-flp-site",
                "title": "Local Fiori Launchpad"
            },
            "payload": {}
        },
        "applications": applications,
        "visualizations": visualizations,
        "vizTypes": {},
        "pages": {
            "home-page": page
        },
        "menus": {
            "main": {
                "payload": {
                    "menuEntries": [
                        {
                            "id": "home-space-entry",
                            "title": "Home",
                            "type": "IBN",
                            "target": {
                                "semanticObject": "Launchpad",
                                "action": "openFLPPage",
                                "parameters": [
                                    { "name": "spaceId", "value": "home-space" },
                                    { "name": "pageId", "value": "home-page" }
                                ]
                            }
                        }
                    ]
                }
            }
        },
        "systemAliases": {}
    })
}

/// Baut die flp.html dynamisch aus den Settings (UI5-Version, Theme, Sprache etc.).
/// Verwendet den CDM-Modus der UShell — Anwendungen werden ueber das
/// CDM Site-Dokument (/cdm/site.json) geladen statt ueber apps.json.
pub fn build_flp_html(settings: &Settings) -> String {
    let libs = settings.libs.join(", ");
    let search_flag = if settings.enable_search { "true" } else { "false" };

    let ushell_properties = if let Some(ref logo) = settings.company_logo {
        format!(
            r#",
            ushellProperties: {{
                "/core/companyLogo/url": "{}"
            }}"#,
            logo
        )
    } else {
        String::new()
    };

    format!(
        r##"<!doctype html>
<html>
<head>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Local Fiori Launchpad</title>
    <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>

    <script type="text/javascript">
        window["sap-ushell-config"] = {{
            defaultRenderer: "{renderer}",
            ushell: {{
                spaces: {{
                    enabled: true,
                    myHome: {{
                        enabled: false
                    }}
                }},
                shell: {{
                    enablePersonalization: false
                }}
            }},
            renderers: {{
                fiori2: {{
                    componentData: {{
                        config: {{
                            enableSearch: {search},
                            rootIntent: "Shell-home"
                        }}
                    }}
                }}
            }},
            services: {{
                CommonDataModel: {{
                    adapter: {{
                        config: {{
                            siteDataUrl: "/cdm/site.json"
                        }}
                    }}
                }},
                // CDM platform has no UserInfoAdapter — use the local one
                UserInfo: {{
                    adapter: {{
                        module: "sap.ushell.adapters.local.UserInfoAdapter",
                        config: {{
                            id: "{user_id}",
                            firstName: "{first}",
                            lastName: "{last}",
                            fullName: "{full}",
                            email: "{email}"
                        }}
                    }}
                }},
                Container: {{
                    adapter: {{
                        config: {{
                            id: "{user_id}",
                            firstName: "{first}",
                            lastName: "{last}",
                            fullName: "{full}",
                            email: "{email}",
                            storageResourceRoot: "/"
                        }}
                    }}
                }},
                // Use local Personalization adapter (localStorage-based)
                Personalization: {{
                    adapter: {{
                        module: "sap.ushell.adapters.local.PersonalizationAdapter",
                        config: {{
                            storageResourceRoot: "/"
                        }}
                    }}
                }},
                PersonalizationV2: {{
                    adapter: {{
                        module: "sap.ushell.adapters.local.PersonalizationAdapter",
                        config: {{
                            storageResourceRoot: "/"
                        }}
                    }}
                }}
            }}{ushell_props}
        }};
    </script>

    <script src="flp-init.js"></script>

    <script
        id="sap-ui-bootstrap"
        src="https://ui5.sap.com/{ui5}/resources/sap-ui-core.js"
        data-sap-ui-libs="{libs}"
        data-sap-ui-async="true"
        data-sap-ui-preload="async"
        data-sap-ui-theme="{theme}"
        data-sap-ui-compatVersion="{compat}"
        data-sap-ui-language="{lang}"
        data-sap-ui-bindingSyntax="complex"
        data-sap-ui-resourceroots='{{
            "{comp_id}": "{res_root}"
        }}'
        data-sap-ui-frameOptions="allow"
    ></script>

    <script>
        sap.ui.getCore().attachInit(function () {{
            sap.ushell.Container.createRenderer("{renderer}", true).then(function (oRenderer) {{
                oRenderer.placeAt("content");
            }});
        }});
    </script>
</head>
<body class="sapUiBody" id="content"></body>
</html>"##,
        renderer = settings.renderer,
        search = search_flag,
        comp_id = settings.component_id,
        res_root = settings.resource_root,
        ushell_props = ushell_properties,
        user_id = settings.user_id,
        first = settings.user_first_name,
        last = settings.user_last_name,
        full = settings.user_full_name,
        email = settings.user_email,
        ui5 = settings.ui5_version,
        libs = libs,
        theme = settings.theme,
        compat = settings.compat_version,
        lang = settings.language,
    )
}
