use crate::configuration::MATRIX_BASE;
use crate::error::Result;
use serde_json::Value;

fn get_room_id(json: &Value) -> String {
    json.get("roomId")
        .expect("Failed to get roomId")
        .as_str()
        .expect("Failed to get &str")
        .to_string()
}

fn get_access_token(json: &Value) -> String {
    json.get("accessToken")
        .expect("Failed to get accessToken")
        .as_str()
        .expect("Failed to get &str")
        .to_string()
}

fn get_message_body(json: &Value) -> &Value {
    json.get("messageBody").expect("Failed to get messageBody")
}

pub fn matrix_login(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/login", MATRIX_BASE);
    let res = client.post(&url).json(&json).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_rooms(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let access_token = get_access_token(&json);
    let url = format!("{}/joined_rooms?access_token={}", MATRIX_BASE, access_token);
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_members(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let room_id = get_room_id(&json);
    let access_token = get_access_token(&json);
    let url = format!(
        "{}/rooms/{}/joined_members?access_token={}",
        MATRIX_BASE, room_id, access_token
    );
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn send_messages(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let room_id = get_room_id(&json);
    let access_token = get_access_token(&json);
    let message_body = get_message_body(&json);
    let url = format!(
        "{}/rooms/{}/send/m.room.message?access_token={}",
        MATRIX_BASE, room_id, access_token
    );
    let res = client.post(&url).json(message_body).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}
