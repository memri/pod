use dgraph::Dgraph;
use log::debug;
use std::sync::Arc;
use std::collections::HashMap;

pub fn version() -> &'static str {
    debug!("Returning API version...");
    env!("CARGO_PKG_VERSION")
}

pub fn hello(user_name: String, server_name: &str) -> String {
    debug!("Saying hello to user {}", user_name);
    format!("Hello, {} from server {}!", user_name, &server_name)
}

pub fn get_item(_dgraph: &Arc<Dgraph>, uid: String) -> String {
    let query = r#"query all($a: string){
    items(func: uid($a)) {
        uid
        deleted
        starred
        version
    }
    }"#
    .to_string();

    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = _dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .expect("query");

    let str = std::str::from_utf8(&resp.json).unwrap();

    str.parse().unwrap()
}



