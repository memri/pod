use crate::error::Result;
use log::info;
use serde_json::Value;
use std::process::Command;

pub const MATRIX_BASE: &str = "http://localhost:8008/_matrix/client/r0";

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

fn get_next_batch(json: &Value) -> String {
    json.get("nextBatch")
        .expect("Failed to get nextBatch")
        .as_str()
        .expect("Failed to get &str")
        .to_string()
}

pub fn matrix_register(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/register", MATRIX_BASE);
    let message_body = get_message_body(&json);
    let res = client.post(&url).json(message_body).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn matrix_login(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/login", MATRIX_BASE);
    let message_body = get_message_body(&json);
    let res = client.post(&url).json(message_body).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn create_room(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let access_token = get_access_token(&json);
    let url = format!("{}/createRoom?access_token={}", MATRIX_BASE, access_token);
    let message_body = get_message_body(&json);
    let res = client.post(&url).json(message_body).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_rooms(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let access_token = get_access_token(&json);
    let url = format!("{}/joined_rooms?access_token={}", MATRIX_BASE, access_token);
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn invite_user_to_join(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let room_id = get_room_id(&json);
    let access_token = get_access_token(&json);
    let url = format!(
        "{}/rooms/{}/invite?access_token={}",
        MATRIX_BASE, room_id, access_token
    );
    let message_body = get_message_body(&json);
    let res = client.post(&url).json(message_body).send()?;
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

pub fn sync_events(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let access_token = get_access_token(&json);
    let url = format!("{}/sync?access_token={}", MATRIX_BASE, access_token);
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_messages(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let room_id = get_room_id(&json);
    let next_batch = get_next_batch(&json);
    let access_token = get_access_token(&json);
    let url = format!(
        "{}/rooms/{}/messages?from={}&access_token={}",
        MATRIX_BASE, room_id, next_batch, access_token
    );
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_qrcode(json: Value) -> Result<Value> {
    let room_id = get_room_id(&json);
    let access_token = get_access_token(&json);
    run_auth_web(access_token, room_id);
    let url = "http://localhost:5000";
    Ok(Value::from(format!(
        "Please access {} website for authentication.",
        url
    )))
}

fn run_auth_web(access_token: String, room_id: String) {
    info!("Trying to run web for whatsapp authentication");
    Command::new("python3")
        .args(&["./res/whatsapp_auth.py"])
        .args(&[access_token, room_id])
        .spawn()
        .expect("Failed to run importer");
}
