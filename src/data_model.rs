use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

/// dgraph uid.
/// It works as a reference to a dgraph node and
/// is guaranteed to be unique for a node by dgraph.
pub type UID = u64;

// tag="type" adds a "type" field during JSON serialization
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct AuditAccessLog {
    pub audit_target: NodeReference,
    pub date_created: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeReference {
    pub uid: UID,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: i64,
    pub _type: String,
    pub created_at: f64,
    pub modified_at: f64,
    pub read_at: f64,
    pub version: i64,
}
