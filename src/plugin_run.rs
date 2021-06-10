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

/// Run a plugin, making sure that the correct ENV variables and settings are passed
/// to the containerization / deployment processes.
///
/// Internally passes to docker / kubernetes / scripts
/// depending on how Pod is configured by the user.
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
    let auth = database_key.create_plugin_auth()?;
    let auth = serde_json::to_string(&auth)?;
    if cli_options.use_kubernetes {
        run_kubernetes_container(
            &container,
            &item,
            pod_owner,
            &auth,
            triggered_by_item_id,
            cli_options,
        )
    } else {
        run_docker_container(
            &container,
            &item,
            pod_owner,
            &auth,
            triggered_by_item_id,
            cli_options,
        )
    }
}

/// Example:
/// docker run \
///     --network=host \
///     --env=POD_FULL_ADDRESS="http://localhost:3030" \
///     --env=POD_TARGET_ITEM="{...json...}" \
///     --env=POD_OWNER="...64-hex-chars..." \
///     --env=POD_AUTH_JSON="{...json...}" \
///     --name="$container-$trigger_item_id" \
///     --rm \
///     -- \
///     "$container"
fn run_docker_container(
    container: &str,
    target_item: &str,
    pod_owner: &str,
    pod_auth: &str,
    triggered_by_item_id: &str,
    cli_options: &CliOptions,
) -> Result<()> {
    let docker_network = match &cli_options.plugins_docker_network {
        Some(net) => net.to_string(),
        None => "host".to_string(),
    };
    let mut args: Vec<String> = Vec::with_capacity(10);
    args.push("run".to_string());
    args.push(format!("--network={}", docker_network));
    args.push(format!(
        "--env=POD_FULL_ADDRESS={}",
        callback_address(cli_options)
    ));
    args.push(format!("--env=POD_TARGET_ITEM={}", target_item));
    args.push(format!("--env=POD_OWNER={}", pod_owner));
    args.push(format!("--env=POD_AUTH_JSON={}", pod_auth));
    args.push(format!("--name={}-{}", &container, triggered_by_item_id));
    args.push("--rm".to_string());
    args.push("--".to_string());
    args.push(container.to_string());
    run_any_command("docker", &args, triggered_by_item_id)
}

/// Example:
/// kubectl run plugin
///     --image="$container" \
///     --env=POD_FULL_ADDRESS="http://localhost:3030" \
///     --env=POD_TARGET_ITEM="{...json...}" \
///     --env=POD_OWNER="...64-hex-chars..." \
///     --env=POD_AUTH_JSON="{...json...}" \
fn run_kubernetes_container(
    container: &str,
    target_item: &str,
    pod_owner: &str,
    pod_auth: &str,
    triggered_by_item_id: &str,
    cli_options: &CliOptions,
) -> Result<()> {
    let mut args: Vec<String> = Vec::with_capacity(7);
    args.push("run".to_string());
    args.push("plugin".to_string());
    args.push(format!("--image={}", container));
    args.push(format!(
        "--env=POD_FULL_ADDRESS={}",
        callback_address(cli_options)
    ));
    args.push(format!("--env=POD_TARGET_ITEM={}", target_item));
    args.push(format!("--env=POD_OWNER={}", pod_owner));
    args.push(format!("--env=POD_AUTH_JSON={}", pod_auth));
    run_any_command("kubectl", &args, triggered_by_item_id)
}

fn run_any_command(cmd: &str, args: &[String], container_id: &str) -> Result<()> {
    let debug_print = args
        .iter()
        .map(|p| escape_bash_arg(p))
        .collect::<Vec<_>>()
        .join(" ");
    log::info!("Starting command {} {}", cmd, debug_print);
    let command = Command::new(cmd).args(args).spawn();
    match command {
        Ok(_child) => {
            log::debug!(
                "Successfully started {} process for Plugin container item {}",
                cmd,
                container_id
            );
            Ok(())
        }
        Err(err) => Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!(
                "Failed to run plugin container triggered by item.rowid{}, {}",
                container_id, err
            ),
        }),
    }
}

fn callback_address(cli_options: &CliOptions) -> String {
    let is_https = cli_options.insecure_non_tls.is_none() && !cli_options.non_tls;
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
            format!("{}:{}", pod_domain, cli_options.port)
        }
    };
    let schema = if is_https { "https" } else { "http" };
    format!("{}://{}", schema, callback)
}

/// Debug-print a bash argument.
/// Never use this for running real code, but for debugging that's good enough.
///
/// From bash manual:
/// > Enclosing  characters in single quotes preserves the literal value of each character
/// > within the quotes. A single quote may not occur between single quotes,
/// > even when preceded by a backslash.
pub fn escape_bash_arg(str: &str) -> String {
    let ok = str.chars().all(|c| c.is_ascii_alphanumeric() || "_-=%".contains(c));
    if ok {
        str.to_string()
    } else {
        let quoted = str.replace("'", "'\\''"); // end quoting, append the literal, start quoting
        return format!("'{}'", quoted);
    }
}
