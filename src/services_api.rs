// use crate::api_model::RunImporter;
// use crate::api_model::RunIntegratorItem;
// use crate::command_line_interface::CLIOptions;
// use crate::error::Error;
// use crate::error::Result;
// use crate::internal_api;
// use log::info;
// use rusqlite::Transaction;
// use std::process::Command;
// use warp::http::status::StatusCode;
//
// pub fn run_importer(
//     tx: &Transaction,
//     payload: RunImporter,
//     cli_options: &CLIOptions,
// ) -> Result<()> {
//     info!("Trying to run importer on item {}", payload.id);
//     let item = internal_api::get_item_tx(tx, &payload.id)?;
//     let item = item.into_iter().next().ok_or_else(|| Error {
//         code: StatusCode::BAD_REQUEST,
//         msg: format!("Failed to get item {}", payload.id),
//     })?;
//     let item: RunIntegratorItem = serde_json::from_value(item)?;
//     let mut args: Vec<String> = Vec::new();
//     args.push("run".to_string());
//     for arg in docker_arguments(cli_options) {
//         args.push(arg);
//     }
//     args.push(format!(
//         "--env=POD_SERVICE_PAYLOAD={}",
//         payload.service_payload
//     ));
//     args.push("--rm".to_string());
//     args.push("--name=memri-importers_1".to_string());
//     args.push(format!("--env=RUN_UID={}", payload.id));
//     args.push("--volume=memri-integrators-volume:/usr/src/importers/data".to_string());
//     args.push(item.repository);
//     log::info!("Starting importer docker command {:?}", args);
//     let command = Command::new("docker").args(&args).spawn();
//     match command {
//         Ok(_child) => {
//             log::debug!("Successfully started importer for {}", payload.id);
//             Ok(())
//         }
//         Err(err) => Err(Error {
//             code: StatusCode::INTERNAL_SERVER_ERROR,
//             msg: format!("Failed to run importer with id {}, {}", payload.id, err),
//         }),
//     }
// }
//
// fn docker_arguments(cli_options: &CLIOptions) -> Vec<String> {
//     let is_https = cli_options.insecure_non_tls.is_none() && !cli_options.non_tls;
//     let schema = if is_https { "https" } else { "http" };
//     let port: u16 = cli_options.port;
//     let network: &str = match &cli_options.services_docker_network {
//         Some(net) => net,
//         None => "host",
//     };
//     let callback = match &cli_options.services_callback_address {
//         Some(addr) => addr.to_string(),
//         None => {
//             // The indexers/importers/downloaders need to have access to the host
//             // This is currently done differently on MacOS and Linux
//             // https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach
//             let pod_domain = if cfg!(target_os = "linux") {
//                 "localhost"
//             } else {
//                 "host.docker.internal"
//             };
//             format!("{}:{}", pod_domain, port)
//         }
//     };
//     vec![
//         format!("--network={}", network),
//         format!("--env=POD_FULL_ADDRESS={}://{}", schema, callback),
//     ]
// }
