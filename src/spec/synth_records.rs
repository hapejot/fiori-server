//! Generate synthetic JSON records from EntitySpec + Relationship definitions.
//!
//! These records are injected into the data store at startup so that
//! builtin (code-defined) entities appear in the Fiori admin UI
//! alongside user-configured generic entities.

use serde_json::{json, Value};

use super::{EntitySpec, Relationship};
use crate::entity::value_list_id;

/// Generate all synthetic records from specs and relationships.
/// Returns (EntityConfigs, EntityFields, EntityFacets, EntityNavigations, EntityTableFacets).
pub fn generate_synth_records(
    specs: &[EntitySpec],
    relationships: &[Relationship],
) -> Vec<(&'static str, Vec<Value>)> {
    let mut config_records = Vec::new();
    let mut field_records = Vec::new();
    let mut facet_records = Vec::new();
    let mut nav_records = Vec::new();
    let mut table_facet_records = Vec::new();

    for spec in specs {
        let config_uuid = value_list_id(&format!("config_{}", spec.set_name));

        // ── EntityFields ────────────────────────────────────────────
        let mut fields_for_config = Vec::new();
        for (idx, field) in spec.fields.iter().enumerate() {
            let (name, label, edm_type, max_length, precision, scale, computed, immutable) =
                match field {
                    super::FieldSpec::Atom {
                        name,
                        label,
                        edm_type,
                        max_length,
                        precision,
                        scale,
                        computed,
                        immutable,
                        ..
                    } => (
                        name.as_str(),
                        label.as_str(),
                        edm_type.as_str(),
                        *max_length,
                        *precision,
                        *scale,
                        *computed,
                        *immutable,
                    ),
                    super::FieldSpec::Measure {
                        name,
                        label,
                        precision,
                        scale,
                        ..
                    } => (
                        name.as_str(),
                        label.as_str(),
                        "Edm.Decimal",
                        None,
                        *precision,
                        *scale,
                        false,
                        false,
                    ),
                };

            let p = field.presentation();
            let field_uuid =
                value_list_id(&format!("field_{}_{}", spec.set_name, name));

            fields_for_config.push((name.to_string(), field_uuid.clone()));

            field_records.push(json!({
                "ID": field_uuid,
                "ConfigID": config_uuid,
                "SetName": spec.set_name,
                "FieldName": name,
                "Label": label,
                "EdmType": edm_type,
                "MaxLength": max_length,
                "Precision": precision,
                "Scale": scale,
                "IsComputed": computed,
                "IsImmutable": immutable,
                "SortOrder": idx as u32,
                "ShowInLineItem": p.show_in_list.unwrap_or(!computed && edm_type != "Edm.Guid"),
                "LineItemImportance": p.list_importance.as_deref().unwrap_or(""),
                "LineItemCriticalityPath": p.criticality_path.as_deref().unwrap_or(""),
                "LineItemLabel": "",
                "LineItemSemanticObject": "",
                "Searchable": p.searchable.unwrap_or(false),
                "FormGroup": p.form_group.as_deref().unwrap_or(""),
                "ReferencesEntity": "",
                "TextPath": "",
                "ValueSource": "",
                "PreferDialog": false,
            }));
        }

        // Resolve header title/description path to field UUIDs
        let title_uuid = spec
            .title_field
            .as_ref()
            .and_then(|tf| {
                fields_for_config
                    .iter()
                    .find(|(n, _)| n == tf)
                    .map(|(_, id)| id.clone())
            })
            .unwrap_or_default();

        let desc_uuid = spec
            .description_field
            .as_ref()
            .and_then(|df| {
                fields_for_config
                    .iter()
                    .find(|(n, _)| n == df)
                    .map(|(_, id)| id.clone())
            })
            .unwrap_or_default();

        // Find parent set from relationships (if child of a composition)
        let parent_set = relationships
            .iter()
            .find(|r| r.owned && r.many.entity == spec.set_name)
            .map(|r| r.one.entity.as_str())
            .unwrap_or("");

        // Resolve selection fields from field specs
        let selection_fields: Vec<&str> = spec
            .fields
            .iter()
            .filter(|f| {
                f.presentation()
                    .searchable
                    .unwrap_or(false)
            })
            .map(|f| f.name())
            .collect();

        // ── EntityConfigs ───────────────────────────────────────────
        let data_points_json = if spec.data_points.is_empty() {
            String::new()
        } else {
            let dp_vals: Vec<Value> = spec
                .data_points
                .iter()
                .map(|dp| {
                    json!({
                        "qualifier": dp.qualifier,
                        "value_path": dp.value_path,
                        "title": dp.title,
                    })
                })
                .collect();
            serde_json::to_string(&dp_vals).unwrap_or_default()
        };
        let header_facets_json = if spec.header_facets.is_empty() {
            String::new()
        } else {
            let hf_vals: Vec<Value> = spec
                .header_facets
                .iter()
                .map(|hf| {
                    json!({
                        "data_point_qualifier": hf.data_point_qualifier,
                        "label": hf.label,
                    })
                })
                .collect();
            serde_json::to_string(&hf_vals).unwrap_or_default()
        };

        config_records.push(json!({
            "ID": config_uuid,
            "SetName": spec.set_name,
            "TypeName": spec.resolved_type_name(),
            "Title": spec.set_name,
            "HeaderTypeName": spec.resolved_type_name(),
            "HeaderTypeNamePlural": spec.resolved_type_name_plural(),
            "HeaderTitlePath": title_uuid,
            "HeaderDescriptionPath": desc_uuid,
            "ParentSetName": parent_set,
            "SelectionFields": selection_fields.join(","),
            "TileDescription": "",
            "TileIcon": "",
            "DefaultValues": "",
            "HeaderFacets": header_facets_json,
            "DataPoints": data_points_json,
        }));

        // ── EntityFacets ────────────────────────────────────────────
        for (idx, section) in spec.facet_sections.iter().enumerate() {
            facet_records.push(json!({
                "ID": value_list_id(&format!("facet_{}_{}", spec.set_name, section.id)),
                "ConfigID": config_uuid,
                "SetName": spec.set_name,
                "SectionLabel": section.label,
                "SectionId": section.id,
                "FieldGroupQualifier": section.field_group_qualifier,
                "FieldGroupLabel": section.field_group_label,
                "FieldGroupFields": "",
                "SortOrder": idx as u32,
            }));
        }

        // ── EntityNavigations (from composition relationships) ──────
        let mut nav_idx = 0u32;
        for rel in relationships {
            if rel.one.entity == spec.set_name && rel.owned && !rel.one_side_hidden() {
                nav_records.push(json!({
                    "ID": value_list_id(&format!("nav_{}_{}", spec.set_name, rel.one.nav_name)),
                    "ConfigID": config_uuid,
                    "SetName": spec.set_name,
                    "NavName": rel.one.nav_name,
                    "TargetSet": rel.many.entity,
                    "TargetType": find_type_name(specs, &rel.many.entity),
                    "ForeignKey": rel.fk_field_name(),
                    "IsCollection": true,
                    "SortOrder": nav_idx,
                }));
                nav_idx += 1;
            }
        }

        // ── EntityTableFacets (from composition relationships) ──────
        let mut tf_idx = 0u32;
        // Explicit table facets from spec
        for tf in &spec.table_facets {
            table_facet_records.push(json!({
                "ID": value_list_id(&format!("tablefacet_{}_{}", spec.set_name, tf.id)),
                "ConfigID": config_uuid,
                "SetName": spec.set_name,
                "FacetLabel": tf.label,
                "FacetId": tf.id,
                "NavigationProperty": tf.navigation_property,
                "SortOrder": tf_idx,
            }));
            tf_idx += 1;
        }
        // Auto-derived from composition relationships (if not already explicit)
        for rel in relationships {
            if rel.one.entity == spec.set_name && rel.owned && !rel.one_side_hidden() {
                let already_exists = spec
                    .table_facets
                    .iter()
                    .any(|tf| tf.navigation_property == rel.one.nav_name);
                if !already_exists {
                    let facet_id = format!("{}Section", rel.one.nav_name);
                    table_facet_records.push(json!({
                        "ID": value_list_id(&format!("tablefacet_{}_{}", spec.set_name, facet_id)),
                        "ConfigID": config_uuid,
                        "SetName": spec.set_name,
                        "FacetLabel": rel.one.nav_name,
                        "FacetId": facet_id,
                        "NavigationProperty": rel.one.nav_name,
                        "SortOrder": tf_idx,
                    }));
                    tf_idx += 1;
                }
            }
        }
    }

    vec![
        ("EntityConfigs", config_records),
        ("EntityFields", field_records),
        ("EntityFacets", facet_records),
        ("EntityNavigations", nav_records),
        ("EntityTableFacets", table_facet_records),
    ]
}

/// Find type name for an entity set from the specs list.
fn find_type_name(specs: &[EntitySpec], set_name: &str) -> String {
    specs
        .iter()
        .find(|s| s.set_name == set_name)
        .map(|s| s.resolved_type_name())
        .unwrap_or_else(|| {
            // Default: strip trailing 's'
            if set_name.ends_with('s') && set_name.len() > 1 {
                set_name[..set_name.len() - 1].to_string()
            } else {
                set_name.to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::meta_package::meta_package;

    #[test]
    fn test_synth_records_from_meta_package() {
        let (specs, rels) = meta_package();
        let records = generate_synth_records(&specs, &rels);

        // Should have 5 categories
        assert_eq!(records.len(), 5);

        let configs = &records[0].1;
        assert_eq!(configs.len(), 7, "7 meta entity specs");

        // EntityConfigs should be a root entity
        let ec = configs
            .iter()
            .find(|c| c["SetName"] == "EntityConfigs")
            .unwrap();
        assert_eq!(ec["ParentSetName"], "");
        assert_eq!(ec["TypeName"], "EntityConfig");

        // EntityFields should be a child of EntityConfigs
        let ef = configs
            .iter()
            .find(|c| c["SetName"] == "EntityFields")
            .unwrap();
        assert_eq!(ef["ParentSetName"], "EntityConfigs");

        // FieldValueListItems should be a child of FieldValueLists
        let fvli = configs
            .iter()
            .find(|c| c["SetName"] == "FieldValueListItems")
            .unwrap();
        assert_eq!(fvli["ParentSetName"], "FieldValueLists");

        // EntityFields should have field records
        let fields = &records[1].1;
        let ec_fields: Vec<_> = fields
            .iter()
            .filter(|f| f["SetName"] == "EntityConfigs")
            .collect();
        // EntityConfigs has 12 fields in its spec
        assert!(ec_fields.len() >= 10, "EntityConfigs should have many fields");

        // EntityConfigs should have navigation records (Fields, Facets, Navigations, TableFacets)
        let navs = &records[3].1;
        let ec_navs: Vec<_> = navs
            .iter()
            .filter(|n| n["SetName"] == "EntityConfigs")
            .collect();
        assert_eq!(ec_navs.len(), 4, "EntityConfigs has 4 composition children");

        // EntityConfigs should have table facet records
        let tfs = &records[4].1;
        let ec_tfs: Vec<_> = tfs
            .iter()
            .filter(|t| t["SetName"] == "EntityConfigs")
            .collect();
        assert_eq!(ec_tfs.len(), 4, "EntityConfigs has 4 table facets");
    }
}
