use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::api_model::RunService;
use crate::command_line_interface::CLIOptions;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use log::info;
use rusqlite::Connection;
use serde_json::Value;
use std::env;
use std::ops::Deref;
use std::process::Command;
use warp::http::status::StatusCode;

pub fn run_downloader(
    conn: &Connection,
    payload: RunDownloader,
    cli_options: &CLIOptions,
) -> Result<()> {
    info!("Trying to run downloader on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_none() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        });
    };
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    for arg in docker_arguments(cli_options) {
        args.push(arg);
    }
    args.push(format!(
        "--env=POD_SERVICE_PAYLOAD={}",
        payload.service_payload
    ));
    args.push("--rm".to_string());
    args.push("--name=memri-downloaders_1".to_string());
    args.push(format!("--env=RUN_UID={}", payload.uid));
    args.push("--volume=download-volume:/usr/src/importers/data".to_string());
    args.push("memri-downloaders:latest".to_string());
    log::debug!("Starting downloader docker command {:?}", args);
    let command = Command::new("docker").args(&args).spawn();
    match command {
        Ok(_child) => {
            log::debug!("Successfully started downloader for {}", payload.uid);
            Ok(())
        }
        Err(err) => {
            log::warn!("Failed to run downloader {}", payload.uid);
            Err(Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("Failed to run downloader with uid {}, {}", payload.uid, err),
            })
        }
    }
}

pub fn run_importer(
    conn: &Connection,
    payload: RunImporter,
    cli_options: &CLIOptions,
) -> Result<()> {
    info!("Trying to run importer on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_none() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        });
    };

    let path = env::current_dir()?;
    let parent = path.parent().expect("Failed to get parent directory");
    let whatsapp_volume = format!(
        "--volume={}/importers/data-synapse:/usr/src/importers/data-synapse",
        parent.display().to_string()
    );
    let docker_image = result
        .first()
        .expect("Failed to get ImporterRun item")
        .as_object()
        .expect("Failed to get value")
        .get("repository")
        .expect("Failed to get repository for docker image")
        .as_str()
        .expect("Failed to get string");
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    for arg in docker_arguments(cli_options) {
        args.push(arg);
    }
    args.push(format!(
        "--env=POD_SERVICE_PAYLOAD={}",
        payload.service_payload
    ));
    args.push("--rm".to_string());
    args.push("--name=memri-importers_1".to_string());
    args.push(format!("--env=RUN_UID={}", payload.uid));
    args.push(whatsapp_volume);
    args.push(format!("{}:latest", docker_image));
    log::debug!("Starting importer docker command {:?}", args);
    let command = Command::new("docker").args(&args).spawn();
    match command {
        Ok(_child) => {
            log::debug!("Successfully started importer for {}", payload.uid);
            Ok(())
        }
        Err(err) => Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to run importer with uid {}, {}", payload.uid, err),
        }),
    }
}

pub fn run_indexers(
    conn: &Connection,
    payload: RunIndexer,
    cli_options: &CLIOptions,
) -> Result<()> {
    info!("Trying to run indexer on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_none() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        });
    };
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    for arg in docker_arguments(cli_options) {
        args.push(arg);
    }
    args.push(format!(
        "--env=POD_SERVICE_PAYLOAD={}",
        payload.service_payload
    ));
    args.push("--rm".to_string());
    args.push("--name=memri-indexers_1".to_string());
    args.push(format!("--env=RUN_UID={}", payload.uid));
    args.push("memri-indexers:latest".to_string());
    log::debug!("Starting indexer docker command {:?}", args);
    let command = Command::new("docker").args(&args).spawn();
    match command {
        Ok(_child) => {
            log::debug!("Successfully started indexer for {}", payload.uid);
            Ok(())
        }
        Err(err) => Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to run indexer with uid {}, {}", payload.uid, err),
        }),
    }
}

pub fn run_services(conn: &Connection, payload: RunService) -> Result<Value> {
    info!("Trying to run service on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_none() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        });
    };
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    args.push("--rm".to_string());
    let content = payload.service_payload.to_string();
    println!("{:#?}", content.trim_matches('\\'));
    args.push(format!(
        "--env=POD_SERVICE_PAYLOAD='{}'",
        content.trim_matches('\\')
    ));
    let service = result
        .first()
        .expect("Failed to get value")
        .as_object()
        .expect("Failed to get map")
        .get("repository")
        .expect("Failed to get service")
        .as_str()
        .expect("Failed to get string");
    args.push(format!("--name={}_1", service));
    args.push(format!("{}:latest", service));
    log::debug!("Starting service docker command {:?}", args);
    let output = Command::new("docker").args(&args).output()?;
    if output.status.success() {
        log::debug!("Successfully started service for {}", payload.uid);
        let output = Value::from(String::from_utf8(output.stdout)?);
        Ok(output)
    } else {
        Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!(
                "Failed to run service with uid {}, {}",
                payload.uid,
                String::from_utf8(output.stderr)?
            ),
        })
    }
}

fn docker_arguments(cli_options: &CLIOptions) -> Vec<String> {
    let is_https = cli_options.insecure_non_tls.is_none() && !cli_options.non_tls;
    let schema = if is_https { "https" } else { "http" };
    let port: u16 = cli_options.port;
    let network: &str = match &cli_options.services_docker_network {
        Some(net) => net,
        None => "host",
    };
    let callback = match &cli_options.services_callback_address {
        Some(addr) => addr.to_string(),
        None => {
            // The indexers/importers/downloaders need to have access to the host
            // This is currently done differently on MacOS and Linux
            // https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach
            let pod_domain = if cfg!(target_os = "linux") {
                "localhost"
            } else {
                "host.docker.internal"
            };
            format!("{}:{}", pod_domain, port)
        }
    };
    vec![
        format!("--network={}", network),
        format!("--env=POD_FULL_ADDRESS={}://{}", schema, callback),
    ]
}
