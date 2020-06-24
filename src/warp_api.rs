use crate::internal_api;
use bytes::Bytes;
use log::debug;
use log::info;
use log::warn;
use warp::http::StatusCode;
use warp::Filter;
use warp::Reply;

/// Start web framework with specified APIs.
pub async fn run_server() {
    info!(
        "Starting {} HTTP server",
        env!("CARGO_PKG_NAME").to_uppercase()
    );
    // Get version of cargo project POD.
    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);
    // Set API version
    let api_version_1 = warp::path("v1");
    // GET API for a single item.
    // Parameter:
    //     id: id of requested item, integer.
    // Return an array of nodes with requested id.
    // Return StatusCode::NOT_FOUND if item does not exist.
    let get_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |id: i64| {
            let string = internal_api::get_item(id);
            let boxed: Box<dyn Reply> = if let Some(string) = string {
                let json: serde_json::Value = serde_json::from_str(&string).unwrap();
                debug!("Response: {}", &json);
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::NOT_FOUND)
            };
            boxed
        });
    // GET API for all nodes.
    // Return an array of all nodes.
    // Return StatusCode::NOT_FOUND if nodes not exist.
    let get_all_items = api_version_1
        .and(warp::path!("all"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            let string = internal_api::get_all_items();
            let boxed: Box<dyn Reply> = if let Some(string) = string {
                let json: serde_json::Value = serde_json::from_str(&string).unwrap();
                debug!("Response: {}", &json);
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::NOT_FOUND)
            };
            boxed
        });
    // POST API for a single node.
    // Input: json of created node within the body.
    // Return uid of created node if node is unique.
    // Return StatusCode::CONFLICT if node already exists.
    let create_item = api_version_1
        .and(warp::path("items"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let uid = internal_api::create_item(body);
            let boxed: Box<dyn Reply> = if let Some(uid) = uid {
                let json = serde_json::json!(uid);
                debug!("Response: {}", &json);
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(warp::reply::with_status("Item contains an uid that already exists in the database, use update() instead.", StatusCode::CONFLICT))
            };
            boxed
        });
    // PUT API for a single node.
    // Parameter:
    //     mid: memriID of the node to be updated.
    // Return without body:
    //     StatusCode::OK if node has been updated successfully.
    //     StatusCode::NOT_FOUND if node is not found in the database.
    let update_item = api_version_1
        .and(warp::path!("items" / String))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .map(move |mid: String, body: serde_json::Value| {
            let result = internal_api::update_item(mid, body);
            if result {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        });
    // DELETE API for a single node.
    // Parameter:
    //     mid: memriID of the node to be deleted.
    // Return without body:
    //     StatusCode::OK if node has been deleted successfully.
    //     StatusCode::NOT_FOUND if node was not found in the database.
    let delete_item = api_version_1
        .and(warp::path!("items" / String))
        .and(warp::path::end())
        .and(warp::delete())
        .map(move |mid: String| {
            let result = internal_api::delete_item(mid);
            if result {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        });
    // QUERY API for a subset of nodes.
    // Input: json of query within the body.
    // Return an array of nodes.
    // Return StatusCode::NOT_FOUND if nodes not exist.
    let query = api_version_1
        .and(warp::path("all"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::bytes())
        .map(move |body: Bytes| {
            let string = internal_api::query(body);
            let boxed: Box<dyn Reply> = if let Some(string) = string {
                let json: serde_json::Value = serde_json::from_str(&string).unwrap();
                debug!("Response: {}", &json);
                Box::new(warp::reply::json(&json))
            } else {
                Box::new(StatusCode::NOT_FOUND)
            };
            boxed
        });
    // IMPORT API to start importing notes.
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
            .or(query)
            .or(import_notes),
    )
    .run(([0, 0, 0, 0], 3030))
    .await;
}
