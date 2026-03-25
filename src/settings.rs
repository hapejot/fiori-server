use serde::Deserialize;
use std::path::Path;

/// Konfiguration aus webapp/config/settings.json.
/// Felder werden beim Serverstart gelesen und in die HTML-Generierung injiziert.
#[derive(Debug, Clone)]
pub struct Settings {
    pub ui5_version: String,
    pub theme: String,
    pub language: String,
    pub compat_version: String,
    pub libs: Vec<String>,
    pub renderer: String,
    pub root_intent: String,
    pub enable_search: bool,
    pub component_id: String,
    pub resource_root: String,
}

#[derive(Deserialize)]
struct RawSettings {
    ui5: Option<Ui5Block>,
    shell: Option<ShellBlock>,
    component: Option<ComponentBlock>,
}

#[derive(Deserialize)]
struct Ui5Block {
    version: Option<String>,
    theme: Option<String>,
    language: Option<String>,
    #[serde(rename = "compatVersion")]
    compat_version: Option<String>,
    libs: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ShellBlock {
    renderer: Option<String>,
    #[serde(rename = "rootIntent")]
    root_intent: Option<String>,
    #[serde(rename = "enableSearch")]
    enable_search: Option<bool>,
}

#[derive(Deserialize)]
struct ComponentBlock {
    id: Option<String>,
    #[serde(rename = "resourceRoot")]
    resource_root: Option<String>,
}

impl Settings {
    /// Liest settings.json vom angegebenen Pfad.
    /// Bei Fehler (Datei fehlt, Parse-Fehler) werden Defaults verwendet.
    pub fn load(path: &Path) -> Self {
        let raw: Option<RawSettings> = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

        match raw {
            Some(r) => Self::from_raw(r),
            None => {
                println!(
                    "  [settings] {} nicht gefunden oder ungueltig – verwende Defaults",
                    path.display()
                );
                Self::defaults()
            }
        }
    }

    fn defaults() -> Self {
        Self {
            ui5_version: "1.139.0".into(),
            theme: "sap_horizon".into(),
            language: "de".into(),
            compat_version: "edge".into(),
            libs: vec![
                "sap.m".into(),
                "sap.ushell".into(),
                "sap.fe.templates".into(),
                "sap.f".into(),
            ],
            renderer: "fiori2".into(),
            root_intent: "Shell-home".into(),
            enable_search: false,
            component_id: "products.demo".into(),
            resource_root: "../".into(),
        }
    }

    fn from_raw(r: RawSettings) -> Self {
        let d = Self::defaults();
        let ui5 = r.ui5.unwrap_or(Ui5Block {
            version: None,
            theme: None,
            language: None,
            compat_version: None,
            libs: None,
        });
        let shell = r.shell.unwrap_or(ShellBlock {
            renderer: None,
            root_intent: None,
            enable_search: None,
        });
        let comp = r.component.unwrap_or(ComponentBlock {
            id: None,
            resource_root: None,
        });

        Self {
            ui5_version: ui5.version.unwrap_or(d.ui5_version),
            theme: ui5.theme.unwrap_or(d.theme),
            language: ui5.language.unwrap_or(d.language),
            compat_version: ui5.compat_version.unwrap_or(d.compat_version),
            libs: ui5.libs.unwrap_or(d.libs),
            renderer: shell.renderer.unwrap_or(d.renderer),
            root_intent: shell.root_intent.unwrap_or(d.root_intent),
            enable_search: shell.enable_search.unwrap_or(d.enable_search),
            component_id: comp.id.unwrap_or(d.component_id),
            resource_root: comp.resource_root.unwrap_or(d.resource_root),
        }
    }
}
