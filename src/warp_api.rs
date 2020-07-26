use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::PayloadWrapper;
use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::api_model::UpdateItem;
use crate::internal_api;
use crate::warp_endpoints;
use log::info;
use log::warn;
use serde_json::Value;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;
use warp::http;
use warp::http::header::HeaderMap;
use warp::http::header::HeaderValue;
use warp::http::status::StatusCode;
use warp::reply::Response;
use warp::Filter;
use warp::Reply;

/// Start web framework with specified APIs.
pub async fn run_server() {
    let package_name = env!("CARGO_PKG_NAME").to_uppercase();
    info!("Starting {} HTTP server", package_name);

    let mut headers = HeaderMap::new();
    warn!("Always adding the insecure Access-Control-Allow-Origin header for development purposes");
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    let headers = warp::reply::with::headers(headers);

    let api_defaults = warp::path("v2")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::post());

    let initialized_databases_arc = Arc::new(RwLock::new(HashSet::<String>::new()));

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);

    let init_db = initialized_databases_arc.clone();
    let get_item = api_defaults
        .and(warp::path!(String / "get_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<i64>| {
            let result = warp_endpoints::get_item(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let get_all_items = api_defaults
        .and(warp::path!(String / "get_all_items"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<()>| {
            let result = warp_endpoints::get_all_items(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let create_item = api_defaults
        .and(warp::path!(String / "create_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<CreateItem>| {
            let result = warp_endpoints::create_item(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let update_item = api_defaults
        .and(warp::path!(String / "update_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<UpdateItem>| {
            let result = warp_endpoints::update_item(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let bulk_action = api_defaults
        .and(warp::path!(String / "bulk_action"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<BulkAction>| {
            let result = warp_endpoints::bulk_action(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let delete_item = api_defaults
        .and(warp::path!(String / "delete_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<i64>| {
            let result = warp_endpoints::delete_item(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let search = api_defaults
        .and(warp::path!(String / "search_by_fields"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<Value>| {
            let result = warp_endpoints::search_by_fields(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let get_items_with_edges = api_defaults
        .and(warp::path!(String / "get_items_with_edges"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<Vec<i64>>| {
            let result = warp_endpoints::get_items_with_edges(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let run_downloaders = api_defaults
        // //! In fact, any type that implements `FromStr` can be used, in any order:
        // ~/.cargo/registry.cache/src/github.com-1ecc6299db9ec823/warp-0.2.4/src/filters/path.rs:45
        .and(warp::path!(String / "run_downloader"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<RunDownloader>| {
            let result = warp_endpoints::run_downloader(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let run_importers = api_defaults
        .and(warp::path!(String / "run_importer"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<RunImporter>| {
            let result = warp_endpoints::run_importer(owner, init_db.deref(), body);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let init_db = initialized_databases_arc.clone();
    let run_indexers = api_defaults
        .and(warp::path!(String / "run_indexer"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<RunIndexer>| {
            let result = warp_endpoints::run_indexer(owner, init_db.deref(), body);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let origin_request =
        warp::options()
            .and(warp::header::<String>("origin"))
            .map(move |_origin| {
                let builder = http::response::Response::builder()
                    .status(StatusCode::OK)
                    .header("access-control-allow-methods", "HEAD, GET, POST, PUT")
                    .header(
                        "access-control-allow-headers",
                        "Origin, X-Requested-With, Content-Type, Accept",
                    )
                    .header("access-control-allow-credentials", "true")
                    .header("access-control-max-age", "300")
                    .header("access-control-allow-origin", "*");
                builder
                    .header("vary", "origin")
                    .body("".to_string())
                    .unwrap()
            });

    let main_filter = version
        .with(&headers)
        .or(get_item.with(&headers))
        .or(get_all_items.with(&headers))
        .or(create_item.with(&headers))
        .or(bulk_action.with(&headers))
        .or(update_item.with(&headers))
        .or(delete_item.with(&headers))
        .or(search.with(&headers))
        .or(get_items_with_edges.with(&headers))
        .or(run_downloaders.with(&headers))
        .or(run_importers.with(&headers))
        .or(run_indexers.with(&headers))
        .or(origin_request);

    warp::serve(main_filter).run(([0, 0, 0, 0], 3030)).await
}

fn respond_with_result<T: Reply>(result: crate::error::Result<T>) -> Response {
    match result {
        Err(err) => {
            let code = err.code.as_str();
            let code_canon = err.code.canonical_reason().unwrap_or("");
            let msg = &err.msg;
            info!("Returning HTTP failure {} {}: {}", code, code_canon, msg);
            warp::reply::with_status(err.msg, err.code).into_response()
        }
        Ok(t) => t.into_response(),
    }
}
