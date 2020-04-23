use serde::{Deserialize, Serialize};
use dgraph::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Items {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub version: u64,
}

pub fn create_edge_property() -> Vec<String> {
    let edge_prop = ["subtype", "instance", "attends", "parent", "contains", "location", "from", "owns", "diet",
        "friend", "wife", "husband", "grandparent", "grandmother", "sister", "brother", "father", "mother",
        "boyfriend", "girlfriend", "daughter", "son"];
    let mut edge_props = vec![];
    edge_props.extend(edge_prop.iter().map(|x| x.to_string()));
    edge_props
}

pub fn create_node_property() -> Vec<(&'static str,&'static str,[&'static str;1])> {
    let s_props = vec!["is_type", "name", "confidence", "profile_picture"];
    let mut string_props = vec![];
    for x in 0..s_props.len() {
        string_props.insert(x, (s_props[x], "string", ["exact"]));
    }
    string_props.push(("creationdate", "DateTime", [""]));
    string_props.push(("aliases", "string", [""]));
    string_props
}

pub fn add_schema_from_properties(e_prop: Vec<String>, n_prop: Vec<(&str,&str,[&str;1])>) -> dgraph::Operation {
    let mut eprops: Vec<String> = vec![];
    eprops.extend(e_prop.iter().map(|x| format_e_prop(x)));

    let mut nprops: Vec<String> = vec![];
    nprops.extend(n_prop.iter()
                      .map(|x| format_n_prop(x.0, x.1, x.2.to_vec())));

    let combine_prop = combine(&mut eprops, &mut nprops);

    let op_schema = dgraph::Operation {
        schema: combine_prop,
        ..Default::default()
    };

    op_schema
}

fn format_e_prop(p: &str) -> String {
    p.to_string() + ": [uid] @reverse ."
}

fn format_n_prop(p: &str, _type: &str, indices: Vec<&str>) -> String {
    let mut joined = String::from(" .");
    if *indices.first().unwrap() != "" {
        joined = String::from("@index(") + &indices.join(",") + ") .";
    }

    p.to_string() + ": " + _type + " " + &joined
}

fn combine(ep: &mut Vec<String>, np: &mut Vec<String>) -> String {
    ep.extend_from_slice(&np);
    ep.join("\n")
}

pub fn add_schema(dgraph: &Dgraph, schema: dgraph::Operation) {
    dgraph.alter(&schema).expect("Failed to set schema.");
}
