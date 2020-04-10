use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client("localhost:9080"));
    let op = dgraph::Operation {
        schema: "name: string @index(exact) .".to_string(),
        ..Default::default()
    };
    let result = dgraph.alter(&op);
    result.unwrap(); // TODO unwrap
    dgraph
}
