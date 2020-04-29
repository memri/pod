use dgraph::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct Items {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub version: u64,
}

pub fn create_edge_property() -> Vec<String> {
    let edge_prop = [
        "writtenBy: [uid] .",
        "sharedWith: [uid] .",
        "comments: [uid] .",
        "appliesTo: [uid] .",
        "file: uid .",
        "includes: [uid] .",
        "usedBy: [uid] .",
        "profilePicture: uid .",
        "relations: [uid] .",
        "phoneNumbers: [uid] .",
        "websites: [uid] .",
        "companies: [uid] .",
        "addresses: [uid] .",
        "publicKeys: [uid] .",
        "onlineProfiles: [uid] .",
        "diets: [uid] .",
        "medicalConditions: [uid] .",
        "sessions: [uid] .",
        "currentSession: uid .",
        "currentView: uid .",
        "country: uid .",
        "location: uid .",
        "flag: uid .",
        "views: [uid] .",
        "screenshot: uid .",
        "selection: [uid] .",
        "session: uid ."
    ];
    edge_prop.iter().map(|x| x.to_string()).collect()
}

pub fn create_node_property() -> Vec<(&'static str, &'static str, [&'static str; 1])> {
    let s_props = vec![
        "title",
        "content",
        "genericType",
        "computeTitle",
        "name",
        "comment",
        "color",
        "uri",
        "firstName",
        "lastName",
        "gender",
        "sexualOrientation",
        "contents",
        "action",
        "type",
        "number",
        "url",
        "city",
        "street",
        "state",
        "postalCode",
        "key",
        "handle",
        "rendererName",
        "subtitle",
        "backTitle",
        "icon",
        "browsingMode",
        "filterText",
        "emptyResultText",
        "_variables"
    ];
    let mut string_props = s_props
        .into_iter()
        .map(|x| (x, "string", ["term"]))
        .collect::<Vec<_>>();

    let other_props = vec![
        "width: int @index(int) .",
        "height: int @index(int) .",
        "duration: int @index(int) .",
        "bitrate: int @index(int) .",
        "birthDate: datetime .",
        "person_height: float @index(float) .",
        "shoulderWidth: float @index(float) .",
        "armLength: float @index(float) .",
        "age: float @index(float) .",
        "date: datetime .",
        "currentSessionIndex: int @index(int) .",
        "latitude: float @index(float) .",
        "longitude: float @index(float) .",
        "additions: [string] @index(term) .",
        "currentViewIndex: int @index(int) .",
        "showFilterPanel: bool .",
        "showContextPane: bool .",
        "editMode: bool .",
        "showLabels: bool .",
        "cascadeOrder: [string] .",
        "sortFields: [string] .",
        "activeStates: [string] ."
    ];

    string_props
}

pub fn add_schema_from_properties(
    e_prop: Vec<String>,
    n_prop: Vec<(&str, &str, [&str; 1])>,
) -> dgraph::Operation {
    let mut eprops: Vec<String> = vec![];
    eprops.extend(e_prop.iter().map(|x| format_e_prop(x)));

    let mut nprops: Vec<String> = vec![];
    nprops.extend(n_prop.iter().map(|x| format_n_prop(x.0, x.1, x.2.to_vec())));

    let combine_prop = combine(&mut eprops, &mut nprops);

    dgraph::Operation {
        schema: combine_prop,
        ..Default::default()
    }
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
        "note": {
            "title",
            "content",
            "genericType",
            "writtenBy",
            "writtenBy",
            "comments"
        },
         "label": {
            "name",
            "comment",
            "color",
            "genericType",
            "computeTitle",
            "appliesTo"
        },
        "photo": {
            "name",
            "file",
            "width",
            "height",
            "genericType",
            "computeTitle",
            "includes",
        },
        "video": {
            "name",
            "file",
            "width",
            "height",
            "duration",
            "genericType",
            "computeTitle",
            "includes"
        },
        "audio": {
            "name",
            "file",
            "bitrate",
            "duration",
            "genericType",
            "computeTitle",
            "includes"
        },
        "file": {
            "uri",
            "genericType",
            "usedBy"
        },
        "person": {
            "firstName",
            "lastName",
            "birthDate",
            "gender",
            "sexualOrientation",
            "height",
            "shoulderWidth",
            "armLength",
            "age",
            "genericType",
            "profilePicture",
            "relations",
            "phoneNumber",
            "websites",
            "companies",
            "addresses",
            "publicKeys",
            "onlineProfiles",
            "diets",
            "medicalConditions",
            "computeTitle"
        },
        "logitem": {
            "date",
            "contents",
            "action",
            "genericType",
            "computeTitle",
            "appliesTo"
        },
        "sessions": {
            "genericType",
            "currentSessionIndex",
            "sessions",
            "currentSession",
            "currentView"
        },
        "phonenumber": {
            "genericType",
            "type",
            "number",
            "computeTitle"
        },
        "website": {
            "genericType",
            "type",
            "url",
            "computeTitle"
        },
        "location": {
            "genericType",
            "latitude",
            "longitude"
        },
        "address": {
            "genericType",
            "type",
            "country",
            "city",
            "street",
            "state",
            "postalCode",
            "location",
            "computeTitle"
        },
        "country": {
            "genericType",
            "name",
            "flag",
            "location",
            "computeTitle"
        },
        "company": {
            "genericType",
            "type",
            "name",
            "computeTitle"
        },
        "publickey": {
            "genericType",
            "type",
            "name",
            "key"
        },
        "onlineprofile": {
            "genericType",
            "type",
            "handle",
            "computeTitle"
        },
        "diet": {
            "genericType",
            "type",
            "name",
            "additions"
        },
        "medicalcondition": {
            "genericType",
            "type",
            "name",
            "computeTitle"
        },
        "session": {
            "genericType",
            "name",
            "currentViewIndex",
            "views",
            "showFilterPanel",
            "showContextPane",
            "editMode",
            "screenshot",
            "currentView"
        },
        "sessionview": {
            "genericType",
            "title",
            "rendererName",
            "subtitle",
            "backTitle",
            "icon",
            "browsingMode",
            "filterText",
            "emptyResultText",
            "showLabels",
            "cascadeOrder",
            "sortFields",
            "selection",
            "activeStates",
            "session",
            "_variables",
            "computeTitle"
        },
    }"#;
    let value: Value = serde_json::from_str(&types).expect("error");

    let mut output = vec![vec![]];
    let current_path = vec![];
    deep_keys(&value, current_path, &mut output);
    output
}

fn deep_keys(value: &Value, current_path: Vec<String>, output: &mut Vec<Vec<String>>) {
    if !current_path.is_empty() {
        output.push(current_path.clone());
    }

    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let mut new_path = current_path.clone();
                new_path.push(k.to_owned());
                deep_keys(v, new_path, output);
            }
        }
        _ => (),
    }
}

// fn set_custom_aliases() {
//     let custom_aliases = r#"
//     {
//         "trip": [{"trip": true}, {"journey": true}],
//         "night out": [{"night out": false}, {"nights out": false}],
//         "bike ride": [{"bike ride": true}, {"bicycle tour": true}],
//         "google conference": [{"google io": false}, {"google conference": true},
//                             {"google seminar": true}, {"google symposion": true}],
//         "atlassian conference": [{"atlassian conference": true}, {"atlassian summit": true},
//                                {"atlassian symposion": true}, {"atlassian seminar": true}],
//         "dawson dance": [{"dawson dance": true}, {"dawson": true}],
//         "sailing trip": [{"sailing": true}, {"sailing trip": true}],
//         "burning man": [{"burning man": true}, {"burning mans": true}, {"the burn": true}],
//         "family weekend": [{"family weekdend": true}, {"family trip": true}]
//     }"#;
//     let aliases: Value = serde_json::from_str(&custom_aliases).expect("error");
//
//     get_aliases()
// }
//
// fn get_aliases() {}

pub fn link_types() {
    let types = set_types();
}
