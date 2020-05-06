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
    // Create dgraph-rs instance.
    let dgraph = dgraph_database::create_dgraph();
    // Set up schema.
    dgraph_database::drop_schema(&dgraph);
    dgraph_database::set_schema(&dgraph);
    // Start web framework warp.
    warp_api::run_server(env!("CARGO_PKG_NAME").to_uppercase(), dgraph).await;
}
