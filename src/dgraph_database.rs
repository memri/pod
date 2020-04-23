use dgraph::*;

pub fn create_dgraph() -> Dgraph {
    let dgraph_client = dgraph::new_dgraph_client("localhost:9080");
    Dgraph::new(vec![dgraph_client])
}

pub fn drop_schema(dgraph: &Dgraph) {
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");
}

fn format_e_prop(p: &str) -> String {
    p.to_string() + ": [uid] @reverse ."
}

fn format_n_prop(p: &str, _type: &str, indices: Vec<&str>) -> String {
    let mut joined = String::from(" .");
    if *indices.first().unwrap() != "" {
        joined = String::from("@index(") + &indices.join(",") + ") .";
    }

    p.to_string() + ": " + _type + " " + &joined
}

fn combine(ep: &mut Vec<String>, np: &mut Vec<String>) -> String {
    ep.extend_from_slice(&np);
    ep.join("\n")
}

fn add_schema(dgraph: &Dgraph, schema: dgraph::Operation) {
    dgraph.alter(&schema).expect("Failed to set schema.");
}

fn add_schema_from_properties(e_prop: Vec<String>, n_prop: Vec<(&str,&str,[&str;1])>) -> dgraph::Operation {
    let mut eprops: Vec<String> = vec![];
    eprops.extend(e_prop.iter().map(|x| format_e_prop(x)));

    let mut nprops: Vec<String> = vec![];
    nprops.extend(n_prop.iter()
                      .map(|x| format_n_prop(x.0, x.1, x.2.to_vec())));

    let combine_prop = combine(&mut eprops, &mut nprops);

    let op_schema = dgraph::Operation {
        schema: combine_prop,
        ..Default::default()
    };

    op_schema
}

pub fn set_schema(dgraph: &Dgraph) {
    let edge_prop = ["subtype", "instance", "attends", "parent", "contains", "location", "from", "owns", "diet",
        "friend", "wife", "husband", "grandparent", "grandmother", "sister", "brother", "father", "mother",
        "boyfriend", "girlfriend", "daughter", "son"];
    let mut edge_props = vec![];
    edge_props.extend(edge_prop.iter().map(|x| x.to_string()));

    let s_props = vec!["is_type", "name", "confidence", "profile_picture"];
    let mut string_props = vec![];
    for x in 0..s_props.len() {
        string_props.insert(x, (s_props[x], "string", ["exact"]));
    }
    string_props.push(("creationdate", "DateTime", [""]));
    string_props.push(("aliases", "string", [""]));

    let op_schema = add_schema_from_properties(edge_props, string_props);

    add_schema(dgraph, op_schema);
}


