//! Layer 3 of the metadata pipeline: OData XML and annotation generation.
//!
//! This module consumes the resolved model and emits metadata artifacts:
//! - [`crate::odata::entity_type`] generates `EntityType` / `EntitySet` / draft action XML.
//! - [`crate::odata::annotations_gen`] generates UI and capability annotations.
//! - [`crate::odata::xml_types`] contains the XML serialization DSL used by generators.
//! - [`crate::odata::vocab`] provides typed SAP UI annotation vocabulary helpers.
//! - [`crate::odata::builders`] re-exports legacy high-level builders (metadata/manifest/CDM/FLP).

pub mod annotations_gen;
pub mod entity_type;
pub mod vocab;
pub mod xml_types;
pub mod builders;

pub use annotations_gen::*;
pub use entity_type::*;
pub use vocab::*;
pub use xml_types::anns_to_xml;
pub use builders::*;
