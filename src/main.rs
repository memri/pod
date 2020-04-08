mod api;
mod api_model;

use api::*;
use rweb::rt::IndexMap;
use rweb::*;

#[tokio::main]
async fn main() {
    let spec = openapi::spec().info(openapi::Info {
        title: "Pod API".into(),
        description: "OpenAPI specification for Pod, the Memri backend".into(),
        terms_of_service: None,
        version: env!("CARGO_PKG_VERSION").into(),
        contact: None,
        license: None,
    });
    let spec = spec.server(openapi::Server {
        url: "pod.memri.io".into(),
        description: "Not really hosted yet. Stay tuned!".into(),
        variables: IndexMap::new(),
    });
    let (spec, _filter) = spec.build(|| version().or(create()));

    // only print the OpenAPI definition for now.
    println!("{}", serde_json::to_string_pretty(&spec).unwrap());
}
