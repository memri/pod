use crate::internal_api;
use bytes::Bytes;
use log::info;
use log::warn;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use warp::http::header::HeaderMap;
use warp::http::header::HeaderValue;
use warp::http::status::StatusCode;
use warp::reply::Response;
use warp::Filter;
use warp::Reply;

/// Start web framework with specified APIs.
pub async fn run_server(sqlite_pool: Pool<SqliteConnectionManager>) {
    let package_name = env!("CARGO_PKG_NAME").to_uppercase();
    info!("Starting {} HTTP server", package_name);

    let insecure_http_headers = std::env::var_os("INSECURE_HTTP_HEADERS").is_some();

    let mut headers = HeaderMap::new();
    if insecure_http_headers {
        warn!("Adding Access-Control-Allow-Origin header as per environment config");
        headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }
    let headers = warp::reply::with::headers(headers);

    let api_version_1 = warp::path("v1");

    let pool_arc = Arc::new(sqlite_pool);

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);

    let pool = pool_arc.clone();
    let get_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |uid: i64| {
            let result = internal_api::get_item(&pool, uid);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let get_all_items = api_version_1
        .and(warp::path!("all_items"))
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
            let result = internal_api::get_all_items(&pool);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let create_item = api_version_1
        .and(warp::path("items"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let result = internal_api::create_item(&pool, body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let update_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .map(move |uid: i64, body: serde_json::Value| {
            let result = internal_api::update_item(&pool, uid, body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let bulk_action = api_version_1
        .and(warp::path!("bulk_action"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: serde_json::Value| {
            let result = internal_api::bulk_action(&pool, body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let delete_item = api_version_1
        .and(warp::path!("items" / i64))
        .and(warp::path::end())
        .and(warp::delete())
        .map(move |uid: i64| {
            let result = internal_api::delete_item(&pool, uid);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let external_id_exists = api_version_1
        .and(warp::path!("deprecated" / "uri_exists" / String))
        .and(warp::path::end())
        .map(move |external_id: String| {
            let body = serde_json::json!({ "uri": external_id });
            let result = internal_api::search_by_fields(&pool, body);
            let result = result.map(|result| warp::reply::json(&!result.is_empty()));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let search = api_version_1
        .and(warp::path("search_by_fields"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::bytes())
        .map(move |body: Bytes| {
            let body =
                serde_json::from_slice(&body).expect("Failed to serialize request body to JSON");
            let result = internal_api::search_by_fields(&pool, body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let pool = pool_arc.clone();
    let get_item_with_edges = api_version_1
        .and(warp::path!("item_with_edges" / i64))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |uid: i64| {
            let result = internal_api::get_item_with_edges(&pool, uid);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let run_downloaders = api_version_1
        .and(warp::path!("run_service" / "downloaders" / String / String))
        .and(warp::path::end())
        .and(warp::post())
        .map(move |service: String, data_type: String| {
            let result = internal_api::run_downloaders(service, data_type);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let run_importers = api_version_1
        .and(warp::path!("run_service" / "importers" / String))
        .and(warp::path::end())
        .and(warp::post())
        .map(move |data_type: String| {
            let result = internal_api::run_importers(data_type);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let pool = pool_arc.clone();
    let run_indexers = api_version_1
        .and(warp::path!("run_service" / "indexers" / i64))
        .and(warp::path::end())
        .and(warp::post())
        .map(move |uid: i64| {
            let result = internal_api::run_indexers(&pool, uid);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let main_filter = version
        .with(&headers)
        .or(get_item.with(&headers))
        .or(get_all_items.with(&headers))
        .or(create_item.with(&headers))
        .or(bulk_action.with(&headers))
        .or(update_item.with(&headers))
        .or(delete_item.with(&headers))
        .or(external_id_exists.with(&headers))
        .or(search.with(&headers))
        .or(get_item_with_edges.with(&headers))
        .or(run_downloaders.with(&headers))
        .or(run_importers.with(&headers))
        .or(run_indexers.with(&headers));

    if insecure_http_headers {
        let origin = warp::options()
            .and(warp::header::<String>("origin"))
            .map(|origin| {
                let builder = warp::http::response::Response::builder()
                    .status(StatusCode::OK)
                    .header("access-control-allow-methods", "HEAD, GET, POST, PUT")
                    .header("access-control-allow-headers", "authorization")
                    .header("access-control-allow-credentials", "true")
                    .header("access-control-max-age", "300")
                    .header("access-control-allow-origin", origin);
                builder
                    .header("vary", "origin")
                    .body("".to_string())
                    .unwrap()
            });
        warp::serve(main_filter.or(origin))
            .run(([0, 0, 0, 0], 3030))
            .await;
    } else {
        warp::serve(main_filter).run(([0, 0, 0, 0], 3030)).await;
    };
}

fn respond_with_result<T: Reply>(result: crate::error::Result<T>) -> Response {
    match result {
        Err(err) => {
            let code = err.code.as_str();
            let code_canon = err.code.canonical_reason().unwrap_or("");
            let msg = &err.msg;
            info!("Returning HTTP failure {} {}, {}", code, code_canon, msg);
            warp::reply::with_status(err.msg, err.code).into_response()
        }
        Ok(t) => t.into_response(),
    }
}
