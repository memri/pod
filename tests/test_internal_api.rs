extern crate pod;

use pod::database_migrate_refinery;
use pod::database_migrate_schema;
use pod::internal_api;
use rusqlite::Connection;
use serde_json::json;
use serde_json::Value;

fn new_conn() -> Connection {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    database_migrate_refinery::embedded::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run refinery migrations");
    database_migrate_schema::migrate(&conn)
        .unwrap_or_else(|err| panic!("Failed to migrate schema, {}", err));
    conn
}

#[test]
fn test_bulk_action() {
    let mut conn = new_conn();

    {
        let tx = conn.transaction().unwrap();
        let json = json!({
            "createItems": [{"uid": 1, "_type": "Person"}, {"uid": 2, "_type": "Person"}],
            "updateItems": [{"uid": 1, "_type": "Person1"}],
            "createEdges": [{"_type": "friend", "_source": 1, "_target": 2, "edgeLabel": "test", "sequence": 1}]
        });
        let result = internal_api::bulk_action_tx(&tx, serde_json::from_value(json).unwrap());
        tx.commit().unwrap();
        assert_eq!(result, Ok(()));
    }

    {
        let tx = conn.transaction().unwrap();
        let get_result = internal_api::get_item_with_edges_tx(&tx, 1);
        assert!(
            get_result.is_ok(),
            "get items with edges failed with: {:?}",
            get_result
        );
    }

    {
        let tx = conn.transaction().unwrap();
        let json = json!({"_type": "Person"});
        let search = internal_api::search_by_fields(&tx, serde_json::from_value(json).unwrap());
        let search = search.expect("Search request failed");
        assert!(!search.is_empty());
    }

    {
        let tx = conn.transaction().unwrap();
        let json = json!({"_type": "Person", "_dateServerModifiedAfter": 1_000_000_000_000_i64 });
        let search = internal_api::search_by_fields(&tx, serde_json::from_value(json).unwrap());
        let search = search.expect("Search request failed");
        assert!(!search.is_empty());
    }

    {
        let tx = conn.transaction().unwrap();
        let json = json!({"_type": "Person", "_dateServerModifiedAfter": 999_000_000_000_000_i64 });
        let search = internal_api::search_by_fields(&tx, serde_json::from_value(json).unwrap());
        let search = search.expect("Search request failed");
        assert_eq!(search, Vec::<Value>::new());
    }
}

#[test]
fn test_insert_item() {
    let mut conn = new_conn();
    let child1_uid = 3;
    let child2_uid = 4;

    {
        let tx = conn.transaction().unwrap();
        let json = json!({
            "_type": "InsertItemSource",
            "dateCreated": 0,
            "_edges": [
                {
                    "_type": "InsertItemEdge",
                    "_target": { "uid": child1_uid, "_type": "InsertItemTarget" }
                },
                {
                    "_type": "InsertItemEdge",
                    "_target": { "uid": child2_uid, "_type": "InsertItemTarget" }
                },
            ]
        });
        let result = internal_api::insert_tree(&tx, serde_json::from_value(json).unwrap());
        result.expect("request failed");
        tx.commit().unwrap();
    }

    {
        let tx = conn.transaction().unwrap();
        let edges = internal_api::get_item_with_edges_tx(&tx, child1_uid).unwrap();
        let edges = edges.get("allEdges").unwrap().as_array().unwrap();
        assert!(edges.is_empty(), "child item has outgoing edges");
    }

    {
        let tx = conn.transaction().unwrap();
        let json = json!({ "_type": "InsertItemSource", "dateCreated": 0 });
        let search = internal_api::search_by_fields(&tx, serde_json::from_value(json).unwrap());
        let search = search.unwrap();
        assert_eq!(search.len(), 1);
        let search = search.first().unwrap();
        let uid = search.get("uid").unwrap().as_i64().unwrap();

        let edges = internal_api::get_item_with_edges_tx(&tx, uid).unwrap();
        let edges = edges.get("allEdges").unwrap().as_array().unwrap();
        assert_eq!(
            edges.len(),
            2,
            "root item does not have exactly 2 outgoing edges"
        );
    }
}
