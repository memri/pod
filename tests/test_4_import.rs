extern crate pod;

use pod::importers::note_importer::*;
use serde::Deserialize;
use serde::Serialize;
use std::str;

mod common;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UidJson {
    pub notes: Vec<Note>,
}

/// Test simple query for get_item()
#[test]
fn it_runs_simple_query() {
    let dgraph = &common::DGRAPH;

    let note: Note = Note {
        title: "aaa".to_string(),
        content: "bbb".to_string(),
        deleted: true,
        // date_created: 1,
        date_modified: 2,
        ..Default::default()
    };

    let assigned_id = insert_note(&dgraph, &note);
    println!("assigned id = {}", assigned_id);

    let query = format!(
        r#"{{
            notes(func: uid({})) {{
                uid,
                title,
                content,
                deleted,
                dateModified,
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
