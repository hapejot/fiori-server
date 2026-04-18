// Layer 2: Resolved data model
//
// Transforms Layer 1 specs (EntitySpec + Relationship) into fully resolved
// OData entities ready for XML generation.

pub mod defaults;
pub mod resolved;
pub mod resolver;

pub use resolved::*;
pub use resolver::resolve;
