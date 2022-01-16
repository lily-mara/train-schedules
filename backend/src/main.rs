use crate::error::{Error, Result};
use actix_files::{Files, NamedFile};
use actix_web::{client::Client, web, App, HttpServer};
use chrono::prelude::*;
use chrono_tz::US::Pacific;
use log::*;
use sqlite::Statement;
use std::env;
use train_schedules_common::*;

mod error;
mod routes;
mod types;

pub struct AppState {
    pub connection: sqlite::Connection,
    pub client: Client,
    pub api_key: String,
}

#[actix_rt::main]
async fn main() {
    color_backtrace::install();
    env_logger::init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/var/schedules.db".to_owned());
    let api_key = env::var("API_KEY").expect("API_KEY environment variable is required");

    info!("Listening on 0.0.0.0:8088");

    HttpServer::new(move || {
        debug!("Opening sqlite connection: {}", db_path);
        App::new()
            .data(AppState {
                connection: sqlite::Connection::open(&db_path)
                    .expect("Failed to connect to sqlite database"),
                api_key: api_key.clone(),
                client: Client::new(),
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
                web::get().to(crate::routes::twostops::upcoming_trips),
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
    .expect("Failed to run server");
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("/var/www/index.html").map_err(Error::File)?)
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    let mut parts = time.split(':');

    let mut add_days = 0;
    let mut hour = parts.next().unwrap().parse()?;
    while hour >= 24 {
        hour -= 24;
        add_days += 1;
    }

    let minute = parts.next().unwrap().parse()?;
    let second = parts.next().unwrap().parse()?;

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

    while let sqlite::State::Row = stmt.next()? {
        let station_name: String = stmt.read(0)?;

        let station_id: i64 = stmt.read(1)?;

        let departure_str: String = stmt.read(2)?;
        let departure = parse_time(&departure_str)?;

        let arrival_str: String = stmt.read(3)?;
        let arrival = parse_time(&arrival_str)?;

        let trip_id = stmt.read(4)?;

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
