use crate::data_model;
use dgraph::*;

/// Create dgraph-rs instance.
/// Connect to dgraph via gRPC.
pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

/// Drop all schema and data.
pub fn drop_schema(dgraph: &Dgraph) {
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");
}

/// Create schema for node, edge properties and types.
/// Create schema for all properties.
/// Set up schema.
pub fn set_schema(dgraph: &Dgraph) {
    let edge_props = data_model::create_edge_property();
    let node_props = data_model::create_node_property();
    let types = data_model::create_types();

    let property_schema = data_model::get_schema_from_properties(edge_props, node_props);
    let type_schema = data_model::get_schema_from_types(types);

    data_model::add_schema(dgraph, property_schema);
    data_model::add_schema(dgraph, type_schema);
}
