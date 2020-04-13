use dgraph::Dgraph;
use log::debug;
use std::sync::Arc;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

pub fn hello(user_name: String, server_name: &str) -> String {
    debug!("Saying hello to user {}", user_name);
    format!("Hello, {} from server {}!", user_name, &server_name)
}

pub fn get_item(_dgraph: &Arc<Dgraph>, uid: String) -> String {

}

//pub fn get_list(_dgraph: &Arc<Dgraph>) -> Json<Vec<DataItem>> {
//    let query = format!(
//        r#"{{
//            items(func: has(deleted)) {{
//                uid
//                deleted
//                starred
//                version
//                }}
//            }}"#
//    );
//
//    let resp = DGRAPH
//        .new_readonly_txn()
//        .query(query)
//        .expect("query");
//
//    let items: ItemList = serde_json::from_slice(&resp.json).expect("Failed to serialize JSON.");
//
//    Json::from(items.items)
//}

