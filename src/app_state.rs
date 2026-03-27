use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use log::info;
use serde_json::Value;

use crate::builders;
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
    /// Mutable Data-Store: EntitySet-Name -> Vec<Value>
    pub data_store: RwLock<HashMap<String, Vec<Value>>>,
    /// Verzeichnis fuer persistente JSON-Dateien
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    pub fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities.iter().find(|e| e.set_name() == set_name).copied()
    }

    /// Speichert alle Entity-Daten (nur aktive, ohne Drafts) in JSON-Dateien.
    pub fn save_data(&self) {
        let store = self.data_store.read().unwrap();
        for entity in &self.entities {
            let set_name = entity.set_name();
            if let Some(records) = store.get(set_name) {
                // Nur aktive Entities speichern (Drafts sind transient)
                let active: Vec<&Value> = records
                    .iter()
                    .filter(|r| {
                        r.get("IsActiveEntity")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true)
                    })
                    .collect();
                // Draft-Felder vor dem Speichern entfernen
                let clean: Vec<Value> = active
                    .into_iter()
                    .map(|r| {
                        let mut c = r.clone();
                        if let Some(obj) = c.as_object_mut() {
                            obj.remove("IsActiveEntity");
                            obj.remove("HasActiveEntity");
                            obj.remove("HasDraftEntity");
                        }
                        c
                    })
                    .collect();
                let json_path = self.data_dir.join(format!("{}.json", set_name));
                if let Ok(content) = serde_json::to_string_pretty(&clean) {
                    if let Err(e) = std::fs::write(&json_path, content) {
                        eprintln!("  WARNUNG: Konnte {} nicht schreiben: {}", json_path.display(), e);
                    }
                }
            }
        }
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

        // Data-Verzeichnis: JSON-Dateien haben Vorrang vor mock_data()
        let data_dir = self
            .data_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("data"));

        // Mutable Data-Store befuellen
        // Draft-Properties werden automatisch hinzugefuegt
        let mut store = HashMap::new();
        for entity in &entities {
            let set_name = entity.set_name();
            let mut records = load_entity_data(set_name, &data_dir, *entity);
            for record in &mut records {
                if let Some(obj) = record.as_object_mut() {
                    obj.entry("IsActiveEntity".to_string())
                        .or_insert(serde_json::Value::Bool(true));
                    obj.entry("HasActiveEntity".to_string())
                        .or_insert(serde_json::Value::Bool(false));
                    obj.entry("HasDraftEntity".to_string())
                        .or_insert(serde_json::Value::Bool(false));
                }
            }
            store.insert(set_name.to_string(), records);
        }

        AppState {
            entities,
            metadata_xml,
            manifest_json,
            entity_manifests,
            flp_html,
            data_store: RwLock::new(store),
            data_dir: data_dir.to_path_buf(),
        }
    }
}

/// Laedt Entity-Daten: zuerst aus `data/{set_name}.json`, dann Fallback auf mock_data().
fn load_entity_data(
    set_name: &str,
    data_dir: &Path,
    entity: &dyn ODataEntity,
) -> Vec<Value> {
    let json_path = data_dir.join(format!("{}.json", set_name));
    if json_path.is_file() {
        match std::fs::read_to_string(&json_path) {
            Ok(content) => match serde_json::from_str::<Vec<Value>>(&content) {
                Ok(records) => {
                    info!(
                        "  {} : {} Eintraege aus {}",
                        set_name,
                        records.len(),
                        json_path.display()
                    );
                    return records;
                }
                Err(e) => {
                    eprintln!(
                        "  WARNUNG: {} ist kein gueltiges JSON-Array: {} – Fallback auf mock_data()",
                        json_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "  WARNUNG: Konnte {} nicht lesen: {} – Fallback auf mock_data()",
                    json_path.display(),
                    e
                );
            }
        }
    } else {
        info!("  {} : mock_data() (keine Datei {})", set_name, json_path.display());
    }
    entity.mock_data()
}