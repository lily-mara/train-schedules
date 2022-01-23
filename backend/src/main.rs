use crate::error::{handle_rejection, Result};
use db::Service;
use eyre::Context;
use reqwest::Client;
use std::{env, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;
use train_schedules_common::*;
use ttl_cache::TtlCache;
use warp::{query, Filter};

mod db;
mod error;
mod routes;
mod types;

type LiveStatusCache = Arc<RwLock<TtlCache<(), Vec<Stop>>>>;

pub struct AppState {
    pub stations: Vec<Station>,
    pub stops: Vec<Stop>,
    pub client: Client,
    pub api_key: String,
    pub live_status_cache: LiveStatusCache,
    pub services: Vec<Service>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = dotenv::dotenv();

    color_backtrace::install();
    tracing_subscriber::fmt::init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/var/schedules.db".to_owned());
    let api_key = env::var("API_KEY").wrap_err("API_KEY environment variable is required")?;
    // let static_file_path = env::var("STATIC_FILE_PATH");

    let connection =
        sqlite::Connection::open(&db_path).wrap_err("failed to open sqlite connection")?;

    info!(socket = "0.0.0.0:8088", "listening");

    let live_status_cache = Arc::new(RwLock::new(TtlCache::new(50)));

    let state = Arc::new(AppState {
        api_key: api_key.clone(),
        client: Client::new(),
        live_status_cache: live_status_cache.clone(),
        stations: db::all_stations(&connection)?,
        stops: db::all_stops(&connection)?,
        services: db::services(&connection)?,
    });

    let state = warp::any().map(move || state.clone());

    let stations = warp::path!("api" / "stations")
        .and(state.clone())
        .map(|state: Arc<AppState>| warp::reply::json(&state.stations));

    let upcoming = warp::path!("api" / "upcoming-trips")
        .and(query())
        .and(state.clone())
        .and_then(
            |query, state| async move { crate::routes::upcoming::upcoming_trips(query, state) },
        );

    let trip = warp::path!("api" / "trip")
        .and(query())
        .and(state.clone())
        .map(crate::routes::trip::trip);

    let live = warp::path!("api" / "stations" / "live")
        .and(state.clone())
        .and_then(crate::routes::live::live_station);

    let index = warp::any().and(warp::filters::fs::file("/var/www/index.html"));

    let routes = warp::get()
        .and(index.or(stations).or(upcoming).or(trip).or(live))
        .recover(handle_rejection);

    warp::serve(routes).run(([0, 0, 0, 0], 8088)).await;

    /*
            .route(
                "/api/stations/live",
                web::get().to(crate::routes::live::live_station),
            )

        if let Ok(path) = &static_file_path {
            app = app.service(Files::new("/", path));
        }
    */

    Ok(())
}
