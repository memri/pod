extern crate pod;

use pod::internal_api::*;

mod common;

#[test]
fn it_searches_by_field() {
    let sqlite = &common::SQLITE;

    let result = get_item_with_edges(&sqlite,1);

    assert_eq!(result.is_ok(), true);
}
