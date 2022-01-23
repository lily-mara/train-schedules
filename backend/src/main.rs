use crate::error::{HttpResult, Result};
use actix_files::{Files, NamedFile};
use actix_web::{client::Client, web, App, HttpServer};
use chrono::prelude::*;
use chrono_tz::US::Pacific;
use eyre::Context;
use log::*;
use sqlite::Statement;
use std::env;
use tokio::sync::RwLock;
use train_schedules_common::*;
use ttl_cache::TtlCache;

mod error;
mod routes;
mod types;

type LiveStatusCache = RwLock<TtlCache<(), Vec<Stop>>>;

pub struct AppState {
    pub connection: sqlite::Connection,
    pub client: Client,
    pub api_key: String,
    pub live_status_cache: LiveStatusCache,
}

#[actix_rt::main]
async fn main() -> eyre::Result<()> {
    let _ = dotenv::dotenv();

    color_backtrace::install();
    env_logger::init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/var/schedules.db".to_owned());
    let api_key = env::var("API_KEY").wrap_err("API_KEY environment variable is required")?;

    let connection =
        sqlite::Connection::open(&db_path).wrap_err("failed to open sqlite connection")?;
    connection
        .execute("select 1")
        .wrap_err("failed to execute sqlite test command")?;

    info!("Listening on 0.0.0.0:8088");

    HttpServer::new(move || {
        debug!("Opening sqlite connection: {}", db_path);
        App::new()
            .data(AppState {
                connection: sqlite::Connection::open(&db_path).unwrap(),
                api_key: api_key.clone(),
                client: Client::new(),
                live_status_cache: RwLock::new(TtlCache::new(50)),
            })
            .route(
                "/api/stations",
                web::get().to(crate::routes::stations::stations),
            )
            .route(
                "/api/stations/live",
                web::get().to(crate::routes::live::live_station),
            )
            .route(
                "/api/upcoming-trips",
                web::get().to(crate::routes::upcoming::upcoming_trips),
            )
            .route("/api/trip", web::get().to(crate::routes::trip::trip))
            .route("/", web::get().to(index))
            .service(Files::new("/", "/var/www/"))
            .default_service(web::route().to(index))
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .run()
    .await
    .wrap_err("Failed to run server")?;

    Ok(())
}

async fn index() -> HttpResult<NamedFile> {
    Ok(NamedFile::open("/var/www/index.html").wrap_err("failed to read index")?)
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    let mut parts = time.split(':');

    let mut add_days = 0;
    let mut hour = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse hour part from time value {time}"))?;

    while hour >= 24 {
        hour -= 24;
        add_days += 1;
    }

    let minute = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse minute part from time value {time}"))?;
    let second = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse second part from time value {time}"))?;

    let time = Pacific
        .from_utc_datetime(&Utc::now().naive_utc())
        .date()
        .and_hms(hour, minute, second)
        + chrono::Duration::days(add_days);

    Ok(time.with_timezone(&FixedOffset::west(0)))
}

/// Expecting a statement with the following select columns:
/// - stop_name
/// - station_id
/// - departure_time
/// - arrival_time
/// - stop_times.trip_id
fn read_stops(stmt: &mut Statement) -> Result<Vec<Stop>> {
    let mut stops = Vec::new();

    while let sqlite::State::Row = stmt.next().wrap_err("error reading from sqlite")? {
        let station_name: String = stmt
            .read(0)
            .wrap_err("error reading column 0 from sqlite query")?;

        let station_id: i64 = stmt
            .read(1)
            .wrap_err("error reading column 1 from sqlite query")?;

        let departure_str: String = stmt
            .read(2)
            .wrap_err("error reading column 2 from sqlite query")?;

        let departure = parse_time(&departure_str)?;

        let arrival_str: String = stmt
            .read(3)
            .wrap_err("error reading column 3 from sqlite query")?;

        let arrival = parse_time(&arrival_str)?;

        let trip_id = stmt
            .read(4)
            .wrap_err("error reading column 4 from sqlite query")?;

        stops.push(Stop {
            trip_id,
            station_id,
            station_name,
            arrival,
            departure,
        });
    }

    Ok(stops)
}
