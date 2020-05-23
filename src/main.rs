mod data_model;
pub mod dgraph_database;
pub mod internal_api;
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

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap()
        .merge(config::Environment::new())
        .unwrap();

    let dgraph = dgraph_database::create_dgraph(&settings.get_str("dgraph_host").unwrap());

    // Drop the old Dgraph schema and all its data, if asked to
    if settings.get_bool("drop_schema_and_all_data").unwrap() {
        dgraph_database::drop_schema_and_all_data_irreversibly(&dgraph);
    }

    // Add Dgraph schema, if asked to
    if settings.get_bool("add_schema_on_start").unwrap() {
        dgraph_database::add_schema(&dgraph);
    }
    // Start web framework warp.
    warp_api::run_server(env!("CARGO_PKG_NAME").to_uppercase(), dgraph).await;
}
