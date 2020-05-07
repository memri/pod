use crate::data_model;
use dgraph::*;

/// Create dgraph gRPC connection.
pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

/// Drop all schema and data.
pub fn drop_schema_and_all_data_irreversibly(dgraph: &Dgraph) {
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");
}

/// Create node, edge properties and types.
/// Create schema for all properties and types.
/// Add schema to dgraph.
pub fn set_schema(dgraph: &Dgraph) {
    let edge_props = data_model::create_edge_property();
    let node_props = data_model::create_node_string_property();
    let types = data_model::create_types();

    let property_schema = data_model::get_schema_from_properties(edge_props, node_props);
    let type_schema = data_model::get_schema_from_types(types);

    data_model::add_schema(dgraph, property_schema);
    data_model::add_schema(dgraph, type_schema);
}
