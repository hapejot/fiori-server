//! Layer 2: Resolved data model types.
//!
//! These are the fully-resolved, tweakable OData model objects produced by the
//! resolver from Layer 1 specs. They carry everything needed for XML generation.

use crate::spec::{MeasureKind, ValueListFilter};

/// A resolved OData property (column in the entity type).
#[derive(Debug, Clone)]
pub struct ResolvedProperty {
    /// Property name (e.g. "OrderDate", "CustomerID").
    pub name: String,
    /// OData type (e.g. "Edm.String", "Edm.Guid", "Edm.Decimal").
    pub edm_type: String,
    /// Display label for Common.Label annotation.
    pub label: String,
    /// Max string length (for Edm.String).
    pub max_length: Option<u32>,
    /// Decimal precision.
    pub precision: Option<u32>,
    /// Decimal scale.
    pub scale: Option<u32>,
    /// Core.Computed — server-generated, never editable.
    pub computed: bool,
    /// Core.Immutable — set at creation, read-only afterward.
    pub immutable: bool,
    /// UI.Hidden — hidden from all UI surfaces (auto-set for Edm.Guid fields).
    pub hidden: bool,
    /// Common.Text path — shows text instead of value (e.g. "Customer/CustomerName").
    pub text_path: Option<String>,
    /// Value help configuration.
    pub value_list: Option<ResolvedValueList>,
    /// Measure annotation (ISOCurrency or Unit).
    pub measure: Option<ResolvedMeasure>,
    /// Presentation metadata.
    pub presentation: ResolvedPresentation,
    /// Package this property came from.
    pub package: Option<String>,
}

/// Resolved value list — ready for annotation generation.
#[derive(Debug, Clone)]
pub enum ResolvedValueList {
    /// FieldValueListItems with Constant filter on ListID.
    CodeList {
        list_id: String,
        fixed_values: bool,
    },
    /// Reference to another EntitySet.
    EntityRef {
        collection_path: String,
        key_property: String,
        display_property: Option<String>,
        filters: Vec<ValueListFilter>,
        fixed_values: bool,
    },
}

/// Resolved measure annotation.
#[derive(Debug, Clone)]
pub struct ResolvedMeasure {
    /// Name of the companion field holding the unit/currency code.
    pub unit_field: String,
    /// Currency or Unit.
    pub kind: MeasureKind,
}

/// Resolved presentation metadata for a property.
#[derive(Debug, Clone, Default)]
pub struct ResolvedPresentation {
    /// Appears in SelectionFields (filter bar).
    pub searchable: bool,
    /// Appears as column in LineItem.
    pub show_in_list: bool,
    /// Column sort order in LineItem.
    pub list_sort_order: Option<u32>,
    /// Responsive table importance ("High", "Medium", "Low").
    pub list_importance: Option<String>,
    /// Criticality path for list column coloring.
    pub criticality_path: Option<String>,
    /// FieldGroup qualifier — controls form section placement.
    pub form_group: Option<String>,
}

/// A resolved navigation property.
#[derive(Debug, Clone)]
pub struct ResolvedNavProperty {
    /// Navigation property name (e.g. "Customer", "Items", "_HeaderTitleField").
    pub name: String,
    /// Target EntityType name (e.g. "Order", "EntityField").
    pub target_type: String,
    /// Target EntitySet name (e.g. "Orders", "EntityFields").
    pub target_set: String,
    /// true = Collection (1:N), false = single (1:1 / N:1).
    pub is_collection: bool,
    /// FK field name for referential constraint (e.g. "CustomerID" → "ID").
    pub foreign_key: Option<String>,
    /// Relationship name this nav came from.
    pub relationship: String,
    /// true = composition (parent controls child lifecycle).
    pub is_composition: bool,
}

/// A resolved data point (KPI in the Object Page header).
#[derive(Debug, Clone)]
pub struct ResolvedDataPoint {
    pub qualifier: String,
    pub value_path: String,
    pub title: String,
    pub max_value: Option<u32>,
    pub visualization: Option<String>,
}

/// A resolved header facet (references a data point).
#[derive(Debug, Clone)]
pub struct ResolvedHeaderFacet {
    pub data_point_qualifier: String,
    pub label: String,
}

/// A resolved facet section (FieldGroup on the Object Page).
#[derive(Debug, Clone)]
pub struct ResolvedFacetSection {
    pub label: String,
    pub id: String,
    pub field_group_qualifier: String,
    pub field_group_label: String,
}

/// A resolved table facet (child collection table on the Object Page).
#[derive(Debug, Clone)]
pub struct ResolvedTableFacet {
    pub label: String,
    pub id: String,
    pub navigation_property: String,
}

/// A fully resolved OData entity — everything needed for EDMX + annotation generation.
///
/// Produced by the resolver from `EntitySpec` + `Relationship`s.
/// Can be tweaked before passing to the XML generators.
#[derive(Debug, Clone)]
pub struct ResolvedEntity {
    /// EntitySet name (e.g. "Orders").
    pub set_name: String,
    /// EntityType name (e.g. "Order").
    pub type_name: String,
    /// Plural display name (e.g. "Orders").
    pub type_name_plural: String,
    /// Key field name (always "ID").
    pub key_field: String,
    /// Title field for UI.HeaderInfo + Common.Text on key.
    pub title_field: String,
    /// Description field for UI.HeaderInfo.
    pub description_field: Option<String>,
    /// Parent EntitySet for composition (child entity gets DraftNode).
    pub parent_set_name: Option<String>,
    /// All properties (own fields + auto-generated FK fields).
    pub properties: Vec<ResolvedProperty>,
    /// All navigation properties (from relationships).
    pub nav_properties: Vec<ResolvedNavProperty>,
    /// Data points (KPIs).
    pub data_points: Vec<ResolvedDataPoint>,
    /// Header facets.
    pub header_facets: Vec<ResolvedHeaderFacet>,
    /// Facet sections (auto-derived from unique form_group values).
    pub facet_sections: Vec<ResolvedFacetSection>,
    /// Table facets (child collection tables).
    pub table_facets: Vec<ResolvedTableFacet>,
    /// Selection fields (filter bar fields).
    pub selection_fields: Vec<String>,
    /// Package this entity belongs to.
    pub package: Option<String>,
}
