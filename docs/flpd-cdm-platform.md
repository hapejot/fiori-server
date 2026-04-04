# FLPD / CDM Platform — How It Works

## Overview

SAP Fiori Launchpad supports multiple **platform adapters** that determine how the shell discovers applications, navigation targets, and personalization. The platform is selected at bootstrap time via `sap.ushell.bootstrap(sPlatform)`, which loads adapters from `sap.ushell.adapters.{platform}`.

Known platforms:

| Platform   | Used By                                      |
|------------|----------------------------------------------|
| `local`    | Local development sandboxes (this server)    |
| `cdm`      | SAP Build Work Zone, SAP Portal Service (FLPD) |
| `abap`     | SAP S/4HANA on-premise FLP                   |

## CDM Platform Architecture

### CommonDataModelAdapter

Source: `sap/ushell/adapters/cdm/CommonDataModelAdapter.js`

The CDM adapter's sole responsibility is to provide a **site document** — a single JSON object that describes everything the launchpad needs: applications, navigation targets, visualizations, pages, spaces, groups, system aliases, and plugins.

The adapter resolves the site from one of three sources, checked in priority order:

1. **`config.siteData`** — site JSON passed inline (e.g. via `window["sap-ushell-config"]`)
2. **`config.siteDataPromise`** — a Promise that resolves to the site JSON
3. **`config.siteDataUrl`** (or legacy `config.cdmSiteUrl`) — a URL fetched via `jQuery.ajax GET`

An optional URL parameter `sap-ushell-cdm-site-url` can override the URL, but only if `allowSiteSourceFromURLParameter: true` is set in the adapter config. This is a development/testing feature.

### CommonDataModel Service

Source: `sap/ushell/services/CommonDataModel.js`

One layer above the adapter, the `CommonDataModel` service:

1. Calls `adapter.getSite()` to get the raw site document
2. Detects the CDM version (`_version` field)
3. For CDM < 3.1: loads personalization early (classic homepage with groups/tiles)
4. For CDM >= 3.1: defers personalization to page-level loading (pages & spaces model)
5. Loads personalization data from `PersonalizationV2` service (stored per-user, keyed by container `sap.ushell.cdm.personalization` or `sap.ushell.cdm3-1.personalization`)
6. Merges personalization deltas into the site/pages using `PersonalizationProcessor`
7. Exposes the personalized site to other services (`ClientSideTargetResolution`, `LaunchPage`, `Menu`, etc.)

### Site Document Structure

The CDM site JSON follows a well-defined schema. Version 3.1.0 supports the pages & spaces model:

```json
{
  "_version": "3.1.0",
  "site": {
    "identification": { "id": "site-id", "title": "My Launchpad" },
    "payload": {
      "groupsOrder": ["default_group"]
    }
  },
  "applications": {
    "app-key": {
      "sap.app": {
        "id": "com.example.myapp",
        "title": "My App",
        "crossNavigation": {
          "inbounds": {
            "inbound-key": {
              "semanticObject": "Product",
              "action": "display",
              "signature": {
                "parameters": {},
                "additionalParameters": "allowed"
              }
            }
          }
        }
      },
      "sap.ui5": {
        "componentName": "com.example.myapp"
      },
      "sap.platform.runtime": {
        "componentProperties": {
          "url": "/apps/Products/webapp",
          "asyncHints": { "libs": [...] }
        }
      },
      "sap.flp": {
        "type": "application"
      },
      "sap.ui": {
        "technology": "UI5",
        "deviceTypes": { "desktop": true, "tablet": true, "phone": true }
      }
    }
  },
  "visualizations": {
    "viz-key": {
      "vizType": "sap.ushell.StaticAppLauncher",
      "businessApp": "app-key",
      "target": {
        "semanticObject": "Product",
        "action": "display"
      },
      "vizConfig": {
        "sap.flp": {
          "target": {
            "appId": "app-key",
            "inboundId": "inbound-key"
          }
        },
        "sap.app": {
          "title": "Products",
          "subTitle": "Manage Products",
          "info": ""
        }
      }
    }
  },
  "vizTypes": {
    "sap.ushell.StaticAppLauncher": {
      "sap.app": { "id": "sap.ushell.StaticAppLauncher", "type": "component" },
      "sap.ui5": { "componentName": "sap.ushell.components.tiles.cdm.applauncher" },
      "sap.flp": {
        "vizOptions": {
          "displayFormats": {
            "supported": ["standard", "standardWide", "flat", "flatWide", "compact"],
            "default": "standard"
          }
        }
      }
    }
  },
  "groups": {
    "default_group": {
      "identification": { "id": "default_group", "title": "My Home" },
      "payload": {
        "tiles": [
          { "id": "tile-1", "vizId": "viz-key" }
        ],
        "links": [],
        "groups": []
      }
    }
  },
  "pages": {
    "page-id": {
      "identification": { "id": "page-id", "title": "Home Page" },
      "payload": {
        "layout": { "sectionOrder": ["section-1"] },
        "sections": {
          "section-1": {
            "id": "section-1",
            "title": "My Apps",
            "viz": {
              "viz-ref-1": {
                "vizId": "viz-key",
                "displayFormatHint": "standard"
              }
            }
          }
        }
      }
    }
  },
  "menus": {
    "main-menu": {
      "payload": {
        "menuEntries": [
          { "id": "space-1", "title": "Home", "type": "IBN", "target": { ... } }
        ]
      }
    }
  },
  "systemAliases": {
    "local": {
      "http": { "host": "localhost", "port": 8000 },
      "https": { "host": "", "port": 0 },
      "rfc": { "host": "", "service": 0 },
      "id": "local",
      "client": "",
      "language": ""
    }
  }
}
```

### Key Differences: CDM 3.0 vs 3.1

| Aspect                | CDM 3.0                        | CDM 3.1                          |
|-----------------------|-------------------------------|----------------------------------|
| Homepage model        | Groups with tiles and links   | Pages with sections and viz refs |
| Personalization       | Applied to entire site at load | Applied per-page on demand       |
| Navigation structure  | Flat groups                    | Spaces → Pages → Sections        |
| Personalization key   | `sap.ushell.cdm.personalization` | `sap.ushell.cdm3-1.personalization` |

### Personalization Flow

1. `CommonDataModelAdapter.getPersonalization()` returns the personalization section from the site (or reads from `PersonalizationV2` storage)
2. `PersonalizationProcessor.mixinPersonalization(site, pers)` merges deltas into the site/page
3. On save, `PersonalizationProcessor.extractPersonalization(personalized, original)` computes the delta
4. `CommonDataModelAdapter.setPersonalization(delta)` persists via `PersonalizationV2` service

Personalization covers: group order, tile arrangement, added/removed tiles, section layout, page customizations.

## FLPD / Work Zone Runtime Flow

In a production SAP Build Work Zone (Standard or Advanced) deployment:

1. The Work Zone runtime serves an HTML page that loads `sap-ushell-config` with `platform: "cdm"` and `siteDataUrl` pointing to a backend endpoint
2. `sap.ushell.bootstrap("cdm")` loads the CDM adapter set
3. `CommonDataModelAdapter` fetches the site JSON from the `siteDataUrl` endpoint via AJAX GET
4. The Work Zone backend assembles the site from: content providers (S/4HANA systems, BTP apps, custom tiles), role-based visibility, admin-configured pages/spaces/groups, and catalog assignments
5. The `CommonDataModel` service processes the site, loads user personalization, and exposes it
6. `ClientSideTargetResolution` resolves intents from `applications[*].sap.app.crossNavigation.inbounds`
7. `LaunchPage` / `FlpLaunchPage` adapter renders tiles from `groups` (3.0) or page sections from `pages` (3.1)
8. Navigation resolves `#SemanticObject-action` intents to `componentProperties.url` for the matching app

## Comparison: This Server vs CDM Platform

| Aspect                 | This Server (local platform)              | CDM Platform (FLPD/Work Zone)                |
|------------------------|------------------------------------------|----------------------------------------------|
| Bootstrap              | `Container.init("local")`                | `sap.ushell.bootstrap("cdm")`               |
| App discovery          | Inline `sap-ushell-config` → `ClientSideTargetResolution.adapter.config.applications` | Site document from REST endpoint → `CommonDataModel` service |
| Site source            | Generated inline in HTML                 | Fetched from `siteDataUrl` via AJAX          |
| Navigation targets     | Flat map: `"SO-action" → { url, ... }`   | Full inbound definitions in `applications[*].sap.app.crossNavigation.inbounds` |
| Homepage               | Simple tile grid from `applications`     | Groups/tiles (3.0) or pages/spaces/sections (3.1) |
| Visualizations         | Not used                                 | `visualizations` + `vizTypes` define tile rendering |
| Personalization        | Not available                            | Full: reorder, add/remove tiles, customize pages |
| Content providers      | Single server                            | Multiple backend systems via `systemAliases` |
| Plugins                | Not used                                 | `sap.flp.type: "plugin"` apps auto-started   |

## What Would Be Needed to Emulate CDM

To switch this server from `local` to `cdm` platform:

1. **Bootstrap change**: Replace `Container.init("local")` with `sap.ushell.bootstrap("cdm")` and set `siteDataUrl` in the config
2. **Site endpoint**: Serve a CDM 3.1 site document at e.g. `/cdm/site.json`
3. **Site builder**: Generate the full site JSON with `applications`, `visualizations`, `vizTypes`, `pages`, `groups`, and `systemAliases` from the registered entities and their navigation targets
4. **Personalization endpoint** (optional): Implement `PersonalizationV2` storage endpoints to persist user customizations

This would unlock: pages & spaces layout, tile visualization types (static launcher, dynamic, custom), multi-page homepages, and a production-like shell experience. The trade-off is substantially more complexity in both the site document generation and the required service endpoints.
