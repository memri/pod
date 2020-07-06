extern crate pod;

use pod::internal_api::*;
use serde_json::json;

mod common;

#[test]
fn it_takes_bulk_action() {
    let sqlite = &common::SQLITE;

    let json = json!({
    "createItems": [{"uid": 1, "_type": "Person"}, {"uid": 2, "_type": "Person"}],
    "updateItems": [{"uid": 1, "_type": "Person1"}],
    "createEdges": [{"_type": "friend", "_source": 1, "_target": 2}]
    });
    let result = bulk_action(&sqlite, json);
    println!("{:#?}", result);
    assert_eq!(result.is_ok(), true);
}
