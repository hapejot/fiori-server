use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use tracing::info;
use serde_json::Value;

use crate::builders;
use crate::data_store::{DataStore, InMemoryDataStore};
use crate::entity::ODataEntity;
use crate::entities::generic::create_generic_entities;
use crate::entities::meta::reconstruct_configs_from_data;
use crate::settings::Settings;

/// Gesamtzustand der Applikation – haelt vorberechnete Artefakte
/// (Metadata-XML, manifest.json, FLP-HTML) und die Entity-Registry.
/// Felder, die sich zur Laufzeit aendern koennen (z.B. nach activate_config),
/// sind hinter RwLock geschuetzt.
pub struct AppState {
    pub entities: RwLock<Vec<&'static dyn ODataEntity>>,
    pub metadata_xml: RwLock<String>,
    pub manifest_json: RwLock<String>,
    /// Per-Entity manifest.json: EntitySet-Name -> JSON-String.
    pub entity_manifests: RwLock<HashMap<String, String>>,
    pub flp_html: String,
    /// Dynamisch generierte apps.json (statische + generische Entitaeten zusammengefuehrt).
    pub apps_json: RwLock<String>,
    /// Mutable Data-Store (abstracted behind DataStore trait)
    pub data_store: Box<dyn DataStore>,
    /// Settings (UI5-Version, Theme etc.)
    pub settings: Settings,
    /// Datenverzeichnis fuer Persistenz und Rekonstruktion
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    pub fn find_entity(&self, set_name: &str) -> Option<&'static dyn ODataEntity> {
        self.entities.read().unwrap().iter().find(|e| e.set_name() == set_name).copied()
    }

    /// Aktiviert eine Entity-Konfiguration zur Laufzeit:
    /// 1. commit() – aktuelle Daten persistieren
    /// 2. Meta-Tabellen aus data/ neu einlesen und GenericEntities neu erzeugen
    /// 3. Builtin-Entities beibehalten, generische ersetzen
    /// 4. metadata_xml, manifest_json, entity_manifests, apps_json neu aufbauen
    /// 5. DataStore-Entities aktualisieren
    pub fn activate_config(&self) {
        info!("  [activate_config] Rebuilding generic entities from meta tables...");

        // 1. Persist current data
        self.data_store.commit();

        // 2. Reconstruct configs from persisted meta tables
        let raw_configs = reconstruct_configs_from_data(&self.data_dir);
        let generic_entities = create_generic_entities(raw_configs);

        // 3. Build new entity list: keep built-in, replace generic.
        //    Built-in entities have known type names that are NOT from EntityConfigs.
        let old_entities = self.entities.read().unwrap().clone();
        let builtin: Vec<&'static dyn ODataEntity> = old_entities
            .iter()
            .filter(|e| !is_generic_entity(e))
            .copied()
            .collect();
        let mut new_entities = builtin;
        new_entities.extend(generic_entities);

        // 4. Rebuild all derived artifacts
        let metadata_xml = builders::build_metadata_xml(&new_entities);
        let manifest_json =
            serde_json::to_string_pretty(&builders::build_manifest_json(&new_entities, &self.settings))
                .unwrap_or_default();

        let mut entity_manifests = HashMap::new();
        for (idx, entity) in new_entities.iter().enumerate() {
            let manifest_val =
                builders::build_manifest_json_with_default(&new_entities, &self.settings, idx);
            entity_manifests.insert(
                entity.set_name().to_string(),
                serde_json::to_string_pretty(&manifest_val).unwrap_or_default(),
            );
        }

        let apps_json = build_apps_json(&new_entities);

        // 5. Update DataStore entity list
        self.data_store.update_entities(new_entities.clone());

        // 6. Swap in new values
        *self.entities.write().unwrap() = new_entities;
        *self.metadata_xml.write().unwrap() = metadata_xml;
        *self.manifest_json.write().unwrap() = manifest_json;
        *self.entity_manifests.write().unwrap() = entity_manifests;
        *self.apps_json.write().unwrap() = apps_json;

        info!("  [activate_config] Done – entities rebuilt.");
    }
}

/// Prueft, ob eine Entity eine generische (aus EntityConfig) ist.
/// Built-in Entities haben bekannte SetNames.
fn is_generic_entity(entity: &&'static dyn ODataEntity) -> bool {
    const BUILTIN_SETS: &[&str] = &[
        "EntityConfigs", "EntityFields", "EntityFacets",
        "EntityNavigations", "EntityTableFacets",
        "FieldValueLists", "FieldValueListItems",
    ];
    !BUILTIN_SETS.contains(&entity.set_name())
}

/// Baut die apps.json aus statischer Datei und Entity-Apps-Eintraegen zusammen.
fn build_apps_json(entities: &[&'static dyn ODataEntity]) -> String {
    let webapp_dir = std::env::current_dir().unwrap_or_default().join("webapp");
    let static_path = webapp_dir.join("config/apps.json");
    let base_json = if static_path.is_file() {
        std::fs::read_to_string(&static_path).ok()
    } else {
        Some(crate::EMBEDDED_APPS_JSON.to_string())
    };
    let mut apps: serde_json::Map<String, Value> = base_json
        .and_then(|c| serde_json::from_str::<Value>(&c).ok())
        .and_then(|v| v.get("applications").cloned())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    for entity in entities {
        if let Some((key, val)) = entity.apps_json_entry() {
            apps.insert(key, val);
        }
    }
    let wrapper = serde_json::json!({ "applications": apps });
    serde_json::to_string_pretty(&wrapper).unwrap_or_default()
}

/// Builder fuer schrittweise Konfiguration des AppState.
pub struct AppStateBuilder {
    pub(crate) entities: Vec<&'static dyn ODataEntity>,
    settings: Option<Settings>,
    data_dir: Option<PathBuf>,
    data_store: Option<Box<dyn DataStore>>,
}

impl AppStateBuilder {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            settings: None,
            data_dir: None,
            data_store: None,
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

    pub fn data_store(mut self, store: Box<dyn DataStore>) -> Self {
        self.data_store = Some(store);
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

        let apps_json = build_apps_json(&entities);

        // Data-Verzeichnis
        let data_dir = self
            .data_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("data"));

        // Build the data store: use provided store or default to in-memory
        let data_store = self.data_store.unwrap_or_else(|| {
            Box::new(InMemoryDataStore::new(data_dir.clone(), entities.clone()))
        });

        AppState {
            entities: RwLock::new(entities),
            metadata_xml: RwLock::new(metadata_xml),
            manifest_json: RwLock::new(manifest_json),
            entity_manifests: RwLock::new(entity_manifests),
            flp_html,
            apps_json: RwLock::new(apps_json),
            data_store,
            settings,
            data_dir,
        }
    }
}