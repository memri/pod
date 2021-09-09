use crate::command_line_interface::CliOptions;
use crate::error::Error;
use crate::error::Result;
use crate::internal_api;
use crate::internal_api::new_random_string;
use crate::plugin_auth_crypto::DatabaseKey;
use crate::schema::Schema;
use log::info;
use rusqlite::Transaction;
use std::collections::HashMap;
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
    container_image: String,
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
    let target_item = internal_api::get_item_tx(tx, schema, target_item_id)?;
    let target_item = target_item.into_iter().next().ok_or_else(|| Error {
        code: StatusCode::BAD_REQUEST,
        msg: format!(
            "Failed to find target item {} to run a plugin against",
            target_item_id
        ),
    })?;
    let target_item_json = serde_json::to_string(&target_item)?;
    let auth = database_key.create_plugin_auth()?;
    let auth = serde_json::to_string(&auth)?;

    let script_override = cli_options
        .insecure_plugin_script
        .iter()
        .find(|(image, _script)| image == &container_image)
        .map(|(_image, script)| script);

    if let Some(script_path) = script_override {
        run_local_script(
            &container_image,
            script_path,
            &target_item_json,
            pod_owner,
            &auth,
            triggered_by_item_id,
            cli_options,
        )
    } else if cli_options.use_kubernetes {
        run_kubernetes_container(
            &container_image,
            &target_item_json,
            pod_owner,
            &auth,
            triggered_by_item_id,
            cli_options,
        )
    } else {
        run_docker_container(
            &container_image,
            &target_item_json,
            pod_owner,
            &auth,
            triggered_by_item_id,
            cli_options,
        )
    }
}

fn run_local_script(
    _container: &str,
    plugin_path: &str,
    target_item: &str,
    pod_owner: &str,
    pod_auth: &str,
    triggered_by_item_id: &str,
    cli_options: &CliOptions,
) -> Result<()> {
    let pod_full_address = callback_address(cli_options, false);
    let args: Vec<String> = Vec::new();
    let mut env_vars = HashMap::new();
    env_vars.insert("POD_FULL_ADDRESS", pod_full_address.as_str());
    env_vars.insert("POD_TARGET_ITEM", target_item);
    env_vars.insert("POD_PLUGINRUN_ID", triggered_by_item_id);
    env_vars.insert("POD_OWNER", pod_owner);
    env_vars.insert("POD_AUTH_JSON", pod_auth);
    run_any_command(plugin_path, &args, &env_vars, triggered_by_item_id)
}

/// Example:
/// docker run \
///     --network=host \
///     --env=POD_FULL_ADDRESS="http://localhost:3030" \
///     --env=POD_TARGET_ITEM="{...json...}" \
///     --env=POD_OWNER="...64-hex-chars..." \
///     --env=POD_AUTH_JSON="{...json...}" \
///     --name="$containerImage-$trigger_item_id" \
///     --rm \
///     -- \
///     "$containerImage"
fn run_docker_container(
    container_image: &str,
    target_item_json: &str,
    pod_owner: &str,
    pod_auth: &str,
    triggered_by_item_id: &str,
    cli_options: &CliOptions,
) -> Result<()> {
    let docker_network = match &cli_options.plugins_docker_network {
        Some(net) => net.to_string(),
        None => "host".to_string(),
    };
    let container_id = format!(
        "{}-{}-{}",
        pod_owner
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>(),
        container_image
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>(),
        triggered_by_item_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
    );
    let mut args: Vec<String> = Vec::with_capacity(10);
    args.push("run".to_string());
    args.push(format!("--network={}", docker_network));
    args.push(format!(
        "--env=POD_FULL_ADDRESS={}",
        callback_address(cli_options, true)
    ));
    args.push(format!("--env=POD_TARGET_ITEM={}", target_item_json));
    args.push(format!("--env=POD_PLUGINRUN_ID={}", triggered_by_item_id));
    args.push(format!("--env=POD_OWNER={}", pod_owner));
    args.push(format!("--env=POD_AUTH_JSON={}", pod_auth));
    args.push(format!("--name={}", sanitize_docker_name(&container_id)));
    args.push("--rm".to_string());
    args.push("--".to_string());
    args.push(container_image.to_string());
    let envs: HashMap<&str, &str> = HashMap::new();
    run_any_command("docker", &args, &envs, triggered_by_item_id)
}

/// Example:
/// kubectl run $owner-$containerImage-$targetItem-$randomHex
///     --image="$containerImage" \
///     --env=POD_FULL_ADDRESS="http://localhost:3030" \
///     --env=POD_TARGET_ITEM="{...json...}" \
///     --env=POD_OWNER="...64-hex-chars..." \
///     --env=POD_AUTH_JSON="{...json...}" \
fn run_kubernetes_container(
    container_image: &str,
    target_item_json: &str,
    pod_owner: &str,
    pod_auth: &str,
    triggered_by_item_id: &str,
    cli_options: &CliOptions,
) -> Result<()> {
    let container_id = format!(
        "c{}-{}-{}-{}",
        pod_owner
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(10)
            .collect::<String>(),
        container_image
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(20)
            .collect::<String>(),
        triggered_by_item_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(20)
            .collect::<String>(),
        new_random_string(8)
    );
    let mut args: Vec<String> = Vec::with_capacity(11);
    let s = container_id.clone();
    args.push("run".to_string());
    args.push("--restart=Never".to_string());
    args.push(s.clone());
    args.push(format!("--labels=app={},type=plugin", s.clone()));
    args.push("--port=8080".to_string());
    args.push("--image-pull-policy=Always".to_string());
    args.push(format!("--image={}", container_image));
    args.push(format!(
        "--env=POD_FULL_ADDRESS={}",
        callback_address(cli_options, false)
    ));
    args.push(format!("--env=POD_TARGET_ITEM={}", target_item_json));
    args.push(format!("--env=POD_PLUGINRUN_ID={}", triggered_by_item_id));
    args.push(format!("--env=POD_OWNER={}", pod_owner));
    args.push(format!("--env=POD_AUTH_JSON={}", pod_auth));
    args.push(format!(
        "--env=PLUGIN_DNS=http://{}.dev.pod.memri.io",
        s.clone()
    ));
    let envs: HashMap<&str, &str> = HashMap::new();
    run_any_command("kubectl", &args, &envs, triggered_by_item_id)
}

fn run_any_command(
    cmd: &str,
    args: &[String],
    envs: &HashMap<&str, &str>,
    container_id: &str,
) -> Result<()> {
    let debug_envs = envs
        .iter()
        .map(|(a, b)| format!("{}={} ", escape_bash_arg(a), escape_bash_arg(b)))
        .collect::<Vec<_>>()
        .join("");
    let debug_args = args
        .iter()
        .map(|p| escape_bash_arg(p))
        .collect::<Vec<_>>()
        .join(" ");
    log::info!("Starting command {}{} {}", debug_envs, cmd, debug_args);
    let command = Command::new(cmd).args(args).envs(envs).spawn();
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
                "Failed to execute {}, a plugin container triggered by item.rowid {}, {}",
                cmd, container_id, err
            ),
        }),
    }
}

/// Determine the callback address to use for plugins.
/// A callback address is an environment variable passed to the Plugin
/// so that the plugin can call back Pod when it needs to.
fn callback_address(cli_options: &CliOptions, docker_magic: bool) -> String {
    if let Some(address_override) = &cli_options.plugins_callback_address {
        address_override.to_string()
    } else {
        // The plugin container needs to have access to the host
        // This is currently done differently on MacOS and Linux
        // https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach
        let pod_domain = if docker_magic && !cfg!(target_os = "linux") {
            "host.docker.internal"
        } else {
            "localhost"
        };
        let is_https = cli_options.insecure_non_tls.is_none() && !cli_options.non_tls;
        let schema = if is_https { "https" } else { "http" };
        format!("{}://{}:{}", schema, pod_domain, cli_options.port)
    }
}

pub fn sanitize_docker_name(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Debug-print a bash argument.
/// Never use this for running real code, but for debugging that's good enough.
///
/// From bash manual:
/// > Enclosing  characters in single quotes preserves the literal value of each character
/// > within the quotes. A single quote may not occur between single quotes,
/// > even when preceded by a backslash.
pub fn escape_bash_arg(str: &str) -> String {
    let ok = str
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "_-+=%".contains(c));
    if ok {
        str.to_string()
    } else {
        let quoted = str.replace("'", "'\\''"); // end quoting, append the literal, start quoting
        return format!("'{}'", quoted);
    }
}
