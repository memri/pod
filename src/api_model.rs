use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItem {
    pub uid: i64,
    #[serde(rename = "dateCreated")]
    date_created: (),
    #[serde(rename = "dateModified")]
    date_modified: (),
    version: (),
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    pub uid: i64,
    _type: (),
    #[serde(rename = "dateCreated")]
    date_created: (),
    #[serde(rename = "dateModified")]
    date_modified: (),
    version: (),
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateEdge {
    pub _type: String,
    pub _source: i64,
    pub _target: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkActions {
    pub create_items: Vec<CreateItem>,
    pub update_items: Vec<UpdateItem>,
    pub create_edges: Vec<CreateEdge>,
}
