use dgraph::{make_dgraph, Dgraph, Operation};

mod common;

fn is_connected(dgraph: &Dgraph) -> bool {
    let q = "schema {}".to_string();
    let response = dgraph.new_readonly_txn().query(q);

    response.is_ok()
}

/// Test connection
#[test]
fn it_connects() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client(common::DGRAPH_URL));

    assert_eq!(is_connected(&dgraph), true);
}

/// Test schema set up
#[test]
fn it_alters_schema() {
    let dgraph = make_dgraph!(dgraph::new_dgraph_client(common::DGRAPH_URL));

    let result = dgraph.alter(&Operation {
        schema: "something: string .".to_string(),
        ..Default::default()
    });

    assert_eq!(result.is_ok(), true);
}

/// Without new_dgraph_client, should not set up schema
#[test]
#[should_panic]
fn it_does_not_alter_without_client() {
    let dgraph = make_dgraph!();
    let _ = dgraph.alter(&Operation {
        schema: "something: string .".to_string(),
        ..Default::default()
    });
}

/// Without new_dgraph_client, should not do transaction
#[test]
#[should_panic]
fn it_does_not_crate_transaction_without_client() {
    let dgraph = make_dgraph!();
    dgraph.new_txn();
}
