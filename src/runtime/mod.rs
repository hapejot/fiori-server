//! Runtime components for serving OData requests.
//!
//! Includes HTTP handlers, URL parsing/routing, query execution, and pluggable
//! data-store implementations (in-memory by default, PostgreSQL when enabled).
pub mod data_store;
pub mod handlers;
#[cfg(feature = "postgres")]
pub mod pg_store;
pub mod query;
pub mod routing;
