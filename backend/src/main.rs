use crate::error::{Error, Result};
use actix_files::{Files, NamedFile};
use actix_web::{
    client::Client, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::{prelude::*, Weekday};
use log::*;
use serde::Deserialize;
use std::{collections::HashMap, env};
use train_schedules_common::*;

mod error;
mod types;

#[derive(Deserialize, Debug, Clone)]
struct UpcomingTripsQuery {
    start: i64,
    end: i64,
}

struct AppState {
    connection: sqlite::Connection,
    client: Client,
    api_key: String,
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
            .route("/api/stations", web::get().to(stations))
            .route("/api/upcoming-trips", web::get().to(upcoming_trips))
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
    Ok(NamedFile::open("/var/www/index.html").map_err(Error::FileIOError)?)
}

fn get_trip_ids_that_hit_stations(
    connection: &sqlite::Connection,
    start_station: i64,
    end_station: i64,
    service_ids: &[String],
) -> Result<Vec<i64>> {
    let mut statement = connection.prepare(
        &format!("
        select trip_id from stop_times join stops on stops.stop_id =stop_times.stop_id where station_id=?
        intersect
        select trip_id from stop_times join stops on stops.stop_id =stop_times.stop_id where station_id=?
        intersect
        select trip_id from trips where service_id in ({})
        ",         bind_placeholders(service_ids.len())),
    )?;

    statement.bind(1, start_station)?;
    statement.bind(2, end_station)?;

    for (idx, i) in service_ids.iter().enumerate() {
        statement.bind(idx + 3, &sqlite::Value::String(i.clone()))?;
    }

    let mut trip_ids = Vec::new();

    while let Ok(sqlite::State::Row) = statement.next() {
        trip_ids.push(statement.read(0)?);
    }

    Ok(trip_ids)
}

fn load_station(connection: &sqlite::Connection, station_id: i64) -> Result<Station> {
    let mut stmt = connection.prepare(
        "
            select distinct stop_name, station_id
            from trips
            join stop_times on stop_times.trip_id = trips.trip_id
            join stops on stops.stop_id = stop_times.stop_id
            where station_id=?;
        ",
    )?;

    stmt.bind(1, station_id)?;

    match stmt.next()? {
        sqlite::State::Row => Ok(Station {
            name: stmt.read(0)?,
            station_id: stmt.read::<i64>(1)? as i32,
        }),
        sqlite::State::Done => Err(Error::NoSuchStation(station_id)),
    }
}

fn load_all_stations(connection: &sqlite::Connection) -> Result<Vec<Station>> {
    let mut stmt = connection.prepare(
        "
            select distinct stop_name, station_id
            from stops
            order by stops.stop_id asc
        ",
    )?;

    let mut stations = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        stations.push(Station {
            name: stmt.read(0)?,
            station_id: stmt.read::<i64>(1)? as i32,
        });
    }

    Ok(stations)
}

fn get_upcoming_trips(
    connection: &sqlite::Connection,
    start_station_id: i64,
    end_station_id: i64,
) -> Result<TripList> {
    let service_ids = get_active_service_ids(connection)?;

    let trip_ids =
        get_trip_ids_that_hit_stations(connection, start_station_id, end_station_id, &service_ids)?;

    let mut stmt = connection.prepare(
        "
        select distinct station_id, departure_time, arrival_time, stop_times.trip_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        join stops on stop_times.stop_id = stops.stop_id
        where
            stop_times.trip_id = ? and
            station_id in (?, ?)
        ",
    )?;

    let start_station = load_station(connection, start_station_id)?;
    let end_station = load_station(connection, end_station_id)?;
    let now = Local::now().with_timezone(&FixedOffset::east(0));

    let mut trips = HashMap::new();

    for id in trip_ids {
        stmt.bind(1, id)?;
        stmt.bind(2, start_station_id)?;
        stmt.bind(3, end_station_id)?;

        while let sqlite::State::Row = stmt.next()? {
            let station_name = match stmt.read::<i64>(0)? {
                x if x == start_station_id => "start",
                x if x == end_station_id => "end",
                _ => continue,
            };

            let departure_str: String = stmt.read(1)?;
            let departure = parse_time(&departure_str)?;

            let arrival_str: String = stmt.read(2)?;
            let arrival = parse_time(&arrival_str)?;

            if departure < now || arrival < now {
                continue;
            }

            let trip_id = stmt.read::<i64>(3)? as i32;

            trips.entry(trip_id).or_insert_with(HashMap::new).insert(
                station_name,
                Departure {
                    departure: Time::new(departure),
                    arrival: Time::new(departure),
                },
            );
        }
        stmt.reset()?;
    }

    let mut trips: Vec<_> = trips
        .into_iter()
        .filter_map(|(trip_id, mut stations)| {
            let start = stations.remove("start")?;
            let end = stations.remove("end")?;

            if *start.arrival > *end.departure {
                return None;
            }

            Some(Trip {
                trip_id,
                start,
                end,
            })
        })
        .collect();

    trips.sort_by(|a, b| a.start.departure.cmp(&b.start.departure));

    Ok(TripList {
        trips,
        start: start_station,
        end: end_station,
    })
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

    let time = Local::today().and_hms(hour, minute, second) + chrono::Duration::days(add_days);

    Ok(time.with_timezone(&time.offset()))
}

fn to_local_time(time: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    time.with_timezone(&Local::now().offset())
}

async fn get_station_estimated_stuff(
    client: &Client,
    api_key: &str,
    station_id: i32,
) -> Result<HashMap<i32, (DateTime<FixedOffset>, DateTime<FixedOffset>)>> {
    let url = format!(
        "https://api.511.org/transit/StopMonitoring?api_key={api_key}&agency=CT&format=json&stopCode={stop_code}",
        api_key=api_key,
        stop_code=station_id
    );

    let mut response = client.get(&url).send().await?;

    let body = response.body().await?;

    if response.status() != StatusCode::OK {
        let body = String::from_utf8_lossy(&body).into_owned();
        return Err(Error::FiveOneOneServerError {
            code: response.status(),
            body,
        });
    }

    let resp: types::ApiResponse = serde_json::from_slice(&body[3..])?;

    debug!("Parsed API response: {:?}", resp);
    let mut trips = HashMap::new();

    for visit in resp
        .ServiceDelivery
        .StopMonitoringDelivery
        .MonitoredStopVisit
    {
        if let Some(vehicle_ref) = visit.MonitoredVehicleJourney.VehicleRef {
            if let Ok(trip_id) = vehicle_ref.parse() {
                trips.insert(
                    trip_id,
                    (
                        to_local_time(
                            visit
                                .MonitoredVehicleJourney
                                .MonitoredCall
                                .ExpectedDepartureTime,
                        ),
                        to_local_time(
                            visit
                                .MonitoredVehicleJourney
                                .MonitoredCall
                                .ExpectedArrivalTime,
                        ),
                    ),
                );
            }
        }
    }

    Ok(trips)
}

async fn add_live_status(client: &Client, api_key: &str, trips: &mut TripList) -> Result<()> {
    let start = trips.start.station_id;
    let end = trips.end.station_id;

    let (start_data, end_data) = futures::try_join!(
        get_station_estimated_stuff(client, &api_key, start),
        get_station_estimated_stuff(client, &api_key, end)
    )?;

    for trip in &mut trips.trips {
        if let Some((departure, arrival)) = start_data.get(&trip.trip_id) {
            trip.start.departure.estimated = Some(*departure);
            trip.start.arrival.estimated = Some(*arrival);
        }

        if let Some((departure, arrival)) = end_data.get(&trip.trip_id) {
            trip.end.departure.estimated = Some(*departure);
            trip.end.arrival.estimated = Some(*arrival);
        }
    }

    Ok(())
}

fn get_active_service_ids(connection: &sqlite::Connection) -> Result<Vec<String>> {
    let today = Local::now();
    let weekday = match today.weekday() {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
    };

    let today_num = (today.year() as i64 * 100 + today.month() as i64) * 100 + today.day() as i64;

    let mut stmt = connection.prepare(format!(
        "
        select service_id
        from calendar
        where
            {} = 1 and ? >= start_date and ? <= end_date
        ",
        weekday
    ))?;

    stmt.bind(1, today_num)?;
    stmt.bind(2, today_num)?;

    let mut ids = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        ids.push(stmt.read(0)?);
    }

    Ok(ids)
}

async fn upcoming_trips(
    query: web::Query<UpcomingTripsQuery>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let mut trips = get_upcoming_trips(&data.connection, query.start, query.end)?;

    if let Err(e) = add_live_status(&data.client, &data.api_key, &mut trips).await {
        error!("Error adding realtime status to trips: {:?}", e.chain());
    }

    Ok(HttpResponse::Ok().json(trips))
}

async fn stations(_req: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(load_all_stations(&data.connection)?))
}

fn bind_placeholders(num: usize) -> String {
    let mut placeholders = String::with_capacity(num * 2);

    for _ in 0..num {
        placeholders.push('?');
        placeholders.push(',');
    }

    placeholders.pop();

    placeholders
}
