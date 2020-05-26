extern crate glob;

use dgraph::Dgraph;
use glob::glob;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::str;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "dgraph.type")]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    title: String,
    content: String,
    generic_type: String,
    written_by: Vec<u64>,
    shared_with: Vec<u64>,
    comments: Vec<u64>,
    labels: Vec<u64>,
    version: i32,
    compute_title: String,
    deleted: bool,
    starred: bool,
    date_created: u64,
    date_modified: u64,
    date_accessed: u64,
    functions: Vec<String>,
    changelog: Vec<u64>,
    #[serde(rename = "memriID")]
    memri_id: i64,
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

/// Import all the tags and notes from the file system
pub fn import_notes(dgraph: &Dgraph, directory: String) {
    // First we insert all the tags from tags.json
    #[derive(Serialize, Deserialize, Debug)]
    struct Tags {
        #[serde(flatten)]
        tags: HashMap<String, String>
    }

    let tags_directory = directory.clone() + "/tags.json";
    let content = fs::read_to_string(tags_directory).unwrap();
    let mut tags: Tags = serde_json::from_str(&content).unwrap();

    for tag in &mut tags.tags {
        let assigned_id = insert_tag(dgraph, tag.1.clone());
        println!("Imported ({}) tag : {}", assigned_id, tag.1);
        *tag.1 = assigned_id;
    }

    // Then we read all the note-JSONs
    let note_directory = directory.clone() + "/notes/*.json";
    for file in glob(&note_directory).expect("Failed to read glob pattern") {
        let content = fs::read_to_string(file.unwrap()).unwrap();
        let deserialized: Note = serde_json::from_str(&content).unwrap();
        //println!("The resulting note : {:?}", deserialized);

        let assigned_id = insert_note(&dgraph, &deserialized);
        println!("Imported ({}) note : {}", assigned_id, deserialized.title);
    }

    let resources_directory = directory.clone() + "/resources/*";
    for resource in glob(&resources_directory).expect("Failed to read glob pattern") {
        println!("Found resource : {:?}", resource.unwrap().display());
    }
}

/// Insert a single tag into the database and return the resulting ID as a String
pub fn insert_tag(dgraph: &Dgraph, tag_name: String) -> String {
    #[derive(Serialize, Deserialize, Debug)]
    struct Tag {
        name: String
    }

    let tag = Tag {
        name: tag_name
    };

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&tag).unwrap());
    let response = txn.mutate(mutation).unwrap();

    let result = txn.commit();
    assert_eq!(result.is_ok(), true);

    response.uids.values().next().unwrap().to_string()
}

/// Insert a single note into the database and return the resulting ID as a String
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

/// Simple example querying all notes
pub fn _query_notes(dgraph: &Dgraph) {
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

/// Simple example writing and querying a person
pub fn _simple_example(dgraph: &Dgraph) {
    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    // Create Klaas from struct > JSON
    let person_klaas = Person { name: "klaas".to_string(), phone: Some("456".to_string()) };
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

/// Test simple query for get_item()
#[test]
fn it_runs_simple_query() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client("localhost:9080"));

    let note: Note = Note {
        title: "aaa".to_string(),
        content: "bbb".to_string(),
        deleted: true,
        date_created: 1,
        date_modified: 2,
        ..Default::default()
    };

    let assigned_id = insert_note(&dgraph, &note);
    println!("assigned id = {}", assigned_id);

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct UidJson {
        pub notes: Vec<Note>,
    }
    let query = format!(
        r#"{{
            notes(func: uid({})) {{
                uid,
                title,
                content,
                deleted,
                dateCreated,
                dateModified
            }}
        }}"#,
        assigned_id
    );

    let resp = dgraph.new_readonly_txn().query(query).unwrap();
    let json_str = str::from_utf8(&resp.json).unwrap();
    let json_note: UidJson = serde_json::from_str(&json_str).unwrap();
    println!("{:?}", json_note);

    assert_eq!(json_note.notes[0], note);
}