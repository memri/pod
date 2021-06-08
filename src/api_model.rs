use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

//
// Wrapper structs:
//

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginAuthData {
    pub nonce: String,
    pub encrypted_permissions: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginAuth {
    pub data: PluginAuthData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientAuth {
    pub database_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AuthKey {
    PluginAuth(PluginAuth),
    ClientAuth(ClientAuth),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadWrapper<T> {
    pub auth: AuthKey,
    pub payload: T,
}

//
// Item API:
//

#[derive(Serialize, Deserialize, Debug, Clone)]
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
pub struct Bulk {
    #[serde(default)]
    pub create_items: Vec<CreateItem>,
    #[serde(default)]
    pub update_items: Vec<UpdateItem>,
    #[serde(default)]
    pub delete_items: Vec<String>,
    #[serde(default)]
    pub create_edges: Vec<CreateEdge>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateEdge {
    #[serde(rename = "_source")]
    pub source: String,
    #[serde(rename = "_target")]
    pub target: String,
    #[serde(rename = "_name")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEdges {
    pub item: String,
    pub direction: EdgeDirection,
    pub expand_items: bool,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum EdgeDirection {
    Outgoing,
    Incoming,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Search {
    pub id: Option<String>,
    pub _type: Option<String>,
    #[serde(rename = "dateServerModified>=")]
    pub date_server_modified_gte: Option<i64>,
    #[serde(rename = "dateServerModified<")]
    pub date_server_modified_lt: Option<i64>,
    pub deleted: Option<bool>,
    #[serde(default = "default_api_sort_order", rename = "_sortOrder")]
    pub sort_order: SortOrder,
    #[serde(default = "default_api_limit", rename = "_limit")]
    pub limit: u64,
    #[serde(flatten)]
    pub other_properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SortOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}
impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
fn default_api_sort_order() -> SortOrder {
    SortOrder::Asc
}
fn default_api_limit() -> u64 {
    u64::MAX
}

//
// Files API:
//

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFile {
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_examples() {
        let client_auth = ClientAuth {
            database_key: "0000".to_string(),
        };
        let client_auth_ser = serde_json::to_string(&client_auth).unwrap();
        assert_eq!(client_auth_ser, r#"{"databaseKey":"0000"}"#);
        let auth_key = AuthKey::ClientAuth(client_auth);
        let auth_key_ser = serde_json::to_string(&auth_key).unwrap();
        assert_eq!(
            auth_key_ser,
            r#"{"type":"ClientAuth","databaseKey":"0000"}"#
        );
    }
}
