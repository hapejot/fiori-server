# Request Flow — fake-fiori-server

## 1. Server Startup

1. `main()` initializes the `tracing` logger (controlled via `RUST_LOG`).
2. `Settings::load()` reads `webapp/config/settings.json` (UI5 version, theme, language, company logo, user profile).
3. `reconstruct_configs_from_data("data/")` reads the meta table JSON files (`EntityConfigs.json`, `EntityFields.json`, `EntityFacets.json`, `EntityNavigations.json`, `EntityTableFacets.json`) and assembles them into `Vec<EntityConfig>` structs.
4. `create_generic_entities()` converts each `EntityConfig` into a `GenericEntity` implementing the `ODataEntity` trait (fields, annotations, navigation properties, manifest routing are all derived from the config).
5. `AppStateBuilder` collects all entities — the seven builtin meta-table entities (`EntityConfigs`, `EntityFields`, `EntityFacets`, `EntityNavigations`, `EntityTableFacets`, `FieldValueLists`, `FieldValueListItems`) plus all generic entities from step 4.
6. `AppStateBuilder::build()` performs all one-time computations:
   - `build_metadata_xml()` — assembles the EDMX document from all registered entities (entity types, entity sets, annotations, draft actions).
   - `build_manifest_json()` — assembles the SAP Fiori manifest with routes, targets, and cross-navigation inbounds for all entities.
   - Per-entity manifests are generated so each entity can serve as the default route when accessed via `/apps/{EntitySet}/`.
   - `build_flp_html()` — generates the Fiori Launchpad HTML shell, injecting UI5 CDN URL, theme, language, company logo, and user profile into `sap-ushell-config`.
   - `build_apps_json()` — merges the static `webapp/config/apps.json` with dynamic tile entries from generic entities.
   - `build_cdm_site_json()` — generates the CDM 3.1 site document with applications, visualizations, pages/sections, and navigation inbounds.
7. The `InMemoryDataStore` (or `PgDataStore` if the `postgres` feature is active and `DATABASE_URL` is set) is created. For each registered entity it loads records from `data/{EntitySet}.json`, falling back to `mock_data()` if the file is absent. Draft flags (`IsActiveEntity`, `HasActiveEntity`, `HasDraftEntity`) are injected into every record.
8. Axum routes are registered:
   - `GET /health` — health check.
   - `GET {base}/$metadata` — EDMX metadata.
   - `GET {base}/` — OData service document.
   - `POST {base}/$batch` — OData batch handler.
   - `GET {base}/{EntitySet}` and `GET {base}/{EntitySet}/$count` — one route pair per registered entity set.
   - A catch-all fallback handles everything else (single-entity reads, PATCH, DELETE, POST, draft actions, sub-collections, static files).
9. The server binds to `0.0.0.0:{PORT}` (default 8000) and starts listening with graceful shutdown on SIGINT/SIGTERM.

## 2. Opening the Fiori Launchpad (GET /)

1. The browser requests `GET /`. The catch-all handler resolves this as a static file request.
2. `handle_file()` normalizes the path to `flp.html` and returns the pre-generated FLP HTML from `AppState.flp_html`.
3. The HTML loads the UI5 core from the SAP CDN (version from settings), along with the UShell bootstrap libraries.
4. The `xx-bootTask` script tag executes `flp-init.js` (served as an embedded file).
5. `flp-init.js` boots the UShell in CDM platform mode (`Container.init("cdm")`). The `CommonDataModelAdapter` automatically fetches the CDM 3.1 site document from `/cdm/site.json`.
6. The site document contains all application definitions, visualizations, pages with sections, and navigation inbounds.
7. After renderer load, `applyUshellProperties()` sets the company logo and `applyUserProfile()` applies user info.
8. The Launchpad renders tiles for each application. Clicking a tile triggers intent-based navigation.

## 3. Opening an Application (GET /apps/{EntitySet}/...)

1. The user clicks a tile (e.g. "Products"). The UShell navigates to `#Products-display`.
2. The UShell resolves the semantic object and loads the component from `/apps/Products/Component.js`.
3. `handle_file()` detects the `/apps/Products/` prefix, extracts `Products` as the entity hint.
4. `Component.js` is generated dynamically with a unique class name (`products.app.Component`) to avoid UI5 caching conflicts between apps.
5. The component loads `/apps/Products/manifest.json`. The handler returns the per-entity manifest where `Products` is the default route (initial load target).
6. SAP Fiori Elements (ListReport/ObjectPage) reads the manifest and requests `$metadata` and the entity data.

## 4. Metadata Request (GET {base}/$metadata)

1. `metadata_handler()` returns the pre-built EDMX XML from `AppState.metadata_xml`.
2. The EDMX includes for every registered entity: the EntityType (with properties, navigation properties), the EntitySet (with navigation property bindings), annotations (UI.LineItem, UI.HeaderInfo, UI.SelectionFields, UI.Facets, UI.FieldGroup, UI.DataPoint, Common.ValueList, draft annotations), and bound draft actions.

## 5. Collection Request (GET {base}/{EntitySet})

1. `collection_handler()` acquires a read lock on the entity list.
2. `resolve_odata_path()` matches the URL path against registered entity set names to find the target entity.
3. `ODataQuery::parse()` extracts query options from the URL (`$filter`, `$orderby`, `$select`, `$expand`, `$top`, `$skip`, `$count`).
4. `DataStore::get_collection()` is called:
   - Reads all records for the entity set from the in-memory store.
   - Applies the `$filter` expression (supports `eq`, `ne`, `lt`, `gt`, `le`, `ge`, `and`).
   - Applies `$orderby` (ascending/descending, string/numeric comparison).
   - For value-list text fields, resolves `_text` suffixed fields by looking up FieldValueListItems.
   - Processes `$expand` — for each navigation property, calls `expand_record()` on the entity to attach related data.
   - Injects `SiblingEntity` and `DraftAdministrativeData` into each record.
   - Applies `$select` to trim fields.
   - Applies `$skip` and `$top` for pagination.
   - If `$count=true`, includes `@odata.count` in the response.
5. The result is serialized as JSON with `@odata.context` and returned.

## 6. Single Entity Request (GET {base}/{EntitySet}(key))

1. The catch-all handler detects `ODataPath::Entity` from URL parsing.
2. `handle_single_entity()` builds an `EntityKey` (composite of the key field + `IsActiveEntity`).
3. `DataStore::read_entity()` finds the matching record, resolves value-list texts, processes `$expand`, injects draft metadata, and applies `$select`.

## 7. Sub-Collection Request (GET {base}/{Parent}(key)/{NavProperty})

1. `resolve_odata_path()` detects the pattern `EntitySet(key)/NavigationProperty` and resolves both the parent entity and the child entity via the navigation property definitions.
2. `handle_sub_collection()` creates a `ParentKey` and delegates to `DataStore::get_collection()` with the parent filter.
3. The data store filters child records by the foreign key matching the parent's key value.

## 8. Batch Request (POST {base}/$batch)

1. `batch_handler()` parses the multipart MIME body using the boundary from the Content-Type header.
2. Each part is classified as either a direct request or a changeset (nested multipart/mixed).
3. Changesets group multiple write operations (POST, PATCH, DELETE) and process them sequentially.
4. For each sub-request, the handler dispatches based on method:
   - `GET` — delegates to `handle_batch_get()` which resolves the URL and reads data.
   - `POST` — delegates to `handle_batch_post()` which handles entity creation, draft actions (`draftEdit`, `draftActivate`, `draftPrepare`), `publishConfig`, and sub-collection creation.
   - `PATCH` — delegates to `handle_batch_patch()` which updates entity fields.
   - `DELETE` — delegates to `handle_batch_delete()` which discards drafts.
5. Responses are assembled as multipart MIME and returned in a single HTTP response.

## 9. Draft Lifecycle (via Batch POST)

1. **draftEdit** — `DataStore::draft_edit()` creates a draft copy of the active record (`IsActiveEntity=false`, `HasActiveEntity=true`) and marks the active record with `HasDraftEntity=true`. Child entities (compositions) are automatically copied as drafts via `copy_children_as_drafts()`.
2. **PATCH on draft** — The UI sends PATCH requests to update individual fields on the draft record.
3. **draftPrepare** — `DataStore::draft_prepare()` validates the draft and returns it (no-op validation currently).
4. **draftActivate** — `DataStore::draft_activate()` merges the draft into the active record (or promotes it to active if new), removes the draft, and activates all child drafts via `activate_children()`. Then `commit()` persists all entity sets to their JSON files.
5. **DELETE on draft** — Discards the draft and resets `HasDraftEntity=false` on the active record. Child drafts are removed via `remove_child_drafts()`.

## 10. Entity Creation (POST {base}/{EntitySet})

1. The catch-all detects a POST to a collection URL.
2. `DataStore::create_entity()` auto-generates a UUID key (if the key field is `ID`), applies `default_values()` from the entity definition, merges the request body, and inserts the record as a draft (`IsActiveEntity=false`).
3. `commit()` persists the updated store.

## 11. Publishing a Configuration (publishConfig Action)

1. A POST to `EntityConfigs('{SetName}')/Service.publishConfig` triggers the publish flow.
2. `publish_entity_config()` finds the active config record and calls `commit()` to persist all data.
3. `AppState::activate_config()` runs the full rebuild cycle:
   - Calls `commit()` to ensure data is on disk.
   - `reconstruct_configs_from_data()` re-reads all meta table JSON files.
   - `create_generic_entities()` rebuilds the generic entities.
   - Builtin entities (meta tables, value lists) are kept; old generic entities are replaced.
   - EDMX metadata, manifest.json, per-entity manifests, apps.json, and CDM site document are all regenerated.
   - `DataStore::update_entities()` registers any new entity sets and loads their data.
   - All RwLock-protected fields on AppState are swapped to the new values.
4. Subsequent requests immediately see the updated metadata, manifests, and entity definitions.

## 12. Static File Serving

1. Any request not matching an OData route falls through to `handle_file()`.
2. Entity-specific app paths (`/apps/{EntitySet}/...`) are detected and the entity hint is extracted for manifest resolution.
3. Special files are generated dynamically: `flp.html`, `manifest.json`, `Component.js`, `config/apps.json`.
4. Embedded files (`flp-init.js`, `i18n/i18n.properties`, `appconfig/fioriSandboxConfig.json`) are served from compile-time constants.
5. Other files are served from the `webapp/` directory with path-traversal protection (canonical path must be within `webapp/`).
6. Extensionless paths that don't start with `/sap/` get the SPA fallback (returns `flp.html`).
