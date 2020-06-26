use crate::internal_api;
use bytes::Bytes;
use log::info;
use log::warn;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
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
    // Set API version
    let api_version_1 = warp::path("v1");

    let pool_arc = Arc::new(sqlite_pool);

    // GET API for a single item.
    // Parameter:
    //     id: id of requested item, integer.
    // Return an array of items with requested id.
    // Return empty array if item does not exist.
    let pool = pool_arc.clone();
    let get_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |id: i64| {
            let result = internal_api::get_item(&pool, id);
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
    // Return id of created item if item is unique.
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
    //      - id of the item to be updated
    //      - json of content to be updated
    // See `internal_api::update_item` for more details
    let pool = pool_arc.clone();
    let update_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .map(move |id: i64, body: serde_json::Value| {
            let result = internal_api::update_item(&pool, id, body);
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
        .map(move |id: i64| {
            let result = internal_api::delete_item(&pool, id);
            let boxed: Box<dyn Reply> = match result {
                Ok(()) => Box::new(warp::reply::json(&serde_json::json!({}))),
                Err(err) => Box::new(warp::reply::with_status(err.msg, err.code)),
            };
            boxed
        });

    // Search items by their fields.
    // Given a JSON like { "author": "Vasili", "_type": "note" }
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

    // IMPORT API to trigger notes importing
    let import_notes = api_version_1
        .and(warp::path("import"))
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .map(move |import_service: String, import_type: String| {
            info!("trying to import {} from {}", import_type, import_service);
            match (import_service.as_str(), import_type.as_str()) {
                ("Evernote", "notes") => unimplemented!(),
                ("iCloud", "notes") => {}
                (_, "notes") => warn!("UNKNOWN SERVICE : {}", import_service),
                (_, _) => warn!("UNKNOWN TYPE : {}", import_type),
            }
            ""
        });

    // Specify APIs.
    // Specify address and port number to listen to.
    warp::serve(
        version
            .or(get_item)
            .or(get_all_items)
            .or(create_item)
            .or(update_item)
            .or(delete_item)
            .or(search)
            .or(import_notes),
    )
    .run(([0, 0, 0, 0], 3030))
    .await;
}
