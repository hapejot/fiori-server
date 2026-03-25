use std::collections::HashMap;
use std::sync::RwLock;

use crate::builders;
use crate::entity::ODataEntity;
use crate::settings::Settings;
use serde_json::Value;

/// Gesamtzustand der Applikation – haelt vorberechnete Artefakte
/// (Metadata-XML, manifest.json, FLP-HTML) und die Entity-Registry.
/// Die Mock-Daten liegen in einem RwLock-geschuetzten Store, damit
/// PATCH-Requests Aenderungen vornehmen koennen.
pub struct AppState {
    pub entities: Vec<&'static dyn ODataEntity>,
    pub metadata_xml: String,
    pub manifest_json: String,
    pub flp_html: String,
    /// Mutable Data-Store: EntitySet-Name -> Vec<Value>
    pub data_store: RwLock<HashMap<String, Vec<Value>>>,
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
}

impl AppStateBuilder {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            settings: None,
        }
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
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
        let flp_html = builders::build_flp_html(&settings);

        // Mutable Data-Store aus den Mock-Daten befuellen
        // Draft-Properties werden automatisch hinzugefuegt
        let mut store = HashMap::new();
        for entity in &entities {
            let mut records = entity.mock_data();
            for record in &mut records {
                if let Some(obj) = record.as_object_mut() {
                    obj.insert("IsActiveEntity".to_string(), serde_json::Value::Bool(true));
                    obj.insert("HasActiveEntity".to_string(), serde_json::Value::Bool(false));
                    obj.insert("HasDraftEntity".to_string(), serde_json::Value::Bool(false));
                }
            }
            store.insert(entity.set_name().to_string(), records);
        }

        AppState {
            entities,
            metadata_xml,
            manifest_json,
            flp_html,
            data_store: RwLock::new(store),
        }
    }
}
