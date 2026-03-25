use crate::builders;
use crate::entity::ODataEntity;
use crate::settings::Settings;

/// Gesamtzustand der Applikation – haelt vorberechnete Artefakte
/// (Metadata-XML, manifest.json, FLP-HTML) und die Entity-Registry.
pub struct AppState {
    pub entities: Vec<&'static dyn ODataEntity>,
    pub metadata_xml: String,
    pub manifest_json: String,
    pub flp_html: String,
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
        AppState {
            entities,
            metadata_xml,
            manifest_json,
            flp_html,
        }
    }
}
