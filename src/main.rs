mod api;
mod api_model;

use api::*;
use rweb::rt::IndexMap;
use rweb::*;
use std::fs::File;
use std::io::prelude::*;

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

    // only write the OpenAPI definition fo a file, for now.
    let openapi_spec_string = serde_json::to_string_pretty(&spec)
        .unwrap_or_else(|err| panic!("Failed to serialize openapi specification to json, {}", err));
    let mut file = File::create("target/openapi.json")
        .unwrap_or_else(|err| panic!("Failed to open openapi.json for writing, {}", err));
    file.write_all(openapi_spec_string.as_bytes())
        .unwrap_or_else(|err| panic!("Failed to write openapi.json, {}", err));
}
