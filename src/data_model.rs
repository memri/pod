use dgraph::*;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use serde_json::map::Keys;
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug)]
pub struct Items {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub version: u64,
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
        create_edge_property().into_iter().map(|item|
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
pub fn create_edge_property() -> Vec<String> {
    let edge_props = [
        "writtenBy: [uid] ",
        "sharedWith: [uid] ",
        "comments: [uid] ",
        "appliesTo: [uid] ",
        "file: uid ",
        "includes: [uid] ",
        "usedBy: [uid] ",
        "profilePicture: uid ",
        "relations: [uid] ",
        "phoneNumbers: [uid] ",
        "websites: [uid] ",
        "companies: [uid] ",
        "addresses: [uid] ",
        "publicKeys: [uid] ",
        "onlineProfiles: [uid] ",
        "diets: [uid] ",
        "medicalConditions: [uid] ",
        "country: uid ",
        "location: uid ",
        "flag: uid ",
        "changelog: [uid] ",
        "labels: [uid] ",
    ];
    edge_props.iter().map(|x| (*x).to_string()).collect()
}

/// Create node properties of `string` type.
/// Return a vector of tuples -> (`node_property, type, [index]`).
pub fn create_node_string_property() -> Vec<(&'static str, &'static str, [&'static str; 1])> {
    let string_props = vec![
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
    ];

    string_props
        .into_iter()
        .map(|x| (x, "string", ["term"]))
        .collect::<Vec<_>>()
}

/// Create other node properties which are not `string` typed,
/// e.g. int, float, datetime etc.
/// Return a vector of &str -> `node_property: type @index .`.
fn create_node_other_property() -> Vec<&'static str> {
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
        "latitude: float @index(float) .",
        "longitude: float @index(float) .",
        "additions: [string] @index(term) .",
        "deleted: bool .",
        "starred: bool .",
        "dateCreated: datetime .",
        "dateModified: datetime .",
        "dateAccessed: datetime .",
        "functions: [string] .",
        "version: int @index(int).",
        "memriID: int @index(int).",
    ];

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
    let o_prop = create_node_other_property();
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

/// Create type by name
/// Return a vector of &str.
fn get_type_name() -> Vec<&'static str> {
    let type_name = vec![
        "dataitem",
        "note",
        "label",
        "photo",
        "video",
        "audio",
        "file",
        "person",
        "logitem",
        "phonenumber",
        "website",
        "location",
        "address",
        "country",
        "company",
        "publickey",
        "onlineprofile",
        "diet",
        "medicalcondition",
    ];
    type_name
}

/// Create type fields, indicating which properties a type contains.
/// Return a vector of &str vector, according to the order of type name.
fn get_type_field() -> Vec<Vec<&'static str>> {
    let type_field = vec![
        // dataitem
        vec![
            "genericType",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "version",
            "changelog",
            "memriID",
        ],
        // note
        vec![
            "title",
            "content",
            "genericType",
            "writtenBy",
            "sharedWith",
            "comments",
            "labels",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // label
        vec![
            "name",
            "comment",
            "color",
            "genericType",
            "computeTitle",
            "appliesTo",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // photo
        vec![
            "name",
            "file",
            "width",
            "height",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // video
        vec![
            "name",
            "file",
            "width",
            "height",
            "duration",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // audio
        vec![
            "name",
            "file",
            "bitrate",
            "duration",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // file
        vec![
            "uri",
            "genericType",
            "usedBy",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // person
        vec![
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
            "phoneNumbers",
            "websites",
            "companies",
            "addresses",
            "publicKeys",
            "onlineProfiles",
            "diets",
            "medicalConditions",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // logitem
        vec![
            "date",
            "contents",
            "action",
            "genericType",
            "computeTitle",
            "appliesTo",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // phonenumber
        vec![
            "genericType",
            "type",
            "number",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // website
        vec![
            "genericType",
            "type",
            "url",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // location
        vec![
            "genericType",
            "latitude",
            "longitude",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // address
        vec![
            "genericType",
            "type",
            "country",
            "city",
            "street",
            "state",
            "postalCode",
            "location",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // country
        vec![
            "genericType",
            "name",
            "flag",
            "location",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // company
        vec![
            "genericType",
            "type",
            "name",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // publickey
        vec![
            "genericType",
            "type",
            "name",
            "key",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // onlineprofile
        vec![
            "genericType",
            "type",
            "handle",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // diet
        vec![
            "genericType",
            "type",
            "name",
            "additions",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
        // medicalcondition
        vec![
            "genericType",
            "type",
            "name",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    ];
    type_field
}

/// Create types based on type name and fields.
/// Return a vector of formatted type string.
pub fn create_types() -> Vec<String> {
    let type_name = get_type_name();
    let type_field = get_type_field();

    type_name
        .iter()
        .zip(type_field)
        .map(|(name, field)| format_type(name, &field))
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
