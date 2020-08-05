use crate::api_model::RunDownloader;
use crate::api_model::RunImporter;
use crate::api_model::RunIndexer;
use crate::configuration::pod_is_in_docker;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use log::info;
use rusqlite::Connection;
use std::env;
use std::ops::Deref;
use std::process::Command;
use warp::http::status::StatusCode;

pub fn run_downloader(conn: &Connection, payload: RunDownloader) -> Result<()> {
    info!("Trying to run downloader on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_some() {
        let command = Command::new("docker")
            .arg("run")
            .args(&docker_arguments())
            .arg(&format!(
                "--env=POD_SERVICE_PAYLOAD={}",
                payload.service_payload
            ))
            .args(&[
                "--rm",
                "--name=memri-downloaders_1",
                &format!("--env=RUN_UID={}", payload.uid),
                "--volume=download-volume:/usr/src/importers/data",
            ])
            .args(&["memri-downloaders:latest"])
            .spawn();
        match command {
            Ok(_child) => Ok(()),
            Err(err) => Err(Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("Failed to run importer with uid {}, {}", payload.uid, err),
            }),
        }
    } else {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item with uid={}", payload.uid),
        })
    }
}

pub fn run_importer(conn: &Connection, payload: RunImporter) -> Result<()> {
    info!("Trying to run importer on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_some() {
        let path = env::current_dir()?;
        let parent = path.parent().expect("Failed to get parent directory");
        let wa_volume = format!(
            "--volume={}/importers/data-mautrix:/usr/src/importers/data-mautrix",
            parent.display().to_string()
        );
        let command = Command::new("docker")
            .arg("run")
            .args(&docker_arguments())
            .arg(&format!(
                "--env=POD_SERVICE_PAYLOAD={}",
                payload.service_payload
            ))
            .args(&[
                "--rm",
                "--name=memri-importers_1",
                &format!("--env=RUN_UID={}", payload.uid),
                "--volume=download-volume:/usr/src/importers/data",
                &wa_volume,
            ])
            .args(&["memri-importers:latest"])
            .spawn();
        match command {
            Ok(_child) => Ok(()),
            Err(err) => Err(Error {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: format!("Failed to run importer with uid {}, {}", payload.uid, err),
            }),
        }
    } else {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        })
    }
}

pub fn run_indexers(conn: &Connection, payload: RunIndexer) -> Result<()> {
    info!("Trying to run indexer on item {}", payload.uid);
    let result = internal_api::get_item(conn.deref(), payload.uid)?;
    if result.first().is_some() {
        Command::new("docker")
            .arg("run")
            .args(&docker_arguments())
            .arg(&format!(
                "--env=POD_SERVICE_PAYLOAD={}",
                payload.service_payload
            ))
            .args(&[
                "--rm",
                "--name=memri-indexers_1",
                &format!("--env=RUN_UID={}", payload.uid),
            ])
            .args(&["memri-indexers:latest"])
            .spawn()
            .expect("Failed to run indexer");
        Ok(())
    } else {
        Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", payload.uid),
        })
    }
}

fn docker_arguments() -> Vec<String> {
    let is_https = crate::configuration::https_certificate_file().is_some();
    let schema = if is_https { "https" } else { "http" };
    if pod_is_in_docker() {
        vec![
            "--network=pod_memri-net".to_string(),
            "--env=POD_ADDRESS=pod_pod_1".to_string(),
            format!("--env=POD_FULL_ADDRESS={}://pod_pod_1", schema),
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
            format!("--env=POD_FULL_ADDRESS={}://{}", schema, pod_address),
            "--network=host".to_string(),
        ]
    }
}
