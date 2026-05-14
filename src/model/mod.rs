//! Layer 2 of the metadata pipeline: resolved data model.
//!
//! This layer transforms Layer 1 declarations (`EntitySpec` + `Relationship`)
//! into fully resolved entities that contain concrete OData-facing properties,
//! navigation definitions, value lists, and facet structure.

pub mod defaults;
pub mod resolved;
pub mod resolver;

pub use resolved::*;
pub use resolver::resolve;
