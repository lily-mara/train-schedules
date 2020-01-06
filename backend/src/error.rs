use actix_web::{HttpResponse, ResponseError};
use auto_from::From;
use serde_json::json;
use std::fmt;

#[derive(From, Debug)]
pub enum Error {
    ParseIntError(std::num::ParseIntError),
    SqlError(sqlite::Error),
    JsonError(serde_json::Error),
    ReqwestError(reqwest::Error),

    #[auto_from(skip)]
    NoSuchStation(i64),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseIntError(e) => write!(f, "{}", e),
            Error::SqlError(e) => write!(f, "{}", e),
            Error::JsonError(e) => write!(f, "{}", e),
            Error::ReqwestError(e) => write!(f, "{}", e),
            Error::NoSuchStation(station_id) => {
                write!(f, "No station found with ID: {}", station_id)
            }
        }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let self_str = format!("{}", self);
        let json = json!({ "error": self_str });

        let mut response = match self {
            Error::NoSuchStation(_) => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        };

        response.json(json)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
