# Copilot Instructions — fake-fiori-server

## Project Overview
Rust/Axum OData V4 mock server for SAP Fiori Elements. Simulates draft-enabled CRUD, batch requests, metadata (EDMX), and a Fiori Launchpad shell using the CDM 3.1 platform.

## Architecture

### Key Files
- `src/main.rs` — Server startup, route registration, graceful shutdown
- `src/app_state.rs` — Shared state with RwLock-wrapped mutable fields + `activate_config()`
- `src/data_store.rs` — `DataStore` trait + in-memory implementation with draft support
- `src/pg_store.rs` — PostgreSQL `DataStore` implementation (feature-gated: `postgres`)
- `src/handlers.rs` — HTTP handlers (collection, entity, batch, draft actions)
- `src/annotations.rs` — Structured annotation types (`PV`, `Rec`, `Ann`, `Anns`), builder functions, `FieldDef`, `ValueListDef`
- `src/entity.rs` — `ODataEntity` trait definition
- `src/entities/` — Entity implementations (products, orders, meta tables, generic entities)
- `src/builders.rs` — Build metadata XML, manifests, CDM site document, FLP HTML
- `src/routing.rs` — OData URL parsing
- `src/query.rs` — OData query execution ($filter, $orderby, $select, $expand)
- `migrations/` — SQL schema files for PostgreSQL

### Entity Registration
1. Create struct implementing `ODataEntity` trait
2. Implement: `set_name`, `key_field`, `type_name`, `mock_data`, `entity_set`, `fields_def`, `annotations_def`
3. Register in `AppStateBuilder` via `.entity()`
4. Automatically included in EDMX, manifest.json, CDM site document
5. Optional: JSON file in `data/` for persistence; falls back to `mock_data()`
6. Optional: `navigation_properties()` for compositions/references, `parent_set_name()` for child entities

### Built-in Entities

**Domain entities:**
- `Products` (key: ID/Guid) — standalone, 12 fields, DataPoints (Price/Stock/Rating), 2 facets, helper: `product_id()`
- `Orders` (key: ID/Guid) — parent of OrderItems, 10 fields, nested ObjectPage routing, helper: `order_id()`
- `OrderItems` (key: ID/Guid, parent: Orders) — composition child, FK `OrderID` → Orders, FK `ProductID` → Products (with `text_path` for display), nav ref `Product`

**Meta tables** (configure generic entities at runtime):
- `EntityConfigs` (key: SetName) — parent of Fields/Facets/Navigations/TableFacets, `publishConfig` action
- `EntityFields` (key: FieldID, parent: EntityConfigs) — 17 fields, nav ref `_ValueList` → FieldValueLists
- `EntityFacets` (key: FacetID, parent: EntityConfigs)
- `EntityNavigations` (key: NavID, parent: EntityConfigs)
- `EntityTableFacets` (key: TableFacetID, parent: EntityConfigs)

**Value lists:**
- `FieldValueLists` (key: ID/Guid) — parent of Items, has Launchpad tile
- `FieldValueListItems` (key: ID/Guid, parent: FieldValueLists) — FK: ListID

### Annotation Architecture

**Structured types** (in `annotations.rs`):
- `PV` enum — Property value variants: `Str`, `Path`, `AnnotationPath`, `NavPropPath`, `PropPath`, `EnumMember`, `Int`, `Bool`, `Record(Rec)`, `Collection(Vec<Rec>)`, `PropertyPaths(Vec<String>)`
- `Rec` struct — `<Record Type="...">` with `props: Vec<PV>`
- `Ann` struct — `<Annotation Term="..." Qualifier="...">` with `AnnContent` payload
- `AnnContent` enum — `Record`, `Collection`, `PropertyPaths`, `Str`, `Bool`, `EnumMember`, `PathWithChildren`, `Empty`
- `Anns` struct — `<Annotations Target="...">` grouping multiple `Ann` items
- All types implement `to_xml()` for serialization; `anns_to_xml(&[Anns])` serializes a full block

**Builder pattern** — Structured builders return `Vec<Anns>`, thin XML wrappers delegate + serialize:
- `build_annotations()` → `build_annotations_xml()` — UI.SelectionFields, UI.LineItem, UI.HeaderInfo, UI.HeaderFacets, UI.DataPoint, UI.Facets, UI.FieldGroup
- `build_capabilities()` → `build_capabilities_annotations()` — UpdateRestrictions, InsertRestrictions, Draft annotations, property-level Common.Label, UI.Hidden, Core.Computed, Core.Immutable, Common.Text
- `build_value_list_anns()` — Common.ValueList parameters (Out, DisplayOnly, Constant for ListID filter)

**Definition types:**
- `AnnotationsDef` composes: `HeaderInfoDef`, `SelectionFields`, `LineItemField[]`, `DataPointDef[]`, `HeaderFacetDef[]`, `FacetSectionDef[]`, `FieldGroupDef[]`, `TableFacetDef[]`
- `LineItemField` variants: `UI.DataField` (default), `UI.DataFieldWithIntentBasedNavigation` (semantic_object), `UI.DataFieldWithNavigationPath` (navigation_path)
- Entity-specific annotations via: `extra_annotations_xml()` (appended to standard), `custom_actions_xml()` (bound OData actions)

### Data Flow
- `InMemoryDataStore::new()` loads from `data/<EntitySet>.json`, falls back to `mock_data()`
- Draft flags (`IsActiveEntity`, `HasActiveEntity`, `HasDraftEntity`) added on load
- `commit()` persists active records to `data/` as JSON (strips draft flags)

## Conventions

### Entity Key Fields
- All entities use `ID` as key field with type `Edm.Guid`
- `create_entity()` auto-generates the key as a random UUID v4 when not provided
- Mock data uses deterministic UUIDs via `value_list_id()` (UUID v5 from a fixed namespace + name)
- Domain entities expose helper functions for cross-referencing: `product_id("P001")`, `order_id("O001")`
- FK fields (e.g. `OrderItems.OrderID`, `OrderItems.ProductID`) store the UUID of the referenced entity

### Title Field (Common.Text)
- Every entity has `title_field()` returning the primary display field name
- Default implementation: derives from `annotations_def().header_info.title_path`
- When `title_field != key_field`, `build_capabilities_annotations` emits `Common.Text` + `UI.TextArrangement/TextOnly` on the key property
- Effect: Fiori Elements shows the text field instead of the key/UUID wherever the entity is referenced
- Generic entities auto-derive from `HeaderTitlePath` — no extra config needed

### Foreign Keys & NavigationProperties
- When parent `key_field` differs from child FK field (e.g. `FieldValueLists.ID` → `FieldValueListItems.ListID`), `NavigationPropertyDef` must declare `foreign_key: Some("ListID")`
- All code resolving child FK must use `resolve_child_fk()` in data_store.rs — never assume `parent.key_field() == child FK column`
- Applies to: `get_collection`, `create_entity`, `copy_children_as_drafts`, `activate_children`, `remove_child_drafts`

### FieldDef
- Every `FieldDef` instance must include `computed` and `value_list` fields
- `computed: true` → `Core.Computed` annotation + `NonInsertableProperties` — field is server-generated, never shown in create/edit forms (key GUIDs, StatusCriticality, CreatedAt, NetAmount, composition FKs)
- `immutable: true` → `Core.Immutable` annotation — field can be set at creation time, becomes read-only afterward (e.g. OrderDate)
- `Edm.Guid` fields also get `UI.Hidden` automatically — hidden from all UI surfaces
- `default_values()` on `ODataEntity` provides entity-specific initial values for new drafts (e.g. `Currency: "EUR"`, `Status: "A"`)
- `text_path: Option<&str>` — points to the computed text field for display (e.g. `"_Status_text"`); when set, `build_capabilities` emits `Common.Text` + `UI.TextArrangement/TextOnly` on the source property
- `ValueListDef` supports flexible value help: `collection_path`, `key_property`, `display_property`, `fixed_values`
- When `value_list` is `Some`, annotation generation uses it; when `None`, falls back to `value_source` (classic FieldValueListItems path)
- **`value_source` stores the UUID** of the FieldValueList, not the list name
- Annotation emits `Common.ValueListParameterConstant` with `ListID` = the UUID to filter items
- EntityField.ValueSource dropdown: `key_property: "ID"`, `display_property: "ListName"` — stores UUID, shows name

### AppState RwLock Pattern
- Mutable fields wrapped in `RwLock`: `entities`, `metadata_xml`, `manifest_json`, `entity_manifests`, `apps_json`, `cdm_site_json`
- Immutable fields: `flp_html`, `settings`, `data_dir`, `data_store`
- Handlers acquire `.read().unwrap()` for reads; `activate_config()` acquires `.write().unwrap()`
- **Important**: Extract owned data from `ODataPath` before dropping `RwLock` guard (borrow checker safety)

### Draft Lifecycle
- `draftEdit` → creates draft copy (IsActiveEntity=false), marks active with HasDraftEntity=true
- `draftPrepare` → validates draft, returns it
- `draftActivate` → merges draft into active, removes draft
- Child entities are automatically copied/activated/removed with parent (via `copy_children_as_drafts`, `activate_children`, `remove_child_drafts`)
- Draft flag properties (`IsActiveEntity`, `HasActiveEntity`, `HasDraftEntity`) are annotated with `Core.Computed` so the UI treats them as server-managed and never includes them in create/edit forms or POST/PATCH payloads

### Generic Entities
- Configured via meta tables: `EntityConfigs`, `EntityFields`, `EntityFacets`, `EntityNavigations`, `EntityTableFacets`
- `create_generic_entities()` builds `GenericEntity` instances from config
- `activate_config()` rebuilds generic entities at runtime after metadata changes (triggered by `publishConfig`)
- Builtin sets (Products, Orders, etc.) are never replaced during rebuild

### FK Auto-Derivation (Generic Entities)
- For 1:1 navigation properties, `from_config()` auto-derives `text_path` and `value_list` on the FK field
- Convention: FK field `CustomerID` with nav `Customer → Customers` (title_path `CustomerName`) → auto-sets `text_path = "Customer/CustomerName"` and `value_list` pointing to `Customers`
- `create_generic_entities()` builds `title_paths` and `key_fields` lookup maps from all configs
- Auto-derivation only applies when `text_path` is `None` and `value_source` is `None` — explicit overrides take priority
- Effect: Fiori shows the customer name instead of the ID in display mode, and provides a selection dialog in edit mode
- `TextPath` field in EntityFields UI remains available as an explicit override for edge cases

### Value Text Resolution (Generic Entities)
- Fields with `value_source` (FieldValueList UUID) auto-generate a hidden computed `_text` field
- Convention: field `Status` with value_source → auto-generates `_Status_text` (computed, hidden)
- `convert_field()` in `generic.rs` sets `text_path: Some("_{name}_text")` when `value_source` is non-empty
- `from_config()` appends computed `FieldDef` entries for each `_text` field
- `Common.Text` annotation on the source field points to the `_text` field → Fiori shows Description instead of Code
- **Server-side resolution** populates `_text` fields at read time:
  - `query_collection_from()` in `query.rs` — builds `(ListID, Code) → Description` lookup from FieldValueListItems, injects into collection results
  - `resolve_value_texts()` in `data_store.rs` — same logic for single entity reads
- Fiori `Common.ValueListWithFixedValues` only controls dropdown vs dialog in edit mode; display-mode text requires `Common.Text` + server-side resolution

### CDM 3.1 Platform
- UShell boots in CDM platform mode via `Container.init("cdm")` — NOT local sandbox mode
- `build_cdm_site_json()` in `builders.rs` generates the CDM 3.1 site document from all entities with `apps_json_entry()`
- Site document served at `GET /cdm/site.json`, referenced by `services.CommonDataModel.adapter.config.siteDataUrl` in the FLP HTML
- Site structure: `applications` (with `crossNavigation.inbounds`), `visualizations` (StaticAppLauncher tiles), `pages` (single home page with one section), `vizTypes` (empty, auto-populated by UShell)
- `flp-init.js` is minimal (~100 lines): boots Container in `"cdm"` mode, applies company logo + user profile post-init
- `cdm_site_json` is a `RwLock<String>` field on `AppState`, rebuilt during `activate_config()` alongside metadata/manifest
- Spaces & pages enabled via `ushell.spaces.enabled: true` in the FLP HTML config
- Intent-based navigation works natively through CDM inbounds — no manual CSTR/NavTargetResolution wiring needed
- `/config/apps.json` endpoint retained for backward compatibility

## Build & Test
- `cargo test` — all tests must pass before committing
- `cargo build --release` — production binary (in-memory storage only)
- `cargo build --release --features postgres` — production binary with PostgreSQL support
- Server runs on port from `PORT` env var (default 8000)

## Storage Backends
- **In-memory** (default): data loaded from `data/*.json`, persisted on `commit()`
- **PostgreSQL** (feature `postgres`): set `DATABASE_URL` env var to activate
  - Schema auto-created on startup from `migrations/001_create_entity_records.sql`
  - Seeds data from `data/*.json` or `mock_data()` when entity set table is empty
  - All entity data stored as JSONB in `entity_records(entity_set, key_value, is_active, data)`
  - `docker-compose.yml` provided for local Postgres

## Production Features
- `GET /health` — health check endpoint returning `{"status":"ok"}`
- Graceful shutdown on SIGINT/SIGTERM
- Port configurable via `PORT` env var
- Structured logging via `tracing` (set `RUST_LOG=info` or `RUST_LOG=debug`)
