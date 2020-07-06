extern crate pod;

use pod::internal_api::*;
use serde_json::json;

mod common;

#[test]
fn it_searches_by_field() {
    let sqlite = &common::SQLITE;

    let json = json!({"_type": "Person"});
    let result = search(&sqlite,json);

    assert_eq!(result.is_ok(), true);
}
