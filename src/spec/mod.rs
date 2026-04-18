// Layer 1: Application specification types
//
// New types for the relationships-first architecture:
pub mod entity_spec;
pub mod meta_package;
pub mod relationship;
pub mod synth_records;

pub use entity_spec::*;
pub use relationship::*;

// Legacy re-exports — existing code uses these from crate::annotations::*
pub use crate::annotations::{
    AnnotationsDef, DataPointDef, FacetSectionDef, FieldDef, HeaderFacetDef, HeaderInfoDef,
    NavigationPropertyDef, TableFacetDef, ValueListDef,
};
