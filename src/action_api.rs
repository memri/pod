use crate::error::Result;
use serde_json::Value;

pub fn matrix_login(json: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://localhost:8008/_matrix/client/r0/login")
        .json(&json)
        .send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}

pub fn get_joined_rooms(token: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let base = "http://localhost:8008/_matrix/client/r0/joined_rooms";
    let access_token = token
        .get("accessToken")
        .expect("Failed to get accessToken")
        .as_str()
        .expect("Failed to get &str");
    let url = format!("{}?access_token={}", base, access_token);
    let res = client.get(&url).send()?;
    Ok(serde_json::from_str(res.text()?.as_str())?)
}
