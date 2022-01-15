use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse int")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("error talking to SQL database")]
    Sql(#[from] sqlite::Error),

    #[error("error deserializing JSON")]
    Json(#[from] serde_json::Error),

    #[error("error sending HTTP request")]
    Http(#[from] actix_web::client::SendRequestError),

    #[error("Got bad response from 501 API server. Code={code}, Body={body}")]
    FiveOneOneServer { code: StatusCode, body: String },

    #[error("error receiving HTTP response payload")]
    HttpJson(#[from] actix_web::client::PayloadError),

    #[error("error reading file")]
    File(std::io::Error),

    #[error("No station found with ID: {0}")]
    NoSuchStation(i64),
}

impl Error {
    pub fn chain(&self) -> Vec<String> {
        anyhow::Chain::new(self).map(|e| format!("{}", e)).collect()
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let json = json!({
            "error": format!("{}", self),
            "chain": self.chain(),
        });

        let mut response = match self {
            Error::NoSuchStation(_) => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        };

        response.json(json)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
