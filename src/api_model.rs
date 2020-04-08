use rweb::*;
use serde::Deserialize;
use serde::Serialize;

/// DataItem used to ... (TODO clarify when the project goes out of PoC stage)
#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "DataItem")]
pub struct DataItem {
    /// ID of the DataItem.
    /// Can be an arbitrary string, don't rely on it being UUID.
    /// (This needs discussion with ios-application.)
    #[schema(example = "\"7fde8928-a67d-4d47-824e-e58992b2832a\"")]
    id: String,
    /// description of deleted (TODO delete or make meaningful)
    #[schema(example = "false")]
    deleted: bool,
    /// description of starred (TODO delete or make meaningful)
    #[schema(example = "false")]
    starred: bool,
}
