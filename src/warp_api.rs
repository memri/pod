use crate::internal_api;
use bytes::Bytes;
use log::info;
use log::warn;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use warp::http::header::HeaderMap;
use warp::http::header::HeaderValue;
use warp::Filter;
use warp::Reply;

/// Start web framework with specified APIs.
pub async fn run_server(sqlite_pool: Pool<SqliteConnectionManager>) {
    let package_name = env!("CARGO_PKG_NAME").to_uppercase();
    info!("Starting {} HTTP server", package_name);

    // Get version of cargo project POD.
    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);

    let mut headers = HeaderMap::new();
    warn!("Remove Access-Control-Allow-Origin before releasing to PROD!!! :see_no_evil:");
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    let headers = warp::reply::with::headers(headers);

    // Set API version
    let api_version_1 = warp::path("v1");

    let pool_arc = Arc::new(sqlite_pool);

    // GET API for a single item.
    // Parameter:
    //     uid: uid of requested item, signed 8-byte (64bit) integer.
    // Return an array of items with requested uid.
    // Return empty array if item does not exist.
    let pool = pool_arc.clone();
    let get_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |uid: i64| {
            let result = internal_api::get_item(&pool, uid);
            let boxed: Box<dyn Reply> = match result {
                Ok(result) => Box::new(warp::reply::json(&result)),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // GET API for all nodes.
    // Return an array of all nodes.
    // Return empty array if nodes not exist.
    let pool = pool_arc.clone();
    let get_all_items = api_version_1
        .and(warp::path!("all"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            let result = internal_api::get_all_items(&pool);
            let boxed: Box<dyn Reply> = match result {
                Ok(result) => Box::new(warp::reply::json(&result)),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // POST API for a single item.
    // Input: json of created item within the body.
    // Return uid of created item if item is unique.
    // Return error if item already exists.
    let pool = pool_arc.clone();
    let create_item = api_version_1
        .and(warp::path("items"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let result = internal_api::create_item(&pool, body);
            let boxed: Box<dyn Reply> = match result {
                Ok(result) => Box::new(warp::reply::json(&result)),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // PUT (update) a single item
    // Input:
    //      - uid of the item to be updated
    //      - json of content to be updated
    // See `internal_api::update_item` for more details
    let pool = pool_arc.clone();
    let update_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .map(move |uid: i64, body: serde_json::Value| {
            let result = internal_api::update_item(&pool, uid, body);
            let boxed: Box<dyn Reply> = match result {
                Ok(()) => Box::new(warp::reply::json(&serde_json::json!({}))),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // DELETE a single item
    let pool = pool_arc.clone();
    let delete_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::delete())
        .map(move |uid: i64| {
            let result = internal_api::delete_item(&pool, uid);
            let boxed: Box<dyn Reply> = match result {
                Ok(()) => Box::new(warp::reply::json(&serde_json::json!({}))),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // Search items by their fields.
    // Given a JSON like { "author": "Vasili", "type": "note" }
    // the endpoint will return all entries with exactly the same properties.
    let pool = pool_arc.clone();
    let search = api_version_1
        .and(warp::path("search_by_fields"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::bytes())
        .map(move |body: Bytes| {
            let body =
                serde_json::from_slice(&body).expect("Failed to serialize request body to JSON");
            let result = internal_api::search(&pool, body);
            let boxed: Box<dyn Reply> = match result {
                Ok(result) => Box::new(warp::reply::json(&result)),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // Specify APIs.
    // Specify address and port number to listen to.
    warp::serve(
        version
            .with(&headers)
            .or(get_item.with(&headers))
            .or(get_all_items.with(&headers))
            .or(create_item.with(&headers))
            .or(update_item.with(&headers))
            .or(delete_item.with(&headers))
            .or(search.with(&headers)),
    )
    .run(([0, 0, 0, 0], 3030))
    .await;
}
