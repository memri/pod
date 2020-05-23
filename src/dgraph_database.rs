use crate::data_model;
use dgraph::*;
use log::info;

/// Create dgraph gRPC connection.
pub fn create_dgraph(dgraph_host: &str) -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client(dgraph_host);
    Dgraph::new(vec![dgraph_client])
}

/// Drop all schema and data.
pub fn drop_schema_and_all_data_irreversibly(dgraph: &Dgraph) {
    info!("Dropping Dgraph schema and all associated data.");
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");
}

/// Create node, edge properties and types.
/// Create schema for all properties and types.
/// Add schema to dgraph.
pub fn add_schema(dgraph: &Dgraph) {
    info!("Adding full Dgraph schema.");
    let edge_props = data_model::dgraph_edge_properties();
    let node_props = data_model::dgraph_node_string_properties();
    let types = data_model::generate_dgraph_type_definitions();

    let property_schema = data_model::get_schema_from_properties(edge_props, node_props);
    let type_schema = data_model::get_schema_from_types(types);

    data_model::add_schema(dgraph, property_schema);
    data_model::add_schema(dgraph, type_schema);
}
