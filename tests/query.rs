use dgraph::make_dgraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str;

mod common;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UidJson {
    pub uids: Vec<Uid>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Uid {
    pub uid: String,
}

/// Test simple query for get_item()
#[test]
fn it_runs_simple_query() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client(common::DGRAPH_URL));

    let uid = "0x1";
    let query = format!(
        r#"{{
            uids(func: uid({})) {{
                uid,
            }}
        }}"#,
        uid
    );

    let resp = dgraph.new_readonly_txn().query(query).unwrap();
    let json_str = str::from_utf8(&resp.json).unwrap();
    let json: UidJson = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json.uids[0].uid, uid);
}

/// Test query with variables for get_item()
#[test]
fn it_runs_query_with_vars() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client(common::DGRAPH_URL));

    let uid = "0x1";
    let query = r#"query all($a: string){
        uids(func: uid($a)) {
            uid,
        }
    }"#
    .to_string();
    let mut vars = HashMap::new();
    vars.insert("$a".to_string(), uid.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(query, vars)
        .unwrap();
    let json: UidJson = serde_json::from_slice(&resp.json).unwrap();

    assert_eq!(json.uids[0].uid, uid);
}

/// Test mutation for create_item()
#[test]
fn it_commits_a_mutation() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client(common::DGRAPH_URL));

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();

    mutation.set_set_json(br#"{"name": "Alice"}"#.to_vec());
    txn.mutate(mutation).unwrap();
    let result = txn.commit();

    assert_eq!(result.is_ok(), true);
}
