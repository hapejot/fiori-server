// Legacy annotation builder tests removed — covered by new pipeline tests in odata/

use fake_fiori_server::{
    model::{ResolvedEntity, ResolvedFacetSection, ResolvedPresentation, ResolvedProperty},
    odata::{self, xml_types::AnnContent},
};

#[test]
fn test_ui_ann_gen() {
    let e = ResolvedEntity {
        set_name: "EName".into(),
        type_name: "TName".into(),
        type_name_plural: "TNames".into(),
        key_field: "ID".into(),
        title_field: "Name".into(),
        description_field: None,
        parent_set_name: None,
        properties: vec![
            ResolvedProperty::new("ID".into(), "Edm.Guid".into()).with_field_group("fg1".into()),
            ResolvedProperty::new("Name".into(), "Edm.String".into()),
        ],
        nav_properties: vec![],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![ResolvedFacetSection {
            label: "General".into(),
            id: "general".into(),
            field_group_qualifier: "fg1".into(),
        }],
        table_facets: vec![],
        selection_fields: vec![],
        package: None,
        extra_annotations_xml: "".into(),
        custom_actions_xml: "".into(),
    };

    let ans = odata::annotations_gen::generate_ui_annotations(&e);

    assert_eq!(ans.len(), 1);
    let ui_ans = &ans[0];
    assert_eq!(ui_ans.target, "Service.TName");
    assert_eq!(6, ui_ans.annotations.len());
    if let Some(x) = ui_ans.annotations.iter().find(|x| x.term == "UI.Facets") {
        println!("{:?}: {:?}", x.term, x.content);
        match &x.content {
            AnnContent::Collection(recs) => assert_eq!(recs.len(), 1, "Expected one Facet record"),
            _ => panic!("Expected collection content for UI.Facets"),
        }
    }
    let ans2 = odata::annotations_gen::generate_annotations(&e);
    assert_eq!(ans.len(), 1);
    for n in [
        "Service.TName",
        "Service.EntityContainer/EName",
        "Service.TName/ID",
        "Service.TName/Name",
        "Service.TName/IsActiveEntity",
        "Service.TName/HasActiveEntity",
        "Service.TName/HasDraftEntity",
    ] {
        assert!(
            ans2.iter().any(|a| a.target == n),
            "Missing annotation for {}",
            n
        );
    }
}
