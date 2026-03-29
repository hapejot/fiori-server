use std::collections::HashMap;
use std::path::PathBuf;

use log::info;
use serde_json::Value;

use crate::builders;
use crate::data_store::{DataStore, InMemoryDataStore};
use crate::entity::ODataEntity;
use crate::settings::Settings;

/// Gesamtzustand der Applikation – haelt vorberechnete Artefakte
/// (Metadata-XML, manifest.json, FLP-HTML) und die Entity-Registry.
/// Die Mock-Daten liegen in einem RwLock-geschuetzten Store, damit
/// PATCH-Requests Aenderungen vornehmen koennen.
pub struct AppState {
    pub entities: Vec<&'static dyn ODataEntity>,
    pub metadata_xml: String,
    pub manifest_json: String,
    /// Per-Entity manifest.json: EntitySet-Name -> JSON-String.
    /// Jede Entitaet bekommt ein eigenes Manifest, bei dem ihre
    /// Liste die Default-Route ist.
    pub entity_manifests: HashMap<String, String>,
    pub flp_html: String,
    /// Dynamisch generierte apps.json (statische + generische Entitaeten zusammengefuehrt).
    pub apps_json: String,
    /// Mutable Data-Store (abstracted behind DataStore trait)
    pub data_store: Box<dyn DataStore>,
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    pub fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities.iter().find(|e| e.set_name() == set_name).copied()
    }
}

/// Builder fuer schrittweise Konfiguration des AppState.
pub struct AppStateBuilder {
    entities: Vec<&'static dyn ODataEntity>,
    settings: Option<Settings>,
    data_dir: Option<PathBuf>,
}

impl AppStateBuilder {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            settings: None,
            data_dir: None,
        }
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    pub fn entity(mut self, entity: &'static dyn ODataEntity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn build(self) -> AppState {
        let entities = self.entities;
        let settings = self.settings.unwrap_or_else(|| {
            Settings::load(std::path::Path::new("webapp/config/settings.json"))
        });
        let metadata_xml = builders::build_metadata_xml(&entities);
        let manifest_json =
            serde_json::to_string_pretty(&builders::build_manifest_json(&entities, &settings))
                .unwrap_or_default();

        // Per-Entity Manifeste: jede Entitaet bekommt ein Manifest,
        // bei dem sie die Default-Route ist.
        let mut entity_manifests = HashMap::new();
        for (idx, entity) in entities.iter().enumerate() {
            let manifest_val =
                builders::build_manifest_json_with_default(&entities, &settings, idx);
            entity_manifests.insert(
                entity.set_name().to_string(),
                serde_json::to_string_pretty(&manifest_val).unwrap_or_default(),
            );
        }

        let flp_html = builders::build_flp_html(&settings);

        // apps.json zusammenbauen: statische Datei (oder eincompiliert) + generische Entitaeten
        let apps_json = {
            let webapp_dir = std::env::current_dir().unwrap_or_default().join("webapp");
            let static_path = webapp_dir.join("config/apps.json");
            let base_json = if static_path.is_file() {
                std::fs::read_to_string(&static_path).ok()
            } else {
                info!("  [apps.json] Datei nicht gefunden -- verwende eincompilierte Version");
                Some(crate::EMBEDDED_APPS_JSON.to_string())
            };
            let mut apps: serde_json::Map<String, Value> = base_json
                .and_then(|c| serde_json::from_str::<Value>(&c).ok())
                .and_then(|v| v.get("applications").cloned())
                .and_then(|v| v.as_object().cloned())
                .unwrap_or_default();
            // Generische Entitaeten einfuegen
            for entity in &entities {
                if let Some((key, val)) = entity.apps_json_entry() {
                    apps.insert(key, val);
                }
            }
            let wrapper = serde_json::json!({ "applications": apps });
            serde_json::to_string_pretty(&wrapper).unwrap_or_default()
        };

        // Data-Verzeichnis
        let data_dir = self
            .data_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("data"));

        // Build the data store using the DataStore abstraction
        let data_store = Box::new(InMemoryDataStore::new(data_dir, entities.clone()));

        AppState {
            entities,
            metadata_xml,
            manifest_json,
            entity_manifests,
            flp_html,
            apps_json,
            data_store,
        }
    }
}