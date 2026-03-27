# FLP & UShell Architecture — Lokaler Sandbox-Modus (UI5 1.139.0)

## Übersicht

Die lokale FLP-Sandbox bootet den SAP Unified Shell im `local`-Modus (`Container.init("local")`).
Dabei werden die `local`-Adapter geladen (z.B. `sap/ushell/adapters/local/...`), nicht die ABAP-Adapter.

---

## Boot-Sequenz

1. **`flp-init.js`** wird per `<script>` vor `sap-ui-core.js` geladen
2. Setzt `window["sap-ui-config"]["xx-bootTask"]` — UI5 ruft diese Funktion beim Core-Init auf
3. `xx-bootTask` → `init()` → `bootstrap(fnCallback)`:
   - Liest `apps.json` via synchronem XHR (`loadApplications()`)
   - Baut `oUshellConfig.applications` map auf (Key = `"SemanticObject-action"`)
   - Ruft `commonUtils.migrateV2ServiceConfig()` auf → migriert alte Services zu V2-Services
   - Ruft `adjustApplicationConfiguration()` auf → baut Tiles, NavTargetResolution-Config
   - Kopiert Apps in alle drei Resolution-Services
   - `ushellUtils.requireAsync(["sap/ushell/Container"])` → `Container.init("local")`

### Dateien
- **`webapp/flp-init.js`** — Komplette Boot-Logik
- **`webapp/config/apps.json`** — App-Definitionen (semanticObject, action, title, icon)
- **`webapp/config/settings.json`** — UI5-Version, Theme, Sprache, Component-ID
- **`src/builders.rs`** → `build_flp_html()` — Generiert `flp.html` mit inline `sap-ushell-config`

---

## Service-Hierarchie für Intent-Navigation

FE V4 nutzt **nicht** direkt `CrossApplicationNavigation`, sondern den `Navigation`-V2-Service:

```
FE V4 Component
  └─ ShellServicesFactory (sap/fe/core/services/ShellServicesFactory)
       ├─ ShellServices (real, wenn Container vorhanden)
       │    └─ this.applicationNavigation = Navigation-Service
       └─ ShellServiceMock (wenn kein Container → getLinks() liefert immer [])
            └─ getLinks() → Promise.resolve([])  ← KEIN Intent-Support!
```

### ShellServicesFactory — Entscheidung real vs. mock
**Datei:** `sap/fe/core/services/ShellServicesFactory-dbg.js`

```js
const shellService = serviceContext.settings.shellContainer
    ? new ShellServices(serviceContext)   // real — Shell vorhanden
    : new ShellServiceMock(serviceContext); // mock — alles leer
```

Danach ruft die Factory `__fetchSemanticObject()` auf und speichert die Ergebnisse in
`internalModel.setProperty("/semanticObjects", semanticObjects)`.

---

## Navigation.getLinks() — Der kritische Pfad

**Datei:** `sap/ushell/services/Navigation-dbg.js`

```
Navigation.getLinks(aLinkFilter)
  └─ für jeden Filter:
       └─ NavTargetResolutionInternal.getLinks(oArgs)
```

### NavTargetResolutionInternal.getLinks() — Verzweigung
**Datei:** `sap/ushell/services/NavTargetResolutionInternal-dbg.js`

```js
this._isClientSideTargetResolutionEnabled = function () {
    return !!(oServiceConfig && oServiceConfig.enableClientSideTargetResolution);
};
```

#### CSTR enabled?

```js
sap.ushell.Container.getServiceAsync("NavTargetResolutionInternal").then(s => console.log("CSTR enabled:", s._isClientSideTargetResolutionEnabled()))
```


**Mit CSTR enabled:**
```
NavTargetResolutionInternal.getLinks()
  └─ _getLinksClientSide()
       └─ ClientSideTargetResolution.getLinks(oArgs)  ← korrekt!
```

**OHNE CSTR enabled (DEFAULT!):**
```
NavTargetResolutionInternal.getLinks()
  └─ _getGetLinksResolver() → Fallback:
       ├─ oAdapter.getLinks() (falls vorhanden)
       └─ oAdapter.getSemanticObjectLinks() (Legacy-Fallback)
            └─ ShellNavigationInternal.hrefForExternal() → problematisch in lokaler Sandbox
```

### WICHTIG: `enableClientSideTargetResolution` muss explizit gesetzt werden!
```js
setNestedValue(oUshellConfig,
    "services.NavTargetResolutionInternal.config.enableClientSideTargetResolution",
    true);
```

Ohne dieses Flag: `getLinks()` liefert leere Ergebnisse → FE V4 rendert IBN-Felder als Text statt Link.

---

## isIntentSupported() vs. getLinks() — Verschiedene Codepfade!

- `isIntentSupported()` → `NavTargetResolutionInternal._isIntentSupported()` → CSTR oder Adapter
- `getLinks()` → `NavTargetResolutionInternal.getLinks()` → **nur mit Flag** → CSTR

`isIntentSupported()` kann `true` liefern, während `getLinks()` leer bleibt!
Das erklärt, warum die Console "supported: true" zeigte, aber kein Link gerendert wurde.

---

## ClientSideTargetResolution Adapter (lokal)
**Datei:** `sap/ushell/adapters/local/ClientSideTargetResolutionAdapter-dbg.js`

Akzeptiert zwei Formate in der Config:
1. **`config.inbounds`** — CDM/App-Descriptor-Format (ab 1.34, empfohlen)
2. **`config.applications`** — Legacy-Format (Key = `"SO-action"`, Value = App-Config)

Bei Legacy-Format: `_transformApplicationsToInbounds()` konvertiert automatisch:
- Key `"Products-display"` → `semanticObject: "Products"`, `action: "display"`
- `additionalInformation: "SAPUI5.Component=..."` → `applicationType: "SAPUI5"`, `ui5ComponentName: ...`

### Resolution-Prozess
```
CSTR.getLinks({semanticObject: "Products"})
  └─ getInbounds() → alle registrierten Inbounds
       └─ Match auf semanticObject + action + params
            └─ Ergebnis: [{intent: "#Products-display", text: "..."}]
```

```
CSTR.resolveHashFragment("#Products-display")
  └─ Inbound lookup → resolutionResult
       └─ {applicationType, url, additionalInformation, ui5ComponentName, ...}
```

---

## NavTargetResolution Adapter (lokal) — Legacy
**Datei:** `sap/ushell/adapters/local/NavTargetResolutionAdapter-dbg.js`

Delegiert direkt an `NavTargetResolutionInternalAdapter`.

**Datei:** `sap/ushell/adapters/local/NavTargetResolutionInternalAdapter-dbg.js`

Einfache Key-Lookup in `oApplications`:
- `resolveHashFragment("#Products-display")` → `oApplications["Products-display"]`
- `getSemanticObjectLinks("Products")` → alle Keys die mit `"Products-"` beginnen
- `isIntentSupported(["#Products-display"])` → `resolveHashFragment()` pro Intent

---

## V2 Service Migration
**Datei:** `sap/ushell/bootstrap/common/common.util-dbg.js`

`migrateV2ServiceConfig()` und `getV2ServiceMigrationConfig()` kopieren Config:

| Von (V1)                         | Nach (V2)                              |
|----------------------------------|----------------------------------------|
| services.LaunchPage              | services.FlpLaunchPage                 |
| services.NavTargetResolution     | services.NavTargetResolutionInternal   |
| services.CrossApplicationNavigation | services.Navigation                 |
| services.ShellNavigation         | services.ShellNavigationInternal       |
| services.Bookmark                | services.BookmarkV2                    |
| services.Personalization         | services.PersonalizationV2             |
| services.Notifications           | services.NotificationsV2               |

**Reihenfolge in flp-init.js:**
1. `loadApplications()` → füllt `oUshellConfig.applications`
2. `migrateV2ServiceConfig()` → INPLACE-Migration (kopiert NTR → NTRI)
3. `adjustApplicationConfiguration()` → verschiebt `applications` nach `services.NavTargetResolution.adapter.config.applications`, baut Tiles/Groups
4. Manuelles Kopieren der Apps in NTRI + CSTR

---

## UShell Config Struktur (nach Bootstrap)

```json
{
  "defaultRenderer": "fiori2",
  "renderers": { "fiori2": { "componentData": { "config": { "enableSearch": false, "rootIntent": "Shell-home" }}}},
  "services": {
    "NavTargetResolution": {
      "adapter": { "config": { "applications": { "Products-display": {...}, "Orders-display": {...} }}}
    },
    "NavTargetResolutionInternal": {
      "adapter": { "config": { "applications": { ... }}},
      "config": { "enableClientSideTargetResolution": true }
    },
    "ClientSideTargetResolution": {
      "adapter": { "config": { "applications": { ... }}}
    },
    "LaunchPage": {
      "adapter": { "config": { "groups": [{ "id": "flp_default_group", "tiles": [...] }] }}
    },
    "FlpLaunchPage": { ... },
    "Ui5ComponentLoader": { "config": { "loadDefaultDependencies": false }}
  }
}
```

---

## Applications-Eintrag Format

```json
{
  "Products-display": {
    "additionalInformation": "SAPUI5.Component=products.demo",
    "applicationType": "URL",
    "url": "./",
    "title": "Produkte",
    "description": "Produktübersicht"
  }
}
```

**apps.json** enthält zusätzlich `semanticObject`, `action`, `icon` — diese werden von
`loadApplications()` **nicht** in den UShell-Config-Eintrag übernommen (nur title, description, url).
Der `semanticObject` + `action` steckt implizit im Key (`"Products-display"`).

---

## FE V4 Intent-Based Navigation (Annotations)

### Benötigte Annotations im $metadata

1. **LineItem — DataFieldWithIntentBasedNavigation:**
```xml
<Record Type="UI.DataFieldWithIntentBasedNavigation">
  <PropertyValue Property="Value" Path="ProductID"/>
  <PropertyValue Property="Label" String="Produkt-ID"/>
  <PropertyValue Property="SemanticObject" String="Products"/>
  <PropertyValue Property="Action" String="display"/>
</Record>
```

2. **FieldGroup — DataFieldWithIntentBasedNavigation** (gleich wie LineItem)

3. **Property-Level Common.SemanticObject:**
```xml
<Annotations Target="ProductsService.Order/ProductID">
  <Annotation Term="Common.SemanticObject" String="Products"/>
</Annotations>
```

### FE V4 Ablauf zur Laufzeit
1. FE V4 liest Annotations aus $metadata
2. Findet `DataFieldWithIntentBasedNavigation` für ProductID
3. Ruft `ShellServices.getLinksWithCache([{semanticObject: "Products"}])` auf
4. → `Navigation.getLinks()` → `NavTargetResolutionInternal.getLinks()` → CSTR
5. Wenn Links zurückkommen → rendert als `<a>` Link
6. Wenn keine Links → rendert als normaler Text
7. Klick → `ShellServices.navigate({target: {semanticObject: "Products", action: "display"}, params: {ProductID: "..."}})`

### Dateien (Server-seitig)
- **`src/annotations.rs`** — `FieldDef.semantic_object`, `LineItemField.semantic_object`, XML-Generation
- **`src/entities/order.rs`** — `ProductID` hat `semantic_object: Some("Products")`
- **`src/entities/product.rs`** — alle Felder `semantic_object: None`

---

## Debugging-Tipps

### Console-Check: Apps in allen Services?
```js
console.log(window["sap-ushell-config"]);
```

### Console-Check: Intent unterstützt?
```js
sap.ushell.Container.getServiceAsync("CrossApplicationNavigation").then(s =>
    s.isIntentSupported(["#Products-display"]).then(r => console.log(r)));
```

### Console-Check: getLinks (der FE-V4-Pfad)?
```js
sap.ushell.Container.getServiceAsync("Navigation").then(s =>
    s.getLinks([{semanticObject: "Products"}])
        .then(r => console.log(r)));
```

### Console-Check: CSTR direkt?
```js
sap.ushell.Container.getServiceAsync("ClientSideTargetResolution").then(s =>
    s.getLinks({semanticObject: "Products"}).then(r => console.log(r)));
```

### Console-Check: ShellServices Instanz-Typ?
```js
// Auf der FE-App (nach Navigation in eine App):
sap.ui.getCore().byId("container-products.demo")?.getShellServices?.()?.instanceType
// "real" = Shell vorhanden, "mock" = kein Shell → getLinks() immer leer
```

```js
sap.ui.core.Element.registry.filter(e => e.getMetadata?.().getName?.() === "sap.fe.core.AppComponent").map(c => ({id: c.getId(), shellType: c.getShellServices?.()?.instanceType}))

Object.keys(sap.ui.core.Component.registry.all()).map(id => { const c = sap.ui.core.Component.registry.get(id); return {id: id, name: c.getMetadata().getName()} })

```

```js
var app = sap.ui.core.Component.registry.get("application-Orders-display-component");
var ss = app.getShellServices();
console.log("instanceType:", ss.instanceType);
console.log("getLinksWithCache:", typeof ss.getLinksWithCache);
ss.getLinksWithCache([{semanticObject: "Products", action: "display"}]).then(r => console.log("result:", JSON.stringify(r)));
```

This will tell us:

Whether FE V4 is using the real ShellServices or the ShellServiceMock
Whether getLinksWithCache returns actual results from within the FE context