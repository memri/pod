use warp::http::status::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub code: StatusCode,
    pub msg: String,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let canon = self.code.canonical_reason().unwrap_or("");
        write!(f, "Error {} {}, {}", self.code.as_str(), canon, self.msg)
    }
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

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        let msg = format!("JSON formatting error {}", err);
        Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        }
    }
}
