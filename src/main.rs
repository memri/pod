mod data_model;
mod dgraph_database;
mod internal_api;
mod sync_state;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use std::io::Write;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    let dgraph = dgraph_database::create_dgraph();
    // Drop and set up schema only in DROP mode.
    // Add "APP_DROP=true" before executable, default is FALSE.
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap()
        .merge(config::Environment::with_prefix("SCHEMA"))
        .unwrap();

    if settings.get_str("drop").unwrap().eq("true") {
        dgraph_database::drop_schema_and_all_data_irreversibly(&dgraph);
    }
    if settings.get_str("set").unwrap().eq("true") {
        dgraph_database::set_schema(&dgraph);
    }
    // Start web framework warp.
    warp_api::run_server(env!("CARGO_PKG_NAME").to_uppercase(), dgraph).await;
}
