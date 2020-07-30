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

fn get_next_batch(json: &Value) -> String {
    json.get("nextBatch")
        .expect("Failed to get nextBatch")
        .as_str()
        .expect("Failed to get &str")
        .to_string()
}

fn get_user_id(json: &Value) -> String {
    json.get("userId")
        .expect("Failed to get userId")
        .as_str()
        .expect("Failed to get &str")
        .to_string()
}

fn get_filter_id(json: &Value) -> String {
    json.get("filterId")
        .expect("Failed to get filterId")
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

pub fn create_filter(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let user_id = get_user_id(&json);
    let access_token = get_access_token(&json);
    let message_body = get_message_body(&json);
    let url = format!(
        "{}/user/{}/filter?access_token={}",
        MATRIX_BASE, user_id, access_token
    );
    let res = client.post(&url).json(message_body).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn sync_events(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let next_batch = get_next_batch(&json);
    let access_token = get_access_token(&json);
    let url = format!(
        "{}/sync?since={}&access_token={}",
        MATRIX_BASE, next_batch, access_token
    );
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_filter(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let user_id = get_user_id(&json);
    let filter_id = get_filter_id(&json);
    let access_token = get_access_token(&json);
    let url = format!(
        "{}/user/{}/filter/{}?access_token={}",
        MATRIX_BASE, user_id, filter_id, access_token
    );
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
