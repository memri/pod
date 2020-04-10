use crate::api_model;
use rweb::*;

/// Creates a DataItem
#[post("/items/")]
pub fn create(_: Json<api_model::DataItem>) -> Result<String, http::Error> {
    Ok(String::from(""))
}

/// Returns the version of this pod instance
#[get("/version")]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
