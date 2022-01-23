use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

pub type Result<T> = std::result::Result<T, eyre::Report>;
pub type HttpResult<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error {
    e: eyre::Report,
}

impl From<eyre::Report> for Error {
    fn from(e: eyre::Report) -> Self {
        Self { e }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let json = json!({
            "error": format!("{}", self.e),
            "chain": self.e.chain().map(|x| format!("{x}")).collect::<Vec<String>>(),
        });

        HttpResponse::InternalServerError().json(json)
    }
}
