pub mod runtime;
pub mod spec;
pub mod settings;
pub mod model;
pub mod entity;
pub mod odata;
pub mod app_state;
pub mod annotations;
pub mod builders;
pub mod entities;

#[cfg(feature = "postgres")]
pub mod pg_store {
    pub use crate::runtime::pg_store::*;
}
pub const BASE_PATH: &str = "/odata/v4/Service";
pub const NAMESPACE: &str = "Service";
// ── Embedded static webapp files ────────────────────────────────────────
pub const EMBEDDED_FLP_INIT_JS: &str = include_str!("../webapp/flp-init.js");
pub const EMBEDDED_SETTINGS_JSON: &str = include_str!("../webapp/config/settings.json");
pub const EMBEDDED_APPS_JSON: &str = include_str!("../webapp/config/apps.json");
pub const EMBEDDED_I18N_PROPERTIES: &str = include_str!("../webapp/i18n/i18n.properties");
pub const EMBEDDED_SANDBOX_CONFIG: &str =
    include_str!("../webapp/appconfig/fioriSandboxConfig.json");
