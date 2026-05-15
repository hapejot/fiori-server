#[test]
fn parse_filter() {
    let s = "(IsActiveEntity eq false)";
    let x = odata_params::filters::parse_str(s);
    assert!(x.is_ok(), "Failed to parse filter: {}", x.err().unwrap());

    let s = "(SiblingEntity/IsActiveEntity eq null)";
    let x = odata_params::filters::parse_str(s);
    assert!(x.is_ok(), "Failed to parse filter: {}", x.err().unwrap());

    let s = "(IsActiveEntity eq false or SiblingEntity/IsActiveEntity eq null)";
    let x = odata_params::filters::parse_str(s);
    assert!(x.is_ok(), "Failed to parse filter: {}", x.err().unwrap());
}
