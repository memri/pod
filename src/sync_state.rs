use crate::data_model;
use serde_json::value::Number;
use serde_json::Map;
use serde_json::Value;

pub fn get_syncstate(_json: Value) -> Value {
    let mut new_json: Map<String, Value> = _json.get("set").unwrap().as_object().unwrap().clone();
    let syncState = new_json.remove("syncState").unwrap();
    new_json.insert("version".to_string(), Value::Number(Number::from(1)));
    Value::Object(new_json)
}

pub fn set_syncstate() -> Value {
    let new_json =
}
