use std::convert::Infallible;

use reqwest::StatusCode;
use serde_json::json;
use warp::{reject::Reject, reply::Json, Rejection, Reply};

pub type Result<T> = std::result::Result<T, eyre::Report>;
pub type HttpResult = std::result::Result<Json, Rejection>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error {
    e: eyre::Report,
}

impl Reject for Error {}

impl From<eyre::Report> for Error {
    fn from(e: eyre::Report) -> Self {
        Self { e }
    }
}

pub trait RejectionExt<T> {
    fn rejection(self) -> std::result::Result<T, Rejection>;
}

impl<T> RejectionExt<T> for std::result::Result<T, eyre::Report> {
    fn rejection(self) -> std::result::Result<T, Rejection> {
        self.map_err(|e| warp::reject::custom(Error { e }))
    }
}

// impl ResponseError for Error {
//     fn error_response(&self) -> HttpResponse {

//         HttpResponse::InternalServerError().json(json)
//     }
// }

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let json;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        json = json!({
            "error": "not found"
        });
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        json = json!({
            "error": "method not allowed"
        });
    } else if let Some(e) = err.find::<Error>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        json = json!({
            "error": format!("{e}"),
            "chain": e.e.chain().map(|e| format!("{e}")).collect::<Vec<String>>(),
        });
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        json = json!({
            "error": format!("uncaught error {err:?}"),
        });
    }

    let json = warp::reply::json(&json);

    Ok(warp::reply::with_status(json, code))
}
