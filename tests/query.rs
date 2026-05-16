use simple_fiori_server::odata::path_parser::parse_odata_resource_path;

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

#[test]
fn parse_uri() {
    let s = "Products";
    let x = parse_odata_resource_path(s);
    assert!(x.is_ok());

    let s = "Products(ID='P001')";
    let x = parse_odata_resource_path(s);
    match x {
        Ok(u) => {
            eprintln!("URI: {:?}", u);
        }
        Err(e) => panic!("Failed to parse URI: {}", e),
    }

    let s = "Orders(ProductID='11111111-1111-1111-1111-111111111111',IsActiveEntity=true)";
    let x = parse_odata_resource_path(s);
    match x {
        Ok(u) => {
            eprintln!("URI: {:?}", u);
        }
        Err(e) => panic!("Failed to parse URI: {}", e),
    }

    let s = "Products(ID='P001')/SiblingEntity";
    let x = parse_odata_resource_path(s);
    match x {
        Ok(u) => {
            eprintln!("URI: {:?}", u);
        }
        Err(e) => panic!("Failed to parse URI: {}", e),
    }

    let s = "Products('P001')/SiblingEntity";
    let x = parse_odata_resource_path(s);
    match x {
        Ok(u) => {
            panic!("Expected parsing to fail, but got: {:?}", u);
        }
        Err(e) => {}
    }
}
