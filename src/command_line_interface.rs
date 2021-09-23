use lazy_static::lazy_static;
use std::error::Error;
use std::net::IpAddr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "Pod, the open-source backend for Memri project.",
    setting = AppSettings::DeriveDisplayOrder,
    setting = AppSettings::UnifiedHelpMessage,
    version = VERSION.as_ref(),
)]
pub struct CliOptions {
    /// Port to listen to.
    #[structopt(short, long, default_value = "3030")]
    pub port: u16,

    /// Comma-separated list of Pod owners (hex-encoded hashes of public keys).
    /// See `docs/HTTP_API.md#api-authentication-credentials` on the format of the owner keys.
    ///
    /// Only those owners are allowed to call Pod endpoints.
    ///
    /// Each Pod owner has its own database and files directory,
    /// the owners do not intersect data-wise.
    /// Pod does not store any data on owners in any external databases.
    ///
    /// A magic value of "ANY" will allow any owner to connect to the Pod.
    #[structopt(
        short = "o",
        long,
        name = "OWNERS",
        required = true,
        env = "POD_OWNER_HASHES"
    )]
    pub owners: String,

    /// If specified, all Plugin containers will be started using kubernetes (`kubectl`).
    /// Otherwise and by default, docker containers are used.
    #[structopt(long)]
    pub use_kubernetes: bool,

    /// Set the callback address for plugins launched from within Pod.
    /// This should be the Pod-s address as seen by external plugins.
    /// It defaults to "pod_pod_1:3030" if Pod is inside docker,
    /// or "localhost:3030" on Linux,
    /// or "host.docker.internal:3030" on other operating systems.
    #[structopt(
        short = "s",
        long,
        name = "ADDRESS",
        env = "POD_PLUGINS_CALLBACK_ADDRESS"
    )]
    pub plugins_callback_address: Option<String>,

    /// Plugin's public address, as seen by Memri clients and the external network.
    /// This is passed to the Plugins so that they can know (and advertise) where they are.
    ///
    /// If not specified, Pod will assume the plugin runs on `localhost`.
    #[structopt(long, env)]
    pub plugins_public_domain: Option<String>,

    /// Docker network to use when running plugins, e.g. `docker run --network=XXX ...`
    /// If not set, "bridge" will be used, which means that started plugins
    /// will share the network with the host system.
    /// If Pod itself is running inside docker, please run both Pod and plugins
    /// in identical network that will then not be shared with the host system
    /// (this is covered in docker-compose.yml by default).
    #[structopt(
        long,
        default_value = "bridge",
        name = "PLUGINS_DOCKER_NETWORK",
        env = "POD_PLUGINS_DOCKER_NETWORK",
    )]
    pub plugins_docker_network: String,

    /// File to read https public certificate from.
    #[structopt(
        short = "c",
        long,
        default_value = "./data/certs/pod.crt",
        name = "CERTIFICATE_FILE"
    )]
    pub tls_pub_crt: String,

    /// File to read https private key from.
    #[structopt(
        short = "k",
        long,
        default_value = "./data/certs/pod.key",
        name = "KEY_FILE"
    )]
    pub tls_priv_key: String,

    /// Override a certain Plugin container to run a script instead.
    /// The value specified should be of the form `image_name=/path/to/your/local/script`.
    /// Whenever Pod is supposed to run a container with this name, it will run
    /// the specified script instead. This argument can be useful during plugin development,
    ///
    /// for example: `my_plugin_container_name=/home/john/memri/my_plugin/script.py`
    /// or `my_plugin_container_name=C:\\memri\\my_plugin\\plugin.exe`
    /// or `my_plugin_container_name=/Users/john/memri/my_plugin/script.py`
    ///
    /// Note that running scripts instead of containers is not secure. A container is limited
    /// in its access to the filesystem, but running a script is only secure if you know and trust
    /// the script.
    #[structopt(
        long,
        parse(try_from_str = parse_key_val),
        number_of_values = 1,
    )]
    pub insecure_plugin_script: Vec<(String, String)>,

    /// Do not use https when starting the server, instead run on http://127.0.0.1.
    /// Running on loopback interface (127.0.0.1) means that only apps
    /// from within the same computer will be able to access Pod.
    /// This option might be used during development as an alternative to self-signed certificates.
    #[structopt(short = "t", long, env = "POD_NON_TLS")]
    pub non_tls: bool,

    /// Unsafe version of --non-tls that runs on a public network, e.g. "http://0.0.0.0".
    /// This option will force Pod to not use https,
    /// and instead run http on the provided network interface.
    /// WARNING: This is heavily discouraged as an intermediary
    /// (even your router on a local network)
    /// could spoof the traffic sent to the server and do a MiTM attack.
    /// Please consider running Pod on a non-public network (--non-tls),
    /// or use Pod with https encryption.
    #[structopt(long, name = "NETWORK_INTERFACE", env = "POD_INSECURE_NON_TLS")]
    pub insecure_non_tls: Option<IpAddr>,

    /// Run server as a "SharedServer". See `/docs/SharedServer.md` documentation
    /// for details on what it is, and how it works.
    #[structopt(long, env)]
    pub shared_server: bool,

    /// SMTP relay server to use (advanced option).
    #[structopt(long, env)]
    pub email_smtp_relay: Option<String>,

    /// SMTP relay server port to use (advanced option).
    #[structopt(long, default_value = "465", env)]
    pub email_smtp_port: u16,

    /// SMTP relay server user (advanced option).
    #[structopt(long, env)]
    pub email_smtp_user: Option<String>,

    /// SMTP relay server password (advanced option).
    #[structopt(long, env)]
    pub email_smtp_password: Option<String>,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

lazy_static! {
    pub static ref VERSION: String = crate::internal_api::get_project_version();
}

lazy_static! {
    pub static ref PARSED: CliOptions = CliOptions::from_args();
}

#[cfg(test)]
pub mod tests {
    use super::CliOptions;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;

    /// Example test CLI. Purely for convenience,
    /// you can instantiate your own / unrelated ones as well.
    pub fn test_cli() -> CliOptions {
        CliOptions {
            port: 3030,
            owners: "ANY".to_string(),
            use_kubernetes: false,
            plugins_callback_address: None,
            plugins_public_domain: None,
            plugins_docker_network: "bridge".to_string(),
            insecure_plugin_script: Vec::new(),
            tls_pub_crt: "".to_string(),
            tls_priv_key: "".to_string(),
            non_tls: true,
            insecure_non_tls: Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            shared_server: false,
            email_smtp_relay: None,
            email_smtp_port: 465,
            email_smtp_user: None,
            email_smtp_password: None,
        }
    }
}
