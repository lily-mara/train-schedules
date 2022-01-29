use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use serde_json::json;

pub type HttpResult<T> = std::result::Result<Json<T>, Error>;

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

pub fn eyre_into_response(e: impl Into<eyre::Report>) -> Response {
    let error = e.into();
    let json = json!({
        "error": format!("{error}"),
        "chain": error.chain().map(|e| format!("{e}")).collect::<Vec<String>>(),
    });

    (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        eyre_into_response(self.e)
    }
}
