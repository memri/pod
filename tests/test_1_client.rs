use dgraph::*;

mod common;

fn is_connected(dgraph: &Dgraph) -> bool {
    let q = "schema {}".to_string();
    let response = dgraph.new_readonly_txn().query(q);

    response.is_ok()
}

/// Test connection.
#[test]
fn it_connects() {
    let dgraph = &common::DGRAPH;

    assert_eq!(is_connected(&dgraph), true);
}

/// Test adding schema.
#[test]
fn it_alters_schema() {
    let dgraph = &common::DGRAPH;

    let result = dgraph.alter(&Operation {
        schema: "firstName: string @index(term) .\nmemriID: string @index(term) .\nversion: int .\ntype person {\nfirstName\nmemriID\nversion\n}\n"
            .to_string(),
        ..Default::default()
    });

    assert_eq!(result.is_ok(), true);
}
