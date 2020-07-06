extern crate pod;

use pod::internal_api::*;
use serde_json::json;
use serial_test::serial;

mod common;

#[test]
#[serial]
fn test_bulk_action() {
    let sqlite = &common::SQLITE;

    let json = json!({
    "createItems": [{"uid": 1, "_type": "Person"}, {"uid": 2, "_type": "Person"}],
    "updateItems": [{"uid": 1, "_type": "Person1"}],
    "createEdges": [{"_type": "friend", "_source": 1, "_target": 2}]
    });
    let result = bulk_action(&sqlite, json);

    assert_eq!(result.is_ok(), true);
}

#[test]
#[serial]
fn test_item_with_edges() {
    let sqlite = &common::SQLITE;

    let result = get_item_with_edges(&sqlite, 1);
    let mut has_item = false;

    if let Some(items) = result.iter().next() {
        if items.iter().next().is_some() {
            has_item = true;
        }
    }

    assert_eq!(has_item, true);
}

#[test]
#[serial]
fn test_searches_by_field() {
    let sqlite = &common::SQLITE;

    let json = json!({"_type": "Person"});
    let result = search(&sqlite, json);
    let mut has_item = false;

    if let Some(items) = result.iter().next() {
        if items.iter().next().is_some() {
            has_item = true;
        }
    }

    assert_eq!(has_item, true);
}
