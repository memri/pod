use dgraph::Dgraph;
use glob::glob;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::str;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "dgraph.type", rename_all = "camelCase", default)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub generic_type: String,
    pub written_by: Vec<u64>,
    pub shared_with: Vec<u64>,
    pub comments: Vec<u64>,
    pub labels: Vec<u64>,
    pub version: i32,
    pub compute_title: String,
    pub deleted: bool,
    pub starred: bool,
    pub date_created: u64,
    pub date_modified: u64,
    pub date_accessed: u64,
    pub functions: Vec<String>,
    pub changelog: Vec<u64>,
    pub memri_id: i64,
}

impl Default for Note {
    fn default() -> Self {
        return Note {
            title: String::new(),
            content: String::new(),
            generic_type: "??".to_string(),
            written_by: vec![],
            shared_with: vec![],
            comments: vec![],
            labels: vec![],
            version: 0,
            compute_title: "??".to_string(),
            deleted: false,
            starred: false,
            date_created: 0,
            date_modified: 0,
            date_accessed: 0,
            functions: vec![],
            changelog: vec![],
            memri_id: -1,
        };
    }
}

/// Import all the tags and notes from the file system.
pub fn import_notes(dgraph: &Dgraph, directory: String) {
    // First we insert all the tags from tags.json.
    #[derive(Serialize, Deserialize, Debug)]
    struct Tags {
        #[serde(flatten)]
        tags: HashMap<String, String>,
    }

    let tags_directory = directory.clone() + "/tags.json";
    let content = fs::read_to_string(tags_directory).unwrap();
    let mut tags: Tags = serde_json::from_str(&content).unwrap();

    for tag in &mut tags.tags {
        let assigned_id = insert_tag(dgraph, tag.1.clone());
        info!("Imported ({}) tag : {}", assigned_id, tag.1);
        *tag.1 = assigned_id;
    }

    // Then we read all the note-JSONs.
    let note_directory = directory.clone() + "/notes/*.json";
    for file in glob(&note_directory).expect("Failed to read glob pattern") {
        let content = fs::read_to_string(file.unwrap()).unwrap();
        let deserialized: Note = serde_json::from_str(&content).unwrap();
        //println!("The resulting note : {:?}", deserialized);

        let assigned_id = insert_note(&dgraph, &deserialized);
        info!("Imported ({}) note : {}", assigned_id, deserialized.title);
    }

    let resources_directory = directory.clone() + "/resources/*";
    for resource in glob(&resources_directory).expect("Failed to read glob pattern") {
        info!("Found resource : {:?}", resource.unwrap().display());
    }
}

/// Insert a single tag into the database and return the resulting ID as a String.
pub fn insert_tag(dgraph: &Dgraph, tag_name: String) -> String {
    #[derive(Serialize, Deserialize, Debug)]
    struct Tag {
        name: String,
    }

    let tag = Tag { name: tag_name };

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&tag).unwrap());
    let response = txn.mutate(mutation).unwrap();

    let result = txn.commit();
    assert_eq!(result.is_ok(), true);

    response.uids.values().next().unwrap().to_string()
}

/// Insert a single note into the database and return the resulting ID as a String.
pub fn insert_note(dgraph: &Dgraph, note: &Note) -> String {
    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&note).unwrap());
    let response = txn.mutate(mutation).unwrap();

    let result = txn.commit();
    assert_eq!(result.is_ok(), true);

    // Return the id from the inserted note
    response.uids.values().next().unwrap().to_string()
}

/// Simple example querying all notes.
pub fn _query_notes(dgraph: &Dgraph) {
    let q = r#"{
      all(func: type(Note)) {
        title
        content
      }
    }"#
    .to_string();

    #[derive(Deserialize, Debug)]
    struct Root {
        all: Vec<Note>,
    }

    let resp = dgraph.new_readonly_txn().query(q).expect("query");
    let response: Root = serde_json::from_slice(&resp.json).unwrap();
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

/// Simple example writing and querying a person.
pub fn _simple_example(dgraph: &Dgraph) {
    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    // Create Klaas from struct into JSON.
    let person_klaas = Person {
        name: "klaas".to_string(),
        phone: Some("456".to_string()),
    };
    let json_klaas = serde_json::to_vec(&person_klaas).unwrap();
    println!(
        "json_klaas is : {}",
        serde_json::to_string(&person_klaas).unwrap()
    );
    mutation.set_set_json(json_klaas);
    txn.mutate(mutation).unwrap();

    // Create Kees directly from JSON.
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
    }"#
    .to_string();

    let resp = dgraph.new_readonly_txn().query(q).expect("query");
    println!("Raw JSON-response from DGraph is : {:?}", &resp);
    let root: Root = serde_json::from_slice(&resp.json).expect("parsing");
    println!(
        "When we turn the JSON-response into structs : {:#?}",
        root.all
    );

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
