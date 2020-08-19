use crate::api_model::Action;
use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::GetFile;
use crate::api_model::PayloadWrapper;
use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::api_model::UpdateItem;
use crate::configuration;
use crate::internal_api;
use crate::warp_endpoints;
use bytes::Bytes;
use log::error;
use log::info;
use log::warn;
use serde_json::Value;
use std::collections::HashSet;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
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

    let items_api = warp::path("v2")
        .and(warp::body::content_length_limit(5 * 1024 * 1024))
        .and(warp::post());
    let services_api = warp::path("v2")
        .and(warp::body::content_length_limit(32 * 1024))
        .and(warp::post());
    let file_api = warp::path("v2")
        .and(warp::body::content_length_limit(500 * 1024 * 1024))
        .and(warp::post());
    let action_api = warp::path("v2")
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::post());

    let initialized_databases_arc = Arc::new(RwLock::new(HashSet::<String>::new()));

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);

    let init_db = initialized_databases_arc.clone();
    let get_item = items_api
        .and(warp::path!(String / "get_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<i64>| {
            let result = warp_endpoints::get_item(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let get_all_items = items_api
        .and(warp::path!(String / "get_all_items"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<()>| {
            let result = warp_endpoints::get_all_items(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let create_item = items_api
        .and(warp::path!(String / "create_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<CreateItem>| {
            let result = warp_endpoints::create_item(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let update_item = items_api
        .and(warp::path!(String / "update_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<UpdateItem>| {
            let result = warp_endpoints::update_item(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let bulk_action = items_api
        .and(warp::path!(String / "bulk_action"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<BulkAction>| {
            let result = warp_endpoints::bulk_action(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let delete_item = items_api
        .and(warp::path!(String / "delete_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<i64>| {
            let result = warp_endpoints::delete_item(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let search = items_api
        .and(warp::path!(String / "search_by_fields"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<Value>| {
            let result = warp_endpoints::search_by_fields(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let get_items_with_edges = items_api
        .and(warp::path!(String / "get_items_with_edges"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<Vec<i64>>| {
            let result = warp_endpoints::get_items_with_edges(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let run_downloader = services_api
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
    let run_importer = services_api
        .and(warp::path!(String / "run_importer"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<RunImporter>| {
            let result = warp_endpoints::run_importer(owner, init_db.deref(), body);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let init_db = initialized_databases_arc.clone();
    let run_indexer = services_api
        .and(warp::path!(String / "run_indexer"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<RunIndexer>| {
            let result = warp_endpoints::run_indexer(owner, init_db.deref(), body);
            respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
        });

    let init_db = initialized_databases_arc.clone();
    let upload_file = file_api
        .and(warp::path!(String / "upload_file" / String / String))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .map(
            move |owner: String, database_key: String, expected_sha256: String, body: Bytes| {
                let result = warp_endpoints::upload_file(
                    owner,
                    init_db.deref(),
                    database_key,
                    expected_sha256,
                    &body,
                );
                respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
            },
        );

    let init_db = initialized_databases_arc.clone();
    let get_file = file_api
        .and(warp::path!(String / "get_file"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<GetFile>| {
            let result = warp_endpoints::get_file(owner, init_db.deref(), body);
            respond_with_result(result.map(|result| result))
        });

    let do_action = action_api
        .and(warp::path!(String / "do_action"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: Action| {
            let result = warp_endpoints::do_action(owner, body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
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
        .or(run_downloader.with(&headers))
        .or(run_importer.with(&headers))
        .or(run_indexer.with(&headers))
        .or(upload_file.with(&headers))
        .or(get_file.with(&headers))
        .or(do_action.with(&headers))
        .or(origin_request);

    if let Some(cert) = configuration::https_certificate_file() {
        let addr = configuration::pod_listen_address()
            .unwrap_or_else(|| format!("0.0.0.0:{}", configuration::DEFAULT_PORT));
        let addr = SocketAddr::from_str(&addr).unwrap_or_else(|err| {
            error!("Failed to parse desired hosting address {}, {}", addr, err);
            std::process::exit(1)
        });
        let cert_path = format!("{}.crt", cert);
        let key_path = format!("{}.key", cert);
        if !PathBuf::from_str(&cert_path)
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            error!("Certificate public key {} not found", cert_path);
            std::process::exit(1)
        }
        if !PathBuf::from_str(&key_path)
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            error!("Certificate private key {} not found", cert_path);
            std::process::exit(1)
        }
        warp::serve(main_filter)
            .tls()
            .cert_path(cert_path)
            .key_path(key_path)
            .run(addr)
            .await;
    } else {
        let addr = configuration::pod_listen_address()
            .unwrap_or_else(|| format!("127.0.0.1:{}", configuration::DEFAULT_PORT));
        let addr = SocketAddr::from_str(&addr).unwrap_or_else(|err| {
            error!("Failed to parse desired hosting address {}, {}", addr, err);
            std::process::exit(1);
        });
        let is_loopback = addr.ip().is_loopback();
        if !is_loopback {
            warn!(
                "Https certificate files not configured. It is best recommended to only \
                run Pod with encryption. To set up certificates once you obtained them, \
                set {} environment variable to the path \
                of the certificates (without .crt and .key suffixes)",
                configuration::HTTPS_CERTIFICATE_ENV_NAME
            );
        }
        if !is_loopback && check_public_ip(addr.ip()) {
            warn!(
                "The server is asked to run on a public IP {} without https encryption. \
                This is discouraged as an intermediary (even your router on a local network) \
                could spoof the traffic sent to the server and do a MiTM attack.\
                Please consider using Pod with https encryption, \
                or run it on a non-public network.",
                addr
            );
        };
        if !is_loopback && !configuration::use_insecure_non_tls() {
            error!(
                "Refusing to run pod without TLS (https). If you want to override this, \
                start pod with environment variable {} set to any value.",
                configuration::USE_INSECURE_NON_TLS_ENV_NAME
            );
            std::process::exit(1)
        }
        warp::serve(main_filter).run(addr).await
    }
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

fn check_public_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) if v6.is_loopback() => true,
        // https://en.wikipedia.org/wiki/Unique_local_address
        // Implementation copied from `v6.is_unique_local()`,
        // which is not yet stabilized in Rust
        IpAddr::V6(v6) if (v6.segments()[0] & 0xfe00) == 0xfc00 => true,
        // https://en.wikipedia.org/wiki/Link-local_address
        // Implementation copied from `v6.is_unicast_link_local()`,
        // which is not yet stabilized in Rust
        IpAddr::V6(v6) if (v6.segments()[0] & 0xffc0) == 0xfe80 => true,
        _ => false,
    }
}
