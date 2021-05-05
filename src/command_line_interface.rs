use lazy_static::lazy_static;
use std::net::IpAddr;
use std::path::PathBuf;
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
    #[structopt(short = "o", long, required = true, env = "POD_OWNER_HASHES")]
    pub owners: String,

    /// Set the callback address for plugins launched from within Pod.
    /// This should be the Pod-s address as seen by external plugins.
    /// It defaults to "pod_pod_1:3030" if Pod is inside docker,
    /// or "localhost:3030" on Linux,
    /// or "host.docker.internal:3030" on other operating systems.
    #[structopt(short = "s", long, name = "ADDRESS", env = "PLUGINS_CALLBACK_ADDRESS")]
    pub plugins_callback_address: Option<String>,

    /// Docker network to use when running plugins, e.g. `docker run --network=XXX ...`
    /// If not set, "host" will be used, which means that started plugins
    /// will share the network with the host system.
    /// If Pod itself is running inside docker, please run both Pod and plugins
    /// in identical network that will then not be shared with the host system
    /// (this is covered in docker-compose.yml by default).
    #[structopt(
        long,
        name = "SERVICES_DOCKER_NETWORK",
        env = "SERVICES_DOCKER_NETWORK"
    )]
    pub plugins_docker_network: Option<String>,

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

    /// Do not use https when starting the server, instead run on http://127.0.0.1.
    /// Running on loopback interface (127.0.0.1) means that only apps
    /// from within the same computer will be able to access Pod.
    /// This option might be used during development as an alternative to self-signed certificates.
    #[structopt(short = "t", long)]
    pub non_tls: bool,

    /// Unsafe version of --non-tls that runs on a public network, e.g. "http://0.0.0.0".
    /// This option will force Pod to not use https when starting the server,
    /// instead run on http on the provided network interface.
    /// WARNING: This is heavily discouraged as an intermediary
    /// (even your router on a local network)
    /// could spoof the traffic sent to the server and do a MiTM attack.
    /// Please consider running Pod on a non-public network (--non-tls),
    /// or use Pod with https encryption.
    #[structopt(long, name = "NETWORK_INTERFACE", env = "INSECURE_NON_TLS")]
    pub insecure_non_tls: Option<IpAddr>,

    /// Add `Access-Control-Allow-Origin: *` header to all HTTP responses,
    /// and make the server answer to ORIGIN requests.
    #[structopt(long)]
    pub insecure_http_headers: bool,

    /// Run server as a "SharedServer". See `/docs/SharedServer.md` documentation
    /// for details on what it is, and how it works.
    #[structopt(long)]
    pub shared_server: bool,

    /// Deprecated: schema file to use.
    /// Note that this Schema is in the process of being re-written to a different format,
    /// The way it works will change in a breaking manner.
    #[structopt(
        long,
        name = "SCHEMA_FILE",
        parse(from_os_str),
        default_value = "res/default_schema.json"
    )]
    pub schema_file: PathBuf,

    /// Validate the schema file and exit. Useful in combination with the --schema-file CLI key.
    #[structopt(long)]
    pub validate_schema: bool,
}

lazy_static! {
    // Don't change to `&'static str` for now, it's a bit hard to get lifetimes straight with Clap.
    pub static ref VERSION: String = {
        // Ideas for future:
        //   * print "which branch" does the commit belong to
        //   * print "dirty" indicator (whether there are local uncommitted changes)
        env!("GIT_DESCRIBE").to_string()
    };
}

lazy_static! {
    pub static ref PARSED: CliOptions = CliOptions::from_args();
}

#[cfg(test)]
pub mod tests {
    use super::CliOptions;
    use std::net::{IpAddr, Ipv4Addr};

    pub fn new_cli() -> CliOptions {
        CliOptions {
            port: 0,
            owners: "".to_string(),
            plugins_callback_address: None,
            plugins_docker_network: None,
            tls_pub_crt: "".to_string(),
            tls_priv_key: "".to_string(),
            non_tls: true,
            insecure_non_tls: Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            insecure_http_headers: false,
            shared_server: false,
            schema_file: Default::default(),
            validate_schema: false,
        }
    }
}
