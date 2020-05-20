use crate::data_model;
use serde_json::Map;
use serde_json::Value;

/// Modify json by adding `syncState`.
/// Add `syncState`, including `isPartiallyLoaded` and `version`, remove original `version`.
/// Return the modified json object.
fn add_sync_state(json: &Value) -> Value {
    let mut new_json = json.as_object().unwrap().clone();
    // Compare the number of returned properties/fields
    // with the default number of fields that type contains
    // to decide if all properties are included/loaded.
    let _fields = json.as_object().unwrap();
    let type_name = json
        .get("dgraph.type")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .as_str()
        .unwrap()
        .clone();
    let field_count = &data_model::FIELD_COUNT.get(type_name).unwrap();
    let is_part_loaded = &_fields.len() < field_count;

    let hex_uid: data_model::UID = serde_json::from_value(json.clone()).unwrap();
    let without_pre = hex_uid.uid.trim_start_matches("0x");
    let uid = u64::from_str_radix(without_pre, 16).unwrap();

    // Create `syncState` and insert to new json as a Value.
    let sync_state = create_sync_state(is_part_loaded);

    new_json.remove("uid").unwrap();
    new_json.insert("uid".to_string(), Value::from(uid));
    new_json.remove("dgraph.type").unwrap();
    new_json.insert("syncState".to_string(), Value::from(sync_state));
    new_json.insert("type".to_string(), Value::from(type_name));
    Value::Object(new_json)
}

/// Create `syncState` with `isPartiallyLoaded`.
/// Return a hashmap -> `<isPartiallyLoaded: bool>`.
fn create_sync_state(is_part_loaded: bool) -> Map<String, Value> {
    let mut sync_state = serde_json::Map::new();
    let is_partially_loaded = Value::Bool(is_part_loaded);
    sync_state.insert("isPartiallyLoaded".to_string(), is_partially_loaded);
    sync_state
}

/// Remove `syncState` from client json.
/// Add `version` to the original json to be stored in dgraph.
/// Return the new json as a Value.
pub fn get_syncstate(_json: Value, version: u64) -> Value {
    let mut new_json: Map<String, Value> = _json.as_object().unwrap().clone();
    let type_name = _json
        .as_object()
        .unwrap()
        .get("type")
        .unwrap()
        .as_str()
        .unwrap();
    new_json.remove("type").unwrap();
    new_json.insert("dgraph.type".to_string(), serde_json::json!(type_name));
    new_json.remove("version").unwrap();
    new_json.insert("version".to_string(), serde_json::json!(version));
    Value::Object(new_json)
}

/// Create a vector of new json as response to get_item().
/// Return a string of the vector.
pub fn set_syncstate(json_value: Value) -> String {
    let json = json_value
        .as_object()
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap();

    let new_json = vec![add_sync_state(&json)];
    Value::Array(new_json).to_string()
}

/// Create a vector of new json as response to get_all_items().
/// Return a string of the vector.
pub fn set_syncstate_all(json_value: Value) -> String {
    let items = json_value
        .as_object()
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap();

    let mut new_json = Vec::new();
    for i in 0..items.len() {
        let item = add_sync_state(items.get(i).unwrap());
        new_json.insert(i, item);
    }
    Value::Array(new_json).to_string()
}
