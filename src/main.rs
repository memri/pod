mod data_model;
mod database_generated_schema;
mod database_schema_struct;
pub mod internal_api;
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

    // Start web framework
    warp_api::run_server().await;
}
