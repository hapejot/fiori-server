//! Built-in entity implementations and generic/meta reconstruction helpers.
//!
//! This module exposes concrete meta-table entities used by the admin/config
//! layer, plus helpers for reconstructing and generating runtime generic
//! entities from persisted configuration records.

pub mod generic;
pub mod meta;
mod entity_config;
mod entity_field;
mod entity_facet;
mod entity_navigation;
mod entity_relationship;
mod entity_table_facet;
mod field_value_list;
mod field_value_list_item;

pub use entity_config::EntityConfigEntity;
pub use entity_field::EntityFieldEntity;
pub use entity_facet::EntityFacetEntity;
pub use entity_navigation::EntityNavigationEntity;
pub use entity_relationship::EntityRelationshipEntity;
pub use entity_table_facet::EntityTableFacetEntity;
pub use field_value_list::FieldValueListEntity;
pub use field_value_list_item::FieldValueListItemEntity;
