use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::configuration::pod_is_in_docker;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use log::info;
use rusqlite::Connection;
use std::ops::Deref;
use std::process::Command;
use warp::http::status::StatusCode;

pub fn run_downloader(conn: &Connection, payload: RunDownloader) -> Result<()> {
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
    for arg in docker_arguments() {
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

pub fn run_importer(conn: &Connection, payload: RunImporter) -> Result<()> {
    info!("Trying to run importer on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_none() {
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        });
    };
    let mut args: Vec<String> = Vec::new();
    args.push("run".to_string());
    for arg in docker_arguments() {
        args.push(arg);
    }
    args.push(format!(
        "--env=POD_SERVICE_PAYLOAD={}",
        payload.service_payload
    ));
    args.push("--rm".to_string());
    args.push("--name=memri-importers_1".to_string());
    args.push(format!("--env=RUN_UID={}", payload.uid));
    args.push("--volume=download-volume:/usr/src/importers/data".to_string());
    args.push("memri-importers:latest".to_string());
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

pub fn run_indexers(conn: &Connection, payload: RunIndexer) -> Result<()> {
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
    for arg in docker_arguments() {
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

fn docker_arguments() -> Vec<String> {
    let is_https = crate::configuration::https_certificate_file().is_some();
    let schema = if is_https { "https" } else { "http" };
    let port = crate::configuration::DEFAULT_PORT;
    if pod_is_in_docker() {
        vec![
            "--network=pod_memri-net".to_string(),
            "--env=POD_ADDRESS=pod_pod_1".to_string(),
            format!("--env=POD_FULL_ADDRESS={}://pod_pod_1:{}", schema, port),
        ]
    } else {
        // The indexers/importers/downloaders need to have access to the host
        // This is currently done differently on MacOS and Linux
        // https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach
        let pod_address = if cfg!(target_os = "linux") {
            "localhost"
        } else {
            "host.docker.internal"
        };
        vec![
            format!("--env=POD_ADDRESS={}", pod_address),
            format!(
                "--env=POD_FULL_ADDRESS={}://{}:{}",
                schema, pod_address, port
            ),
            "--network=host".to_string(),
        ]
    }
}
