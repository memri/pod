extern crate pod;

use dgraph::Dgraph;
use lazy_static::lazy_static;
use pod::dgraph_database::create_dgraph;

// Create a dgraph instance.
lazy_static! {
    pub static ref DGRAPH: Dgraph = {
        let mut settings = config::Config::default();
        settings.merge(config::Environment::new()).unwrap();

        let dgraph_host = settings.get_str("dgraph_host");
        let dgraph_host = dgraph_host.unwrap_or_else(|_| "localhost:9080".to_string());
        create_dgraph(&dgraph_host)
    };
}
