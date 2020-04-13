use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    let dgraph = make_dgraph!(dgraph_client);
    dgraph
}
