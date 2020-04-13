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
        .map(move || internal_api::version());

    let hello = warp::path!("hello" / String)
        .map(move |user_name| internal_api::hello(user_name, &server_name));

    let get_item = warp::path!("items" / String)
        .and(warp::get())
        .map(move |id: String| internal_api::get_item(&dgraph, id));

    warp::serve(version.or(hello).or(get_item))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
