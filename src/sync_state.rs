use crate::data_model;
use serde_json::value::Number;
use serde_json::value::Value::Array;
use serde_json::Map;
use serde_json::Value;

fn create_new_json(json: &Value) -> Value {
    let mut new_json = json.as_object().unwrap().clone();

    let _fields = new_json.len();
    let type_name = new_json
        .get("type")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .as_str()
        .unwrap();
    let type_count = data_model::get_field_count();
    let field_count = type_count.get(type_name).unwrap();
    let is_part_loaded = &_fields < field_count;

    let version = new_json.get("version").unwrap().as_f64().unwrap() as u64;

    let sync_state = create_sync_state(version, is_part_loaded);

    new_json.remove("version").unwrap();
    new_json.insert("syncState".to_string(), Value::from(sync_state));
    Value::Object(new_json)
}

fn create_sync_state(version: u64, is_part_loaded: bool) -> Map<String, Value> {
    let mut sync_state = serde_json::Map::new();
    let is_partially_loaded = Value::Bool(is_part_loaded);
    let new_version = serde_json::json!(version);
    sync_state.insert("isPartiallyLoaded".to_string(), is_partially_loaded);
    sync_state.insert("version".to_string(), new_version);
    sync_state
}

pub fn get_syncstate(_json: Value) -> Value {
    let mut new_json: Map<String, Value> = _json.get("set").unwrap().as_object().unwrap().clone();
    new_json.remove("syncState").unwrap();
    new_json.insert("version".to_string(), serde_json::json!(1));
    Value::Object(new_json)
}

pub fn set_syncstate(_json: Vec<u8>) -> String {
    let json_value: Value = serde_json::from_slice(&_json).unwrap();
    let json = json_value
        .as_object()
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap();

    let new_json = vec![create_new_json(&json)];
    Value::Array(new_json).to_string()
}

pub fn set_syncstate_all(_json: Vec<u8>) -> String {
    let json: Value = serde_json::from_slice(&_json).unwrap();
    let items = json
        .as_object()
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap();

    let mut new_json = Vec::new();
    for i in 0..items.len() {
        let item = create_new_json(items.get(i).unwrap());
        new_json.insert(i, item);
    }
    Value::Array(new_json).to_string()
}
