extern crate pod;

use dgraph::Dgraph;
use pod::dgraph_database::create_dgraph;
use pod::internal_api::_write_access_audit_log;

#[test]
fn test_writing_audit_logs() {
    let mut settings = config::Config::default();
    settings.merge(config::Environment::new()).unwrap();

    let dgraph_host = settings.get_str("dgraph_host");
    let dgraph_host = dgraph_host.unwrap_or_else(|_| "localhost:9080".to_string());
    let dgraph: Dgraph = create_dgraph(&dgraph_host);
    _write_access_audit_log(&dgraph, 2);
}
