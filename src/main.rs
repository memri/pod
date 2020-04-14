mod dgraph_database;
mod internal_api;
mod warp_api;

use chrono::Utc;
use env_logger::Env;
use std::io::Write;

use crate::dgraph_database::set_schema;

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

    set_schema(&dgraph);

    warp_api::run_server(env!("CARGO_PKG_NAME").to_uppercase(), dgraph).await;
}
