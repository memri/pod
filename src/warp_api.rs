use crate::internal_api;
use dgraph::Dgraph;
use log::info;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::Filter;
use warp::Reply;

pub async fn run_server(server_name: String, dgraph: Dgraph) {
    info!("Starting {} HTTP server", server_name);
    let dgraph = Arc::new(dgraph);

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::version);

    let dgraph_clone = dgraph.clone();
    let get_item = warp::path!("items" / u64)
        .and(warp::path::end())
        .and(warp::get())
        .map(move |id: u64| {
            let string = internal_api::get_item(&dgraph_clone, id);
            let boxed: Box<dyn Reply> = if let Some(string) = string {
                let json: serde_json::Value = serde_json::from_str(&string).unwrap();
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::NOT_FOUND)
            };
            boxed
        });

    let dgraph_clone = dgraph.clone();
    let get_all_item = warp::path!("all")
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            let string = internal_api::get_all_item(&dgraph_clone);
            let boxed: Box<dyn Reply> = if let Some(string) = string {
                let json: serde_json::Value = serde_json::from_str(&string).unwrap();
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::NOT_FOUND)
            };
            boxed
        });

    let dgraph_clone = dgraph.clone();
    let create_item = warp::path("items")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let uid = internal_api::create_item(&dgraph_clone, body);
            let boxed: Box<dyn Reply> = if let Some(uid) = uid {
                let json = serde_json::json!(uid);
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::CONFLICT)
            };
            boxed
        });

    let dgraph_clone = dgraph.clone();
    let update_item = warp::path!("items" / u64)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .map(move |uid: u64, body: serde_json::Value| {
            let result = internal_api::update_item(&dgraph_clone, uid, body);
            if result {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        });

    let dgraph_clone = dgraph.clone();
    let delete_item = warp::path!("items" / u64)
        .and(warp::path::end())
        .and(warp::delete())
        .map(move |uid: u64| {
            let result = internal_api::delete_item(&dgraph_clone, uid);
            if result {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        });

    warp::serve(
        version
            .or(get_item)
            .or(get_all_item)
            .or(create_item)
            .or(update_item)
            .or(delete_item),
    )
    .run(([127, 0, 0, 1], 3030))
    .await;
}
