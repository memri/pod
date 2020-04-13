use crate::internal_api;
use dgraph::Dgraph;
use log::info;
use std::sync::Arc;
use warp::Filter;

pub async fn run_server(server_name: String, dgraph: Dgraph) {
    info!("Starting {} HTTP server", server_name);
    let dgraph = Arc::new(dgraph);

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::version);

    let dgraph_clone = dgraph.clone();
    let get_item = warp::path!("items" / u64)
        .and(warp::get())
        .map(move |id: u64| {
            let json = internal_api::get_item(&dgraph_clone, id);
            warp::reply::json(&json)
        });

    let set_item = warp::path("items")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let uid = internal_api::set_item(&dgraph, body);
            let json = serde_json::json!(uid);
            warp::reply::json(&json)
        });

    warp::serve(version.or(get_item).or(set_item))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
