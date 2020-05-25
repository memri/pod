extern crate glob;
use glob::glob;

use dgraph::Dgraph;
use std::str;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "dgraph.type")]
#[serde(default)]
// struct Note {
//     //notebooks: Vec<String>,
//     creator: String,
//     id: String,
//     title: String,
//     content: String,
//     created: Option<u64>,
//     modified: Option<u64>,
//     deleted: Option<u64>,
//     tags: Vec<String>,
//     resources: Vec<String>
// }
struct Note {
    title: String,
    content: String,
    genericType: String,
    writtenBy: Vec<u64>,
    sharedWith: Vec<u64>,
    comments: Vec<u64>,
    labels: Vec<u64>,
    version: i32,
    computeTitle: String,
    deleted: bool,
    starred: bool,
    dateCreated: u64,
    dateModified: u64,
    dateAccessed: u64,
    functions: Vec<String>,
    changelog: Vec<u64>,
    memriID: i64,
}

impl Default for Note {
    fn default() -> Self {
        return Note {
            title: String::new(),
            content: String::new(),
            genericType: "??".to_string(),
            writtenBy: vec![],
            sharedWith: vec![],
            comments: vec![],
            labels: vec![],
            version: 0,
            computeTitle: "??".to_string(),
            deleted: false,
            starred: false,
            dateCreated: 0,
            dateModified: 0,
            dateAccessed: 0,
            functions: vec![],
            changelog: vec![],
            memriID: -1,
        }
    }
}

pub fn import_notes(dgraph: &Dgraph) {
    for file in glob("data/notes/*.json").expect("Failed to read glob pattern") {
        let content = fs::read_to_string(file.unwrap()).unwrap();
        let deserialized: Note = serde_json::from_str(&content).unwrap();
        println!("The resulting note : {:?}", deserialized);

        let mut txn = dgraph.new_txn();
        let mut mutation = dgraph::Mutation::new();
        mutation.set_set_json(serde_json::to_vec(&deserialized).unwrap());
        txn.mutate(mutation).unwrap();

        let result = txn.commit();
        assert_eq!(result.is_ok(), true);

        println!("Imported note : {}", deserialized.title);
    }

    for resource in glob("data/resources/*").expect("Failed to read glob pattern") {
        println!("Found resource : {:?}", resource.unwrap().display());
    }
}

pub fn query_notes(dgraph: &Dgraph) {
    let q = r#"{
      all(func: type(Note)) {
        title
        content
      }
    }"#.to_string();

    #[derive(Deserialize, Debug)]
    struct Root {
        all: Vec<Note>,
    }

    let resp = dgraph.new_readonly_txn().query(q).expect("query");
    let response : Root = serde_json::from_slice(&resp.json).unwrap();
    for note in &response.all {
        println!("{:?}", &note);
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Root {
    all: Vec<Person>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(tag = "dgraph.type")]
struct Person {
    name: String,
    phone: Option<String>,
}

pub fn simple_example(dgraph: &Dgraph) {
    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    // Create Klaas from struct > JSON
    let person_klaas = Person {name: "klaas".to_string(), phone: Some("456".to_string())};
    let json_klaas = serde_json::to_vec(&person_klaas).unwrap();
    println!("json_klaas is : {}", serde_json::to_string(&person_klaas).unwrap());
    mutation.set_set_json(json_klaas);
    txn.mutate(mutation).unwrap();

    // Create Kees directly from JSON
    let mut mutation = dgraph::Mutation::new();
    let json_kees = r#"{"name": "kees", "phone": "400", "dgraph.type":"Person"}"#;
    println!("json_kees is : {}", &json_kees);
    mutation.set_set_json(json_kees.as_bytes().to_vec());
    txn.mutate(mutation).unwrap();

    // Not completely sure, can we include two mutations in one transaction?
    // Or should we check result in between?
    let result = txn.commit();
    assert_eq!(result.is_ok(), true);

    // Request all objects of type Person from DGraph
    let q = r#"{
      all(func: type(Person)) {
        name
        phone
      }
    }"#.to_string();

    let resp = dgraph.new_readonly_txn().query(q).expect("query");
    println!("Raw JSON-response from DGraph is : {:?}", &resp);
    let root: Root = serde_json::from_slice(&resp.json).expect("parsing");
    println!("When we turn the JSON-response into structs : {:#?}", root.all);

    // Request all things called "klaas" from DGraph, this does not work yet as
    // "Predicate name is not indexed"

    // let q = r#"query all($a: string) {
    //     all(func: eq(name, $a)) {
    //       name
    //       phone
    //     }
    //   }"#.to_string();
    // let mut vars = HashMap::new();
    // vars.insert("$a".to_string(), "klaas".to_string());
    //
    // let resp = dgraph.new_readonly_txn().query_with_vars(q, vars).expect("query");
    // let root: Root = serde_json::from_slice(&resp.json).expect("parsing");
    // println!("When we turn the JSON-response into structs : {:#?}", root.all);
}