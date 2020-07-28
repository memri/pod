use crate::configuration::MATRIX_BASE;
use crate::error::Result;
use serde_json::Value;

pub fn matrix_login(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/login", MATRIX_BASE);
    let res = client.post(&url).json(&json).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_rooms(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let access_token = json
        .get("accessToken")
        .expect("Failed to get accessToken")
        .as_str()
        .expect("Failed to get &str");
    let url = format!("{}/joined_rooms?access_token={}", MATRIX_BASE, access_token);
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_members(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let room_id = json
        .get("roomId")
        .expect("Failed to get roomId")
        .as_str()
        .expect("Failed to get &str");
    let accss_token = json
        .get("accessToken")
        .expect("Failed to get accessToken")
        .as_str()
        .expect("Failed to get &str");
    let url = format!(
        "{}/rooms/{}/joined_members?access_token={}",
        MATRIX_BASE, room_id, accss_token
    );
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}
