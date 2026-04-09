# Metadata Tables Reference

This document describes the meta tables that define **generic entities** at runtime.
Generic entities are created, configured, and published entirely through the Fiori UI
without writing Rust code.

## Overview

```
EntityConfigs (SetName)
 ├── 1:n → EntityFields (SetName)
 │         └── 1:1 → FieldValueLists (_ValueList, FK: ValueSource=UUID)
 ├── 1:n → EntityFacets (SetName)
 ├── 1:n → EntityNavigations (SetName)
 └── 1:n → EntityTableFacets (SetName)

FieldValueLists (ID: Guid)
 └── 1:n → FieldValueListItems (ListID → FieldValueLists.ID)
```

All child meta tables join to their parent via the **`SetName`** string field.
FieldValueLists/Items use GUID-based keys with `ListID` as the FK.

### Lifecycle

1. Create/edit meta table entries via the Fiori UI (EntityConfigs app)
2. Trigger **`publishConfig`** action on the EntityConfig record
3. Server calls `activate_config()` → persists data, reconstructs configs
   from `data/*.json`, rebuilds EDMX/manifest, and hot-swaps entities
4. New entity is immediately available — no server restart needed

---

## EntityConfigs

Master configuration record for each generic entity. One record per entity set.

| Field | Type | Key/Immutable | Description |
|-------|------|:---:|-------------|
| `SetName` | Edm.String(40) | **KEY**, immutable | Entity set name (e.g. `"Customers"`). Used to join all child tables. |
| `KeyField` | Edm.String(40) | | Name of the key property. If `"ID"` → auto Edm.Guid with computed key. Otherwise the field must exist in EntityFields. |
| `TypeName` | Edm.String(40) | | OData entity type name (e.g. `"Customer"`) → EDMX `<EntityType Name="Customer">` |
| `ParentSetName` | Edm.String(40) | | If non-empty, marks this entity as a **composition child** of the named parent set. Enables draft propagation and nested ObjectPage routing. |
| `TileTitle` | Edm.String(80) | | Fiori Launchpad tile title. Leave empty for no tile (e.g. child entities). |
| `TileDescription` | Edm.String(120) | | Tile subtitle text. |
| `TileIcon` | Edm.String(80) | | SAP icon URI (e.g. `"sap-icon://customer"`). |
| `HeaderTypeName` | Edm.String(40) | | `UI.HeaderInfo/TypeName` — singular label on the ObjectPage (e.g. `"Kunde"`). |
| `HeaderTypeNamePlural` | Edm.String(40) | | `UI.HeaderInfo/TypeNamePlural` — plural label on the ListReport (e.g. `"Kunden"`). |
| `HeaderTitlePath` | Edm.String(40) | | Field path for `UI.HeaderInfo/Title`. Also becomes `title_field()` → drives `Common.Text` on the key. |
| `HeaderDescriptionPath` | Edm.String(40) | | Field path for `UI.HeaderInfo/Description`. |
| `SelectionFields` | Edm.String(200) | | Comma-separated field names → `UI.SelectionFields` (filter bar on ListReport). |

**Example:**
```json
{
  "SetName": "Customers",
  "KeyField": "CustomerID",
  "TypeName": "Customer",
  "ParentSetName": "",
  "TileTitle": "Kunden",
  "TileDescription": "Kundenübersicht",
  "TileIcon": "sap-icon://customer",
  "HeaderTypeName": "Kunde",
  "HeaderTypeNamePlural": "Kunden",
  "HeaderTitlePath": "CustomerName",
  "HeaderDescriptionPath": "CustomerID",
  "SelectionFields": "City,Country"
}
```

### Key field convention

| `KeyField` value | Effect |
|------------------|--------|
| `"ID"` | Auto-generates an `Edm.Guid` key field at position 0, marked `computed: true`. UUID is server-generated — never shown in create/edit forms. |
| Anything else (e.g. `"CustomerID"`) | The named field must exist in EntityFields. It becomes the key as-is. If the field is user-editable, it appears in the creation dialog. |

---

## EntityFields

Defines each property/column of a generic entity.

| Field | Type | Key/Immutable | Description |
|-------|------|:---:|-------------|
| `FieldID` | Edm.String(80) | **KEY**, immutable | Convention: `{SetName}_{FieldName}` (e.g. `"Customers_Email"`). |
| `SetName` | Edm.String(40) | immutable | FK to parent EntityConfigs. |
| `FieldName` | Edm.String(40) | | Property name → EDMX `<Property Name="...">`. |
| `Label` | Edm.String(80) | | Display label → `Common.Label` annotation. |
| `EdmType` | Edm.String(30) | | OData type (see table below). Has a built-in value help dropdown. |
| `MaxLength` | Edm.Int32 | | `MaxLength` facet on `Edm.String` properties. |
| `Precision` | Edm.Int32 | | `Precision` facet on `Edm.Decimal` properties. |
| `Scale` | Edm.Int32 | | `Scale` facet on `Edm.Decimal` properties. |
| `IsImmutable` | Edm.Boolean | | `true` → `Core.Immutable` — editable at creation, read-only afterward. |
| `IsComputed` | Edm.Boolean | | `true` → `Core.Computed` — field is server-generated, never shown in create/edit forms. |
| `SemanticObject` | Edm.String(40) | | Enables intent-based navigation (e.g. `"Customers"` → opens Customers app). |
| `ValueSource` | Edm.String(40) | | **UUID** of a FieldValueList. Generates dropdown and auto `_text` field. (See [Value Lists](#value-text-resolution) below.) |
| `TextPath` | Edm.String(80) | | Explicit text path override for `Common.Text` annotation (e.g. `"Customer/CustomerName"`). When empty, auto-derived from navigation properties. |
| `DefaultValue` | Edm.String(120) | | Default value for new records. Applied during entity creation when the field is not provided in the request body. |
| `SortOrder` | Edm.Int32 | | Controls order in EDMX and FieldGroups. |
| `ShowInLineItem` | Edm.Boolean | | `true` → field appears as a column in the ListReport table (`UI.LineItem`). |
| `LineItemImportance` | Edm.String(10) | | `UI.Importance` on the column (e.g. `"High"` — always visible on narrow screens). |
| `LineItemLabel` | Edm.String(80) | | Override label for the list column only. Empty = use `Label`. |
| `LineItemCriticalityPath` | Edm.String(40) | | Path to a criticality field → colored status indicator in the list column (e.g. `"StatusCriticality"`). |
| `LineItemSemanticObject` | Edm.String(40) | | SemanticObject on list column → `UI.DataFieldWithIntentBasedNavigation`. |

### Supported EdmType values

| Code | Description |
|------|-------------|
| `Edm.String` | String (requires `MaxLength`) |
| `Edm.Int32` | Integer |
| `Edm.Int64` | Long integer |
| `Edm.Decimal` | Decimal (set `Precision` and `Scale`) |
| `Edm.Boolean` | Boolean (checkbox) |
| `Edm.DateTimeOffset` | Date and time |
| `Edm.Date` | Date only |
| `Edm.Guid` | GUID (auto-hidden in UI) |
| `Edm.Byte` | Small integer (0–255) |

**Example:**
```json
{
  "FieldID": "Customers_Email",
  "SetName": "Customers",
  "FieldName": "Email",
  "Label": "E-Mail",
  "EdmType": "Edm.String",
  "MaxLength": 120,
  "Precision": null,
  "Scale": null,
  "IsImmutable": false,
  "IsComputed": false,
  "SemanticObject": "",
  "ValueSource": "",
  "TextPath": "",
  "DefaultValue": "",
  "SortOrder": 2,
  "ShowInLineItem": true,
  "LineItemImportance": "",
  "LineItemLabel": "",
  "LineItemCriticalityPath": "",
  "LineItemSemanticObject": ""
}
```

---

## EntityFacets

Defines ObjectPage sections — each maps to a `UI.Facets` reference pointing to a `UI.FieldGroup`.

| Field | Type | Key/Immutable | Description |
|-------|------|:---:|-------------|
| `FacetID` | Edm.String(80) | **KEY**, immutable | Convention: `{SetName}_{SectionId}`. |
| `SetName` | Edm.String(40) | immutable | FK to parent EntityConfigs. |
| `SectionLabel` | Edm.String(80) | | Section heading on the ObjectPage. |
| `SectionId` | Edm.String(40) | | ReferenceFacet `ID` attribute (unique within entity). |
| `FieldGroupQualifier` | Edm.String(40) | | Links to `UI.FieldGroup#<Qualifier>`. Must be unique per entity. |
| `FieldGroupLabel` | Edm.String(80) | | Label on the FieldGroup. |
| `FieldGroupFields` | Edm.String(500) | | **Comma-separated field names** for this section. Each becomes a `UI.DataField`. Order matters. |
| `SortOrder` | Edm.Int32 | | Display order of sections on the ObjectPage. |

**Example:**
```json
{
  "FacetID": "Customers_ContactInfo",
  "SetName": "Customers",
  "SectionLabel": "Kontaktdaten",
  "SectionId": "ContactInfo",
  "FieldGroupQualifier": "Contact",
  "FieldGroupLabel": "Kontakt",
  "FieldGroupFields": "CustomerID,CustomerName,Email,Phone",
  "SortOrder": 0
}
```

---

## EntityNavigations

Defines navigation properties (associations and compositions).

| Field | Type | Key/Immutable | Description |
|-------|------|:---:|-------------|
| `NavID` | Edm.String(80) | **KEY**, immutable | Convention: `{SetName}_{NavName}`. |
| `SetName` | Edm.String(40) | immutable | FK to parent EntityConfigs. |
| `NavName` | Edm.String(40) | | Navigation property name (e.g. `"Contacts"`, `"Customer"`). |
| `TargetType` | Edm.String(40) | | Target entity type (e.g. `"Contact"`) → `Type` attribute in EDMX. |
| `TargetSet` | Edm.String(40) | | Target entity set (e.g. `"Contacts"`) → `NavigationPropertyBinding Target`. |
| `IsCollection` | Edm.Boolean | | `true` = 1:n (composition/collection), `false` = 1:1 (reference). |
| `ForeignKey` | Edm.String(40) | | **For 1:n:** FK field on the *child* entity referencing our key. **For 1:1:** FK field on *this* entity pointing to the target's key. |
| `SortOrder` | Edm.Int32 | | Order in EDMX output. |

### Navigation pattern examples

**1:n composition** (parent → children):
```json
{
  "NavName": "Contacts",
  "TargetType": "Contact",
  "TargetSet": "Contacts",
  "IsCollection": true,
  "ForeignKey": "CustomerID"
}
```
→ Customers.ID is the key, Contacts.CustomerID is the FK pointing back.

**1:1 reference** (child → parent):
```json
{
  "NavName": "Customer",
  "TargetType": "Customer",
  "TargetSet": "Customers",
  "IsCollection": false,
  "ForeignKey": "CustomerID"
}
```
→ Contacts.CustomerID on this entity joins to Customers.ID.

---

## EntityTableFacets

Defines table sections on the ObjectPage that display child collections.

| Field | Type | Key/Immutable | Description |
|-------|------|:---:|-------------|
| `TableFacetID` | Edm.String(80) | **KEY**, immutable | Convention: `{SetName}_{FacetId}`. |
| `SetName` | Edm.String(40) | immutable | FK to parent EntityConfigs. |
| `FacetLabel` | Edm.String(80) | | Section heading for the table. |
| `FacetId` | Edm.String(40) | | ReferenceFacet `ID` attribute. |
| `NavigationProperty` | Edm.String(40) | | Name of the navigation property to render as a table. Must match an EntityNavigation's `NavName`. |
| `SortOrder` | Edm.Int32 | | Order among table sections. |

**Example:**
```json
{
  "TableFacetID": "Customers_ContactsTable",
  "SetName": "Customers",
  "FacetLabel": "Ansprechpartner",
  "FacetId": "ContactsSection",
  "NavigationProperty": "Contacts",
  "SortOrder": 0
}
```

This renders the `Contacts` collection as a table on the Customers ObjectPage, using the child entity's `UI.LineItem` for column definitions.

---

## FieldValueLists & FieldValueListItems

Reusable code lists for dropdowns.

### FieldValueLists

| Field | Type | Computed | Description |
|-------|------|:---:|-------------|
| `ID` | Edm.Guid | yes | Auto-generated key. |
| `ListName` | Edm.String(40) | | Unique name (e.g. `"StatusCodes"`). |
| `Description` | Edm.String(120) | | Human-readable description. |

### FieldValueListItems

| Field | Type | Computed | Description |
|-------|------|:---:|-------------|
| `ID` | Edm.Guid | yes | Auto-generated key. |
| `ListID` | Edm.Guid | | FK to `FieldValueLists.ID`. |
| `Code` | Edm.String(40) | | The stored value (e.g. `"A"`, `"P"`). |
| `Description` | Edm.String(120) | | Display text in the dropdown (e.g. `"Active"`, `"Pending"`). |
| `SortOrder` | Edm.Int32 | | Order in the dropdown. |

### Value Text Resolution

When an EntityField's **`ValueSource`** is set to a FieldValueList UUID:

1. At **build time** — a hidden computed field `_{FieldName}_text` is auto-generated (e.g. field `Status` → `_Status_text`)
2. At **read time** — the server resolves `Code` → `Description` from FieldValueListItems and injects the text into `_Status_text`
3. The `Common.Text` annotation on `Status` points to `_Status_text` with `TextArrangement/TextOnly` — Fiori shows "Active" instead of "A"
4. In edit mode — `Common.ValueListWithFixedValues` renders a dropdown with Code/Description pairs

**Important:** `ValueSource` stores the **UUID** of the FieldValueList, not the list name. The entity field's value help dropdown shows `ListName` but stores `ID`.

---

## Complete Example: Defining a "Customers" Entity

**EntityConfigs** record:
- `SetName: "Customers"`, `KeyField: "CustomerID"`, `TypeName: "Customer"`
- `HeaderTitlePath: "CustomerName"` → shows name instead of ID everywhere
- `SelectionFields: "City,Country"` → filter bar

**EntityFields** records (8 total):
- `CustomerID` — Edm.String, immutable, ShowInLineItem=true, Importance=High
- `CustomerName` — Edm.String, ShowInLineItem=true
- `Email`, `Phone` — Edm.String, ShowInLineItem=true
- `Street`, `City`, `PostalCode`, `Country` — Edm.String

**EntityFacets** (2 sections):
- "Kontaktdaten" → FieldGroup `Contact` with fields `CustomerID,CustomerName,Email,Phone`
- "Adresse" → FieldGroup `Address` with fields `Street,City,PostalCode,Country`

**EntityNavigations** (1 nav):
- `Contacts` → Contacts entity set (1:n collection, FK: `CustomerID`)

**EntityTableFacets** (1 table):
- "Ansprechpartner" table showing `Contacts` navigation property

After `publishConfig`, this produces a fully functional CRUD app with:
- ListReport with filterable table
- ObjectPage with two form sections and a contacts table
- Draft editing support
- Launchpad tile
