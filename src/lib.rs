//! # simple-fiori-server
//!
//! An OData V4 backend for SAP Fiori Elements applications without any further SAP dependencies.
//!
//! The crate provides a layered metadata pipeline, runtime request handling,
//! draft-enabled CRUD behavior, and generated FLP/CDM artifacts to support
//! realistic local development and UI prototyping.
//!
//! ## Architecture
//!
//! The metadata system is organized as a 3-layer flow:
//! - [`spec`] defines high-level entity and relationship declarations.
//! - [`model`] resolves declarations into a complete intermediate model.
//! - [`odata`] generates OData XML and UI annotations from the resolved model.
//!
//! Runtime request processing and persistence abstractions are provided by
//! [`runtime`], while entity definitions and dynamic generic-entity creation
//! live in [`entities`].

/// Runtime request handling, query execution, routing, and data-store backends.
pub mod runtime;
/// Layer 1 specifications: entity declarations, relationships, and synthetic records.
pub mod spec;
/// Server and UI settings loading.
pub mod settings;
/// Layer 2 resolved model built from specifications.
pub mod model;
/// OData entity trait and wrapper types used by the registry/runtime.
pub mod entity;
/// Layer 3 OData metadata and annotation XML generation.
pub mod odata;
/// Shared application state and state-builder pipeline.
pub mod app_state;
/// Legacy annotation definition types and XML helpers.
pub mod annotations;
/// Builders for metadata XML, manifests, CDM site document, and FLP shell HTML.
pub mod builders;
/// Built-in entities and generic/meta entity construction.
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
