use dgraph::*;
use serde::Deserialize;
use serde::Serialize;

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
    edge_prop.iter().map(|x| (*x).to_string()).collect()
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
    ];

    s_props
        .into_iter()
        .map(|x| (x, "string", ["term"]))
        .collect::<Vec<_>>()
}

fn create_other_property() -> Vec<&'static str> {
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
        "version: int .",
    ];

    other_props
}

pub fn get_schema_from_properties(
    e_prop: Vec<String>,
    n_prop: Vec<(&str, &str, [&str; 1])>,
) -> dgraph::Operation {
    let mut eprops: Vec<String> = vec![];
    eprops.extend(e_prop.iter().map(|x| format_e_prop(x)));

    let mut nprops: Vec<String> = vec![];
    nprops.extend(n_prop.iter().map(|x| format_n_prop(x.0, x.1, x.2.to_vec())));
    let o_prop = create_other_property();
    nprops.extend(o_prop.iter().map(|x| format_o_prop(x)));

    let combine_prop = combine(&mut eprops, &mut nprops);

    dgraph::Operation {
        schema: combine_prop,
        ..Default::default()
    }
}

fn format_e_prop(p: &str) -> String {
    p.to_string() + "@reverse ."
}

fn format_n_prop(p: &str, _type: &str, indices: Vec<&str>) -> String {
    let joined = if *indices.first().unwrap() != "" {
        format!("@index({}) .", indices.join(","))
    } else {
        String::from(" .")
    };

    p.to_string() + ": " + _type + " " + &joined
}

fn format_o_prop(p: &str) -> String {
    p.to_string()
}

fn combine(ep: &mut Vec<String>, np: &mut Vec<String>) -> String {
    ep.extend_from_slice(&np);
    ep.join("\n")
}

pub fn add_schema(dgraph: &Dgraph, schema: dgraph::Operation) {
    dgraph.alter(&schema).expect("Failed to set schema.");
}

pub fn create_types() -> Vec<String> {
    let types = vec![
        "dataitem {
            genericType
            computeTitle
            deleted
            starred
            dateCreated
            dateModified
            dateAccessed
            functions
            version
         }",
        "note {
            title
            content
            genericType
            writtenBy
            sharedWith
            comments
            labels
            version
         }",
        "label {
            name
            comment
            color
            genericType
            computeTitle
            appliesTo
            version
         }",
        "photo {
            name
            file
            width
            height
            genericType
            computeTitle
            includes
            version
         }",
        "video {
            name
            file
            width
            height
            duration
            genericType
            computeTitle
            includes
            version
         }",
        "audio {
            name
            file
            bitrate
            duration
            genericType
            computeTitle
            includes
            version
         }",
        "file {
            uri
            genericType
            usedBy
            version
         }",
        "person {
            firstName
            lastName
            birthDate
            gender
            sexualOrientation
            height
            shoulderWidth
            armLength
            age
            genericType
            profilePicture
            relations
            phoneNumbers
            websites
            companies
            addresses
            publicKeys
            onlineProfiles
            diets
            medicalConditions
            computeTitle
            version
         }",
        "logitem {
            date
            contents
            action
            genericType
            computeTitle
            appliesTo
            version
         }",
        "phonenumber {
            genericType
            type
            number
            computeTitle
            version
         }",
        "website {
            genericType
            type
            url
            computeTitle
            version
         }",
        "location {
            genericType
            latitude
            longitude
            version
         }",
        "address {
            genericType
            type
            country
            city
            street
            state
            postalCode
            location
            computeTitle
            version
         }",
        "country {
            genericType
            name
            flag
            location
            computeTitle
            version
         }",
        "company {
            genericType
            type
            name
            computeTitle
            version
         }",
        "publickey {
            genericType
            type
            name
            key
            version
         }",
        "onlineprofile {
            genericType
            type
            handle
            computeTitle
            version
         }",
        "diet {
            genericType
            type
            name
            additions
            version
         }",
        "medicalcondition {
            genericType
            type
            name
            computeTitle
            version
        }",
    ];

    types
        .into_iter()
        .map(|x| (format_type(x)))
        .collect::<Vec<_>>()
}

fn format_type(p: &str) -> String {
    String::from("type ") + &p.to_string()
}
pub fn get_schema_from_types(types: Vec<String>) -> dgraph::Operation {
    dgraph::Operation {
        schema: types.join("\n"),
        ..Default::default()
    }
}
