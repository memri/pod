use crate::api_model::BulkAction;
use crate::api_model::CreateItem;
use crate::api_model::GetFile;
use crate::api_model::PayloadWrapper;
// use crate::api_model::RunImporter;
use crate::api_model::Search;
use crate::api_model::UpdateItem;
use crate::command_line_interface;
use crate::command_line_interface::CLIOptions;
use crate::internal_api;
use crate::warp_endpoints;
use log::error;
use log::info;
use log::warn;
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
use warp::hyper::body::Bytes;
use warp::reply::Response;
use warp::Filter;
use warp::Reply;

/// Start web framework with specified APIs.
pub async fn run_server(cli_options: &CLIOptions) {
    let package_name = env!("CARGO_PKG_NAME").to_uppercase();
    info!("Starting {} HTTP server", package_name);

    let mut headers = HeaderMap::new();
    if cli_options.insecure_http_headers {
        info!("Adding insecure http headers Access-Control-Allow-Origin header as per CLI option");
        headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }
    let headers = warp::reply::with::headers(headers);

    let items_api = warp::path("v3")
        .and(warp::body::content_length_limit(5 * 1024 * 1024))
        .and(warp::post());
    // let services_api = warp::path("v3")
    //     .and(warp::body::content_length_limit(32 * 1024))
    //     .and(warp::post());
    let file_api = warp::path("v3")
        .and(warp::body::content_length_limit(500 * 1024 * 1024))
        .and(warp::post());

    let initialized_databases_arc = Arc::new(RwLock::new(HashSet::<String>::new()));

    let version = warp::path("version")
        .and(warp::path::end())
        .and(warp::get())
        .map(internal_api::get_project_version);

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
    let get_item = items_api
        .and(warp::path!(String / "get_item"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<String>| {
            let result = warp_endpoints::get_item(owner, init_db.deref(), body);
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
        .map(move |owner: String, body: PayloadWrapper<String>| {
            let result = warp_endpoints::delete_item(owner, init_db.deref(), body);
            let result = result.map(|()| warp::reply::json(&serde_json::json!({})));
            respond_with_result(result)
        });

    let init_db = initialized_databases_arc.clone();
    let search_by_fields = items_api
        .and(warp::path!(String / "search"))
        .and(warp::path::end())
        .and(warp::body::json())
        .map(move |owner: String, body: PayloadWrapper<Search>| {
            let result = warp_endpoints::search(owner, init_db.deref(), body);
            let result = result.map(|result| warp::reply::json(&result));
            respond_with_result(result)
        });

    // let init_db = initialized_databases_arc.clone();
    // let cli_options_arc = Arc::new(cli_options.clone());
    // let run_importer = services_api
    //     .and(warp::path!(String / "run_importer"))
    //     .and(warp::path::end())
    //     .and(warp::body::json())
    //     .map(move |owner: String, body: PayloadWrapper<RunImporter>| {
    //         let cli: &CLIOptions = &cli_options_arc.deref();
    //         let result = warp_endpoints::run_importer(owner, init_db.deref(), body, cli);
    //         respond_with_result(result.map(|()| warp::reply::json(&serde_json::json!({}))))
    //     });

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
            respond_with_result(result)
        });

    let insecure_http_headers = Arc::new(cli_options.insecure_http_headers);
    let origin_request =
        warp::options()
            .and(warp::header::<String>("origin"))
            .map(move |_origin| {
                if *insecure_http_headers {
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
                } else {
                    http::Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(String::new())
                        .expect("Failed to return an empty body")
                }
            });

    let always_enabled_filters = version.with(&headers).or(create_item.with(&headers));

    let sensitive_filters = warp::any().and_then(|| async move {
        if command_line_interface::PARSED.shared_server {
            Ok(warp::reply::with_status("", StatusCode::NOT_FOUND).into_response())
        } else {
            Err(warp::reject::not_found()) // reject in order to pass to filters below
        }
    });
    let sensitive_filters = sensitive_filters
        .or(get_item.with(&headers))
        .or(bulk_action.with(&headers))
        .or(update_item.with(&headers))
        .or(delete_item.with(&headers))
        .or(search_by_fields.with(&headers))
        // .or(run_importer.with(&headers))
        .or(upload_file.with(&headers))
        .or(get_file.with(&headers))
        .or(origin_request);

    let not_found = warp::any().map(|| {
        warp::reply::with_status("Endpoint not found", StatusCode::NOT_FOUND).into_response()
    });

    let filters = always_enabled_filters.or(sensitive_filters).or(not_found);

    if cli_options.non_tls || cli_options.insecure_non_tls.is_some() {
        let ip = if let Some(ip) = cli_options.insecure_non_tls {
            if ip.is_loopback() {
                log::info!(
                    "You seem to use --insecure-non-tls option with a loopback address. \
                    It is recommended to instead use --non-tls CLI option \
                    for clarity and simplicity."
                );
            } else if check_public_ip(&ip) {
                warn!(
                    "The server is asked to run on a public IP {} without https encryption. \
                    This is discouraged as an intermediary (even your router on a local network) \
                    could spoof the traffic sent to the server and do a MiTM attack.\
                    Please consider using Pod with https encryption, \
                    or run it on a non-public network.",
                    ip
                );
            };
            ip
        } else {
            IpAddr::from([127, 0, 0, 1])
        };
        let socket = SocketAddr::new(ip, cli_options.port);
        warp::serve(filters).run(socket).await
    } else {
        let cert_path = &cli_options.tls_pub_crt;
        let key_path = &cli_options.tls_priv_key;
        if PathBuf::from_str(&cert_path)
            .map(|p| !p.exists())
            .unwrap_or(true)
        {
            error!("Certificate public key {} not found", cert_path);
            std::process::exit(1)
        };
        if PathBuf::from_str(&key_path)
            .map(|p| !p.exists())
            .unwrap_or(true)
        {
            error!("Certificate private key {} not found", cert_path);
            std::process::exit(1)
        };
        let socket = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), cli_options.port);
        warp::serve(filters)
            .tls()
            .cert_path(cert_path)
            .key_path(key_path)
            .run(socket)
            .await;
    }
}

fn respond_with_result<T: Reply>(result: crate::error::Result<T>) -> Response {
    match result {
        Err(err) => {
            let code = err.code.as_str();
            let code_canon = err.code.canonical_reason().unwrap_or("");
            info!(
                "Returning HTTP failure {} {}: {}",
                code, code_canon, &err.msg
            );
            let msg = format!("Failure: {}", err.msg);
            warp::reply::with_status(msg, err.code).into_response()
        }
        Ok(t) => t.into_response(),
    }
}

fn check_public_ip(addr: &IpAddr) -> bool {
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
