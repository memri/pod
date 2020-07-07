use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateItem {
    pub uid: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    pub uid: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEdge {
    pub _type: String,
    pub _source: i64,
    pub _target: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEdge {
    pub _type: String,
    pub _source: i64,
    pub _target: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BulkAction {
    #[serde(default)]
    pub create_items: Vec<CreateItem>,
    #[serde(default)]
    pub update_items: Vec<UpdateItem>,
    #[serde(default)]
    pub delete_items: Vec<i64>,
    #[serde(default)]
    pub create_edges: Vec<CreateEdge>,
    #[serde(default)]
    pub delete_edges: Vec<DeleteEdge>,
}
