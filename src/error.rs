use warp::http::status::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub code: StatusCode,
    pub msg: String,
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        let msg = format!("Database r2d2 error {}", err);
        Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        let msg = format!("Database rusqlite error {}", err);
        Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg,
        }
    }
}
