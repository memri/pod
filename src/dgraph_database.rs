use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    let dgraph = Dgraph::new(vec![dgraph_client]);
    dgraph
}
