use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

pub fn set_schema(dgraph: &Dgraph) {
    let op_schema = dgraph::Operation {
        schema: r#"
            deleted: bool .
            starred: bool .
            action: string  @index(exact) .
            color: string @index(exact) .
            comment: string @index(exact) .
            content: string @index(exact) .
            contents: string @index(exact) .
            date: datetime .
            name: string @index(exact) .
            title: string @index(exact) .
            version: int .

            type Note {
                content
                deleted
                starred
                title
                version
            }

            type LogItem {
                action
                contents
                date
                deleted
                starred
                version
            }

            type Label {
                color
                comment
                deleted
                name
                starred
                version
            }
                "#
        .to_string(),
        ..Default::default()
    };

    dgraph.alter(&op_schema).expect("Failed to set schema.");
}
