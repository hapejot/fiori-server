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
        .map(|e| build_draft_actions_xml(e.type_name()))
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
pub fn build_manifest_json(entities: &[&dyn ODataEntity], settings: &Settings) -> Value {
    let mut routes = Vec::new();
    let mut targets = serde_json::Map::new();
    let mut inbounds = serde_json::Map::new();

    for (idx, entity) in entities.iter().enumerate() {
        let entity_routes = entity.manifest_routes();
        // First entity gets an additional default route (empty pattern)
        // so the app has a landing page when opened with no inner hash.
        if idx == 0 {
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

/// Baut die flp.html dynamisch aus den Settings (UI5-Version, Theme, Sprache etc.).
/// Anwendungs-Kacheln werden NICHT eingebettet — sie werden von flp-init.js
/// aus /config/apps.json geladen.
pub fn build_flp_html(settings: &Settings) -> String {
    let libs = settings.libs.join(", ");
    let search_flag = if settings.enable_search { "true" } else { "false" };

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
            renderers: {{
                fiori2: {{
                    componentData: {{
                        config: {{
                            enableSearch: {search}
                        }}
                    }}
                }}
            }},
            // Passed to flp-init.js so it can enrich each app entry
            _flpComponent: {{
                id: "{comp_id}",
                resourceRoot: "{res_root}"
            }}
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
        ui5 = settings.ui5_version,
        libs = libs,
        theme = settings.theme,
        compat = settings.compat_version,
        lang = settings.language,
    )
}
