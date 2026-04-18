//! Smart defaults — fills in presentation details that weren't explicitly set.

use super::resolved::*;

/// Apply smart defaults to a resolved entity:
/// - selection_fields from searchable properties
/// - facet_sections from unique form_group values
pub fn apply_defaults(entity: &mut ResolvedEntity) {
    derive_selection_fields(entity);
    derive_facet_sections(entity);
}

/// Collect all searchable properties into selection_fields.
fn derive_selection_fields(entity: &mut ResolvedEntity) {
    if !entity.selection_fields.is_empty() {
        return; // already set explicitly
    }
    entity.selection_fields = entity
        .properties
        .iter()
        .filter(|p| p.presentation.searchable)
        .map(|p| p.name.clone())
        .collect();
}

/// Auto-derive facet sections from unique form_group values found in properties.
/// Each unique form_group becomes a FacetSection with the group name as both
/// the qualifier and label.
fn derive_facet_sections(entity: &mut ResolvedEntity) {
    if !entity.facet_sections.is_empty() {
        return; // already set explicitly
    }
    let mut seen: Vec<String> = Vec::new();
    for prop in &entity.properties {
        if let Some(group) = &prop.presentation.form_group {
            if !seen.contains(group) {
                seen.push(group.clone());
            }
        }
    }
    entity.facet_sections = seen
        .into_iter()
        .map(|group| ResolvedFacetSection {
            label: group.clone(),
            id: format!("{group}Section"),
            field_group_qualifier: group.clone(),
            field_group_label: group,
        })
        .collect();
}
