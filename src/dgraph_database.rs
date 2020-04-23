use crate::data_model;
use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

pub fn drop_schema(dgraph: &Dgraph) {
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");
}

pub fn set_schema(dgraph: &Dgraph) {
    let edge_props = data_model::create_edge_property();

    let node_props = data_model::create_node_property();

    let op_schema = data_model::add_schema_from_properties(edge_props, node_props);

    data_model::add_schema(dgraph, op_schema);
}



