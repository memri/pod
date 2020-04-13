use dgraph::*;

#[derive(Serialize, Deserialize, Debug)]
struct ItemList {
    items: Vec<DataItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataItem {
    uid: String,
    deleted: bool,
    starred: bool,
}

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    let dgraph = Dgraph::new(vec!(dgraph_client));
    dgraph
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
