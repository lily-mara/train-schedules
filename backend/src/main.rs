use axum::{
    body::Body,
    extract::Extension,
    http::Request,
    response::Response,
    routing::{get, get_service},
    AddExtensionLayer, Json, Router,
};
use db::Service;
use eyre::{Context, Result};
use opentelemetry::trace::SpanKind;
use reqwest::Client;
use std::{env, sync::Arc, time::Duration};
use tokio::sync::RwLock;

use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info_span, Span};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter, Registry};
use train_schedules_common::*;
use ttl_cache::TtlCache;

mod db;
mod error;
mod routes;
mod types;

type LiveStatusCache = Arc<RwLock<TtlCache<(), Vec<Stop>>>>;

pub struct State {
    pub stations: Vec<Station>,
    pub stops: Vec<Stop>,
    pub client: Client,
    pub api_key: String,
    pub live_status_cache: LiveStatusCache,
    pub services: Vec<Service>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();

    color_backtrace::install();

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::EXIT))
        .init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/var/schedules.db".to_owned());
    let api_key = env::var("API_KEY").wrap_err("API_KEY environment variable is required")?;
    // let static_file_path = env::var("STATIC_FILE_PATH");

    let connection =
        sqlite::Connection::open(&db_path).wrap_err("failed to open sqlite connection")?;

    let live_status_cache = Arc::new(RwLock::new(TtlCache::new(50)));

    let state = Arc::new(State {
        api_key: api_key.clone(),
        client: Client::new(),
        live_status_cache: live_status_cache.clone(),
        stations: db::all_stations(&connection)?,
        stops: db::all_stops(&connection)?,
        services: db::services(&connection)?,
    });

    let api_routes =
        Router::new()
            .route(
                "/stations",
                get(|Extension(state): Extension<Arc<State>>| async move {
                    Json(state.stations.clone())
                }),
            )
            .route(
                "/upcoming-trips",
                get(crate::routes::upcoming::upcoming_trips),
            )
            .route("/trip", get(crate::routes::trip::trip))
            .route("/stations/live", get(crate::routes::live::live_station));

    let app = Router::new()
        .nest("/api", api_routes)
        .route(
            "/c/",
            get_service(ServeFile::new("/var/www/index.html"))
                .handle_error(|e: std::io::Error| async move { error::eyre_into_response(e) }),
        )
        .route(
            "/",
            get_service(ServeDir::new("/var/www/"))
                .handle_error(|e: std::io::Error| async move { error::eyre_into_response(e) }),
        )
        .layer(AddExtensionLayer::new(state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| http_span(request, SpanKind::Server))
                .on_response(|response: &Response, _duration: Duration, span: &Span| {
                    http_span_response(response, span)
                }),
        );

    axum::Server::bind(&"0.0.0.0:8088".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn http_span<B>(request: &axum::http::Request<B>, kind: SpanKind) -> Span {
    info_span!(
        "http request",
        name = &format!("{} {}", request.method(), request.uri().path())[..],
        http.target = request.uri().path(),
        http.url = tracing::field::display(&request.uri()),
        http.method = request.method().as_str(),
        http.status_code = tracing::field::Empty,
        otel.kind = %kind,
    )
}

fn http_span_response<B>(response: &axum::http::Response<B>, span: &Span) {
    span.record("http.status_code", &response.status().as_u16());
}
