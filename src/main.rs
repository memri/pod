mod data_model;
pub mod dgraph_database;
pub mod importers;
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

    // Drop the old Dgraph schema and all its data, if asked to.
    if settings.get_bool("drop_schema_and_all_data").unwrap() {
        dgraph_database::drop_schema_and_all_data_irreversibly(&dgraph);
    }

    // Add Dgraph schema, if asked to.
    if settings.get_bool("add_schema_on_start").unwrap() {
        dgraph_database::add_schema(&dgraph);
    }

    // Import notes from Evernote.
    if settings.get_bool("import_notes_evernote").unwrap() {
        println!("Importing notes from Evernote folder");
        importers::note_importer::import_notes(&dgraph, "data/Evernote".to_string());
        // TODO: create test for
        // importers::note_importer::query_notes(&dgraph);
    }
    // TODO: create test for
    // importers::note_importer::simple_example(&dgraph);
    // Import notes from iCloud.
    if settings.get_bool("import_notes_icloud").unwrap() {
        println!("Importing notes from iCloud folder");
        importers::note_importer::import_notes(&dgraph, "data/iCloud".to_string());
    }

    // Start web framework warp.
    warp_api::run_server(env!("CARGO_PKG_NAME").to_uppercase(), dgraph).await;
}
