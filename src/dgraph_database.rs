use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

pub fn set_schema(dgraph: &Dgraph) {
    let op_schema = dgraph::Operation {
        schema: r#"
        "#
        .to_string(),
        ..Default::default()
    };

    dgraph.alter(&op_schema).expect("Failed to set schema.");
}
