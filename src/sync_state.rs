use crate::data_model;
use serde_json::value::Number;
use serde_json::Map;
use serde_json::Value;

pub fn get_syncstate(_json: Value) -> Value {
    let mut new_json: Map<String, Value> = _json.get("set").unwrap().as_object().unwrap().clone();
    new_json.remove("syncState").unwrap();
    new_json.insert("version".to_string(), Value::Number(Number::from(1)));
    Value::Object(new_json)
}

pub fn set_syncstate(_json: Vec<u8>) -> String {
    let root: data_model::Items = serde_json::from_slice(&_json).unwrap();
    let version = root.items.first().unwrap().version as f64;

    let json: Value = serde_json::from_slice(&_json).unwrap();
    let mut new_json = json
        .as_object()
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

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

    new_json.remove("version").unwrap();

    let mut sync_state = serde_json::Map::new();
    let is_partially_loaded = Value::Bool(is_part_loaded);
    let new_version = Value::Number(Number::from_f64(version).unwrap());
    sync_state.insert("isPartiallyLoaded".to_string(), is_partially_loaded);
    sync_state.insert("version".to_string(), new_version);

    new_json.insert("syncState".to_string(), Value::from(sync_state));

    Value::Object(new_json).to_string()
}
