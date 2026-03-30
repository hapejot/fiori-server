# Copilot Instructions — fake-fiori-server

## Project Overview
Rust/Axum OData V4 mock server for SAP Fiori Elements. Simulates draft-enabled CRUD, batch requests, metadata (EDMX), and a Fiori Launchpad shell.

## Architecture

### Key Files
- `src/main.rs` — Server startup, route registration
- `src/app_state.rs` — Shared state with RwLock-wrapped mutable fields + `activate_config()`
- `src/data_store.rs` — In-memory data store with draft support, commit/persistence
- `src/handlers.rs` — HTTP handlers (collection, entity, batch, draft actions)
- `src/annotations.rs` — OData annotation XML generation, `FieldDef`, `ValueListDef`
- `src/entity.rs` — `ODataEntity` trait definition
- `src/entities/` — Entity implementations (products, orders, meta tables, generic entities)
- `src/builders.rs` — Build metadata XML, manifests, apps.json
- `src/routing.rs` — OData URL parsing
- `src/query.rs` — OData query execution ($filter, $orderby, $select, $expand)

### Entity Registration
1. Create struct implementing `ODataEntity` trait
2. Implement: `set_name`, `key_field`, `type_name`, `mock_data`, `entity_set`, `fields_def`
3. Register in `AppStateBuilder` via `.entity()`
4. Automatically included in EDMX, manifest.json, apps.json
5. Optional: JSON file in `data/` for persistence; falls back to `mock_data()`

### Data Flow
- `InMemoryDataStore::new()` loads from `data/<EntitySet>.json`, falls back to `mock_data()`
- Draft flags (`IsActiveEntity`, `HasActiveEntity`, `HasDraftEntity`) added on load
- `commit()` persists active records to `data/` as JSON (strips draft flags)

## Conventions

### Entity Key Fields
- All **new** entities use `ID` as key field with type `Edm.Guid`
- Existing entities (Orders, OrderItems, Products, etc.) keep their original key naming
- FieldValueListItem was the first entity using this convention
- `create_entity()` auto-generates the key as a random UUID v4 when not provided
- Mock data uses deterministic UUIDs via `value_list_id()` (UUID v5 from a fixed namespace + name)

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
- Every `FieldDef` instance must include `value_list` field (either `None` or `Some(&ValueListDef)`)
- `ValueListDef` supports flexible value help: `collection_path`, `key_property`, `display_property`, `fixed_values`
- When `value_list` is `Some`, annotation generation uses it; when `None`, falls back to `value_source` (classic FieldValueListItems path)
- **`value_source` stores the UUID** of the FieldValueList, not the list name
- Annotation emits `Common.ValueListParameterConstant` with `ListID` = the UUID to filter items
- EntityField.ValueSource dropdown: `key_property: "ID"`, `display_property: "ListName"` — stores UUID, shows name

### AppState RwLock Pattern
- Mutable fields wrapped in `RwLock`: `entities`, `metadata_xml`, `manifest_json`, `entity_manifests`, `apps_json`
- Immutable fields: `flp_html`, `settings`, `data_dir`, `data_store`
- Handlers acquire `.read().unwrap()` for reads; `activate_config()` acquires `.write().unwrap()`
- **Important**: Extract owned data from `ODataPath` before dropping `RwLock` guard (borrow checker safety)

### Draft Lifecycle
- `draftEdit` → creates draft copy (IsActiveEntity=false), marks active with HasDraftEntity=true
- `draftPrepare` → validates draft, returns it
- `draftActivate` → merges draft into active, removes draft
- Child entities are automatically copied/activated/removed with parent (via `copy_children_as_drafts`, `activate_children`, `remove_child_drafts`)

### Generic Entities
- Configured via meta tables: `EntityConfigs`, `EntityFields`, `EntityFacets`, `EntityNavigations`, `EntityTableFacets`
- `create_generic_entities()` builds `GenericEntity` instances from config
- `activate_config()` rebuilds generic entities at runtime after metadata changes (triggered by `publishConfig`)
- Builtin sets (Products, Orders, etc.) are never replaced during rebuild

## Build & Test
- `cargo test` — 181 tests, all must pass before committing
- `cargo build --release` — production binary
- Server runs on port from `config/settings.json` (default 3000)
