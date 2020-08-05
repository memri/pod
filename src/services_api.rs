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

pub fn run_downloader(payload: RunDownloader) -> Result<()> {
    let service = &payload.service;
    let data_type = &payload.data_type;
    info!("Trying to run downloader {} for {}", service, data_type);
    match service.as_str() {
        "evernote" => match data_type.as_str() {
            "note" => {
                Command::new("docker")
                    .arg("run")
                    .args(&docker_arguments())
                    .arg(&format!(
                        "--env=POD_SERVICE_PAYLOAD={}",
                        payload.service_payload
                    ))
                    .args(&["--rm", "--name=memri-indexers_1", "-it"])
                    .args(&["memri-downloaders:latest"])
                    .spawn()
                    .expect("Failed to run downloader");
            }
            _ => {
                return Err(Error {
                    code: StatusCode::BAD_REQUEST,
                    msg: format!("Data type {} not supported", data_type),
                })
            }
        },
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!("Service {} not supported", service),
            })
        }
    }
    Ok(())
}

pub fn run_importer(payload: RunImporter) -> Result<()> {
    let data_type = &payload.data_type;
    info!("Trying to run importer for {}", data_type);
    match data_type.as_str() {
        "note" => {
            Command::new("docker")
                .arg("run")
                .args(&docker_arguments())
                .arg(&format!(
                    "--env=POD_SERVICE_PAYLOAD={}",
                    payload.service_payload
                ))
                .args(&[
                    "--rm",
                    "--volume=download-volume:/usr/src/importers/data",
                    "--name=memri-importers_1",
                ])
                .args(&["memri-importers:latest"])
                .spawn()
                .expect("Failed to run importer");
        }
        "whatsapp" => {
            let path = env::current_dir()?;
            let parent = path.parent().expect("Failed to get parent directory");
            let wa_volume = format!(
                "--volume={}/importers/data-mautrix:/usr/src/importers/data-mautrix",
                parent.display().to_string()
            );
            Command::new("docker")
                .arg("run")
                .args(&docker_arguments())
                .arg(&format!(
                    "--env=POD_SERVICE_PAYLOAD={}",
                    payload.service_payload
                ))
                .args(&[
                    "--rm",
                    &wa_volume,
                    "--name=memri-importers_1",
                    "--env=IMPORTER_TYPE=whatsapp",
                ])
                .args(&["memri-importers:latest"])
                .spawn()
                .expect("Failed to run importer");
        }
        _ => {
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg: format!("Data type {} not supported", data_type),
            })
        }
    }
    Ok(())
}

pub fn run_indexers(conn: &Connection, payload: RunIndexer) -> Result<()> {
    let uid = payload.uid;
    info!("Trying to run indexer on item {}", uid);
    let result = internal_api::get_item(conn.deref(), uid)?;
    match result.first() {
        Some(_item) => {
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
                    &format!("--env=RUN_UID={}", uid),
                ])
                .args(&["memri-indexers:latest"])
                .spawn()
                .expect("Failed to run indexer");
            Ok(())
        }
        None => Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg: format!("Failed to get item {}", uid),
        }),
    }
}

fn docker_arguments() -> Vec<String> {
    if pod_is_in_docker() {
        vec![
            "--network=pod_memri-net".to_string(),
            "--env=POD_ADDRESS=pod_pod_1".to_string(),
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
            "--network=host".to_string(),
        ]
    }
}
