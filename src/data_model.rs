use chrono::DateTime;
use chrono::Utc;
use dgraph::*;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use serde_json::map::Keys;
use std::collections::HashMap;
use std::collections::HashSet;

mod schema_definition;

#[derive(Deserialize, Serialize, Debug)]
pub struct Items {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub version: u64,
}

/// dgraph uid.
/// It works as a reference to a dgraph node and
/// is guaranteed to be unique for a node by dgraph.
pub type UID = u64;

// tag="type" adds a "type" field during JSON serialization
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct AuditAccessLog {
    pub audit_target: NodeReference,
    pub date_created: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeReference {
    pub uid: UID,
}

/// TODO: add docs what that means. I've no clue yet. But certainly needs a clean-up.
/// As was written in older code, these fields are meant as minimum required:
// * memriID
// * dgraph.type
// * version
pub static MINIMUM_FIELD_COUNT: usize = 3;

lazy_static! {
    /// A set of possible dgraph edges
    static ref VALID_EDGES: HashSet<String> = {
        dgraph_edge_properties().into_iter().map(|item|
            item.split(':').next().unwrap().to_string()
        ).collect()
    };
}

pub fn has_edge(fields: Keys) -> Vec<String> {
    let filtered = fields.filter(|field| VALID_EDGES.contains(*field));
    filtered.map(|f| f.to_string()).collect()
}

/// Create edge properties of all types.
/// Return a vector of edge properties in String -> `edge_property: type `.
pub fn dgraph_edge_properties() -> Vec<String> {
    let edge_props = schema_definition::get_edge_props();
    edge_props.iter().map(|x| (*x).to_string()).collect()
}

/// Create node properties of `string` type.
/// Return a vector of tuples -> (`node_property, type, [index]`).
pub fn dgraph_node_string_properties() -> Vec<(&'static str, &'static str, [&'static str; 1])> {
    let string_props: Vec<&str> = schema_definition::get_string_props();
    string_props
        .into_iter()
        .map(|x| (x, "string", ["term"]))
        .collect::<Vec<_>>()
}

/// Create other node properties which are not `string` typed,
/// e.g. int, float, datetime etc.
/// Return a vector of &str -> `node_property: type @index .`.
fn dgraph_node_nonstring_properties() -> Vec<&'static str> {
    let other_props: Vec<&str> = schema_definition::get_other_props();
    other_props
}

/// Get schema for edge and node properties.
/// Return a dgraph operation of schema.
pub fn get_schema_from_properties(
    e_prop: Vec<String>,
    n_prop: Vec<(&str, &str, [&str; 1])>,
) -> dgraph::Operation {
    // Format edge properties
    let mut eprops: Vec<String> = vec![];
    eprops.extend(e_prop.iter().map(|x| format_edge_prop(x)));
    // Format node properties
    let mut nprops: Vec<String> = vec![];
    nprops.extend(
        n_prop
            .iter()
            .map(|x| format_node_string_prop(x.0, x.1, x.2.to_vec())),
    );
    let o_prop = dgraph_node_nonstring_properties();
    nprops.extend(o_prop.iter().map(|x| format_node_other_prop(x)));
    // Combine both
    let combine_prop = combine(&mut eprops, &mut nprops);

    dgraph::Operation {
        schema: combine_prop,
        ..Default::default()
    }
}

/// Format edge properties to a string -> `edge_property: type @reverse .`.
fn format_edge_prop(p: &str) -> String {
    p.to_string() + "@reverse ."
}

/// Format node properties of `string` type.
/// Return a string -> `node_property: type @index .`.
fn format_node_string_prop(p: &str, _type: &str, indices: Vec<&str>) -> String {
    let joined = if *indices.first().unwrap() != "" {
        format!("@index({}) .", indices.join(","))
    } else {
        String::from(" .")
    };

    p.to_string() + ": " + _type + " " + &joined
}

/// Format other node properties which are not `string` typed.
/// Return a sting -> `other_property: type @index .`.
fn format_node_other_prop(p: &str) -> String {
    p.to_string()
}

/// Combine edge and node properties.
/// Return a string of combined properties.
fn combine(ep: &mut Vec<String>, np: &mut Vec<String>) -> String {
    ep.extend_from_slice(&np);
    ep.join("\n")
}

/// Add schema by altering dgraph.
pub fn add_schema(dgraph: &Dgraph, schema: dgraph::Operation) {
    dgraph.alter(&schema).expect("Failed to add schema.");
}

/// Generate type definitions based on type name and fields.
/// Return a vector of formatted type string.
pub fn generate_dgraph_type_definitions() -> Vec<String> {
    let all_types: HashMap<&str, Vec<&str>> = schema_definition::get_all_types();
    all_types
        .into_iter()
        .map(|(_type, fields)| format_type(_type, &fields))
        .collect()
}

/// Format type.
/// Return a string -> `type type_name { field1, field2, ... }`.
fn format_type(name: &str, fields: &[&str]) -> String {
    String::from("type ") + &name.to_string() + " {\n" + &fields.join("\n") + "\n}"
}

/// Get schema for types.
/// Return a dgraph operation of schema.
pub fn get_schema_from_types(types: Vec<String>) -> dgraph::Operation {
    dgraph::Operation {
        schema: types.join("\n"),
        ..Default::default()
    }
}
