extern crate pod;

use pod::internal_api::*;
use serde_json::json;
use serde_json::Value;

mod common;

#[test]
fn test_bulk_action() {
    let sqlite = &common::SQLITE;

    let json = json!({
    "createItems": [{"uid": 1, "_type": "Person"}, {"uid": 2, "_type": "Person"}],
    "updateItems": [{"uid": 1, "_type": "Person1"}],
    "createEdges": [{"_type": "friend", "_source": 1, "_target": 2}]
    });

    let result1 = bulk_action(&sqlite, json);

    let result2 = get_item_with_edges(&sqlite, 1);

    let json = json!({"_type": "Person"});
    let result3 = search_by_fields(&sqlite, json);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(check_has_item(result3.ok()));
}

fn check_has_item(result: Option<Vec<Value>>) -> bool {
    result.iter().flatten().next().is_some()
}
