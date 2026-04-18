// Layer 3: OData XML generation
//
// xml_types — PV/Rec/Ann/Anns serialization DSL (new home)
// entity_type — EntityType/EntitySet/DraftActions XML from ResolvedEntity
// annotations_gen — UI + Capability annotations from ResolvedEntity
// builders — re-exports from legacy `builders` module (metadata, manifest, CDM, FLP)
//
// Annotation builders (build_annotations, build_capabilities, etc.) remain in
// the legacy `annotations` module for now and are re-exported from crate root.
pub mod annotations_gen;
pub mod entity_type;
pub mod xml_types;
pub mod builders;

pub use annotations_gen::*;
pub use entity_type::*;
pub use xml_types::*;
pub use builders::*;
