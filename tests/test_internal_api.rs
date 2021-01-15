extern crate pod;

use pod::api_model::CreateItem;
use pod::database_migrate_refinery;
use pod::internal_api;
use pod::schema::Schema;
use pod::schema::SchemaPropertyType;
use rusqlite::Connection;
use serde_json::json;
use std::collections::HashMap;
use warp::hyper::StatusCode;

fn new_conn() -> Connection {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    database_migrate_refinery::embedded::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run refinery migrations");
    conn
}

fn minimal_schema() -> Schema {
    let mut schema = HashMap::new();
    schema.insert("itemType".to_string(), SchemaPropertyType::Text);
    schema.insert("propertyName".to_string(), SchemaPropertyType::Text);
    schema.insert("valueType".to_string(), SchemaPropertyType::Text);
    Schema {
        property_types: schema,
    }
}

#[test]
fn test_item_insert_schema() {
    let mut conn = new_conn();
    let minimal_schema = minimal_schema();

    {
        let tx = conn.transaction().unwrap();
        let json = json!({
            "type": "ItemPropertySchema",
            "itemType": "Person",
            "propertyName": "age",
            "valueType": "Integer",
        });
        let create_item: CreateItem = serde_json::from_value(json.clone()).unwrap();
        let result = internal_api::create_item_tx(&tx, &minimal_schema, create_item).unwrap();
        assert!(result < 10); // no more than 10 items existed before in the DB

        let bad_empty_schema = Schema {
            property_types: HashMap::new(),
        };
        let create_item: CreateItem = serde_json::from_value(json).unwrap();
        let result = internal_api::create_item_tx(&tx, &bad_empty_schema, create_item).unwrap_err();
        assert_eq!(result.code, StatusCode::BAD_REQUEST);
        assert!(result.msg.contains("not defined in Schema"));

        tx.commit().unwrap();
    }
}
