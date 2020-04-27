use serde::{Deserialize, Serialize};
use dgraph::*;
use serde_json::{Value};
use std::collections::HashMap;

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

fn set_types() -> Vec<Vec<String>> {
    let types = r#"
    {
        "photo": null,
        "person": null,
        "location": {
            "city": null,
            "country": null,
            "restaurant": null,
            "island": null
        },
        "event": {
            "trip": {
                "sailing trip": null
            },
            "conference": {
                "google conference": null,
                "atlassian conference": null
            },
            "bike ride": null,
            "birthday": null,
            "night out": null,
            "visit": null,
            "party": {
                "birthday party": null,
                "dinner party": null
            },
            "anniversary": null,
            "dinner": {
                "birthday dinner": null,
                "anniversary dinner": null,
                "engagement dinner": null
            },
            "family weekend": null,
            "wedding": null,
            "festival": {
                "burning man": null
            },
            "meeting": {
                "current meeting": null,
                "show and tell": null,
                "weekly ai meeting": null
            },
            "dawson dance": null,
            "icebong": null,
            "new years eve": null,
            "public holiday": {
                "kingsday": null
            }
        }
    }"#;
    let value: Value = serde_json::from_str(&types).expect("error");

    let mut output = vec![vec![]];
    let current_path = vec![];
    deep_keys(&value, current_path, &mut output);
    output
}

fn deep_keys(value: &Value, current_path: Vec<String>, output: &mut Vec<Vec<String>>) {
   if current_path.len() > 0 {
        output.push(current_path.clone());
    }

    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let mut new_path = current_path.clone();
                new_path.push(k.to_owned());
                deep_keys(v,  new_path, output);

            }
        },
        _ => ()
    }
}

fn set_custom_aliases() {
    let custom_aliases = r#"
    {
        "trip": [{"trip": true}, {"journey": true}],
        "night out": [{"night out": false}, {"nights out": false}],
        "bike ride": [{"bike ride": true}, {"bicycle tour": true}],
        "google conference": [{"google io": false}, {"google conference": true},
                            {"google seminar": true}, {"google symposion": true}],
        "atlassian conference": [{"atlassian conference": true}, {"atlassian summit": true},
                               {"atlassian symposion": true}, {"atlassian seminar": true}],
        "dawson dance": [{"dawson dance": true}, {"dawson": true}],
        "sailing trip": [{"sailing": true}, {"sailing trip": true}],
        "burning man": [{"burning man": true}, {"burning mans": true}, {"the burn": true}],
        "family weekend": [{"family weekdend": true}, {"family trip": true}]
    }"#;
    let aliases: Value = serde_json::from_str(&custom_aliases).expect("error");


    get_aliases()

}

fn get_aliases() {

}

pub fn link_types() {
    let types = set_types();
    set_custom_aliases();
}
