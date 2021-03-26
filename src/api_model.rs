use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateItem {
    pub rowid: Option<i64>,
    pub id: Option<String>,
    /// Mandatory type of the item, serialized as just "type" (without underscore)
    pub _type: String,
    pub date_created: Option<i64>,
    pub date_modified: Option<i64>,
    #[serde(default)]
    pub deleted: bool,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    pub id: String,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItemOld {
    pub uid: i64,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEdge {
    pub _type: String,
    pub _source: String,
    pub _target: String,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEdge {
    pub _source: i64,
    pub _target: i64,
    pub _type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BulkAction {
    #[serde(default)]
    pub create_items: Vec<CreateItem>,
    #[serde(default)]
    pub update_items: Vec<UpdateItem>,
    #[serde(default)]
    pub delete_items: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Search {
    pub id: Option<String>,
    pub _type: Option<String>,
    #[serde(rename = "dateServerModified>=")]
    pub _date_server_modified_gte: Option<i64>,
    #[serde(rename = "dateServerModified<")]
    pub _date_server_modified_lt: Option<i64>,
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub other_properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadWrapper<T> {
    pub database_key: String,
    pub payload: T,
}

//
// Services:
//

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunDownloader {
    pub id: String,
    pub service_payload: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunImporter {
    pub id: String,
    pub service_payload: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunIndexer {
    pub id: String,
    pub service_payload: Value,
}

//
// Files:
//

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFile {
    pub sha256: String,
}

//
// Items:
//

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunIntegratorItem {
    pub repository: String,
}
