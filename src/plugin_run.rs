use crate::command_line_interface::CliOptions;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::schema::Schema;
use log::info;
use rusqlite::Transaction;
use std::process::Command;
use warp::http::status::StatusCode;

#[allow(clippy::too_many_arguments)]
pub fn run_plugin_container(
    tx: &Transaction,
    schema: &Schema,
    container: String,
    target_item_id: &str,
    triggered_by_item_id: &str,
    pod_owner: &str,
    database_key: &DatabaseKey,
    cli_options: &CliOptions,
) -> Result<()> {
    info!(
        "Trying to run plugin container for target_item_id {}",
        target_item_id
    );
    let item = internal_api::get_item_tx(tx, schema, target_item_id)?;
    let item = item.into_iter().next().ok_or_else(|| Error {
        code: StatusCode::BAD_REQUEST,
        msg: format!(
            "Failed to find target item {} to run a plugin against",
            target_item_id
        ),
    })?;
    let item = serde_json::to_string(&item)?;
    let mut args: Vec<String> = Vec::with_capacity(12);
    args.push("run".to_string());
    for arg in docker_arguments(cli_options) {
        args.push(arg);
    }
    args.push(format!("--env=POD_TARGET_ITEM={}", item));
    args.push(format!("--env=POD_OWNER={}", pod_owner));
    let auth = database_key.create_plugin_auth()?;
    let auth = serde_json::to_string(&auth)?;
    args.push(format!("--env=POD_AUTH_JSON={}", auth));
    args.push("--rm".to_string());
    args.push(format!("--name={}-{}", &container, triggered_by_item_id));
    args.push(container);
    log::info!("Starting plugin docker command {:?}", args);
    let command = Command::new("docker").args(&args).spawn();
    match command {
        Ok(_child) => {
            log::debug!(
                "Successfully started Plugin container for {}",
                triggered_by_item_id
            );
            Ok(())
        }
        Err(err) => Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!(
                "Failed to run plugin container triggered by item.rowid{}, {}",
                triggered_by_item_id, err
            ),
        }),
    }
}

fn docker_arguments(cli_options: &CliOptions) -> Vec<String> {
    let is_https = cli_options.insecure_non_tls.is_none() && !cli_options.non_tls;
    let schema = if is_https { "https" } else { "http" };
    let port: u16 = cli_options.port;
    let network: &str = match &cli_options.plugins_docker_network {
        Some(net) => net,
        None => "host",
    };
    let callback = match &cli_options.plugins_callback_address {
        Some(addr) => addr.to_string(),
        None => {
            // The plugin container needs to have access to the host
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
