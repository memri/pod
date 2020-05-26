extern crate pod;

use pod::internal_api::*;
use serde_json::json;
use serde_json::Value;
use serial_test::serial;

mod common;

/// Test query with get_item().
/// Return `None`.
#[test]
#[serial]
fn get_item_returns_none() {
    let dgraph = &common::DGRAPH;

    let memri_id = "10".to_string();
    let result = get_item(&dgraph, memri_id);

    assert_eq!(result, None);
}

/// Test mutation with create_item().
#[test]
#[serial]
fn it_commits_a_mutation() {
    let dgraph = &common::DGRAPH;

    let json = json!(
        {
        "memriID": "1",
        "firstName": "Alice",
        "type": "person",
        "version": 1
        }
    );

    let result = create_item(&dgraph, json);

    assert_ne!(result, None);
}

/// Test query with get_item().
/// Return item with `memriID`=1.
#[test]
#[serial]
fn it_gets_an_item() {
    let dgraph = &common::DGRAPH;

    let memri_id = "1".to_string();
    let result = get_item(&dgraph, memri_id.clone());
    let json: Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(
        json.as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_object()
            .unwrap()
            .get("memriID")
            .unwrap()
            .as_str()
            .unwrap(),
        memri_id
    );
}

/// Test query with update_item().
#[test]
#[serial]
fn it_puts_an_update() {
    let dgraph = &common::DGRAPH;

    let memri_id = "1".to_string();
    let json = json!(
        {
        "memriID": "1",
        "firstName": "Bob",
        "type": "person",
        "version": 2
        }
    );
    let result = update_item(&dgraph, memri_id, json);

    assert_eq!(result, true);
}

/// Test query with delete_item().
#[test]
#[serial]
fn it_removes_an_item() {
    let dgraph = &common::DGRAPH;

    let memri_id = "1".to_string();
    let result = delete_item(&dgraph, memri_id);

    assert_eq!(result, true);
}
