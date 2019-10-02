use crate::error::{Error, Result};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{prelude::*, Weekday};
use serde::Deserialize;
use std::collections::HashMap;
use train_schedules_common::*;

mod error;

#[derive(Deserialize, Debug, Clone)]
struct UpcomingTripsQuery {
    start: i64,
    end: i64,
}

struct AppState {
    connection: sqlite::Connection,
}

fn main() {
    let sys = actix_rt::System::new("example");

    HttpServer::new(|| {
        App::new()
            .data(AppState {
                connection: sqlite::Connection::open("schedules.db").unwrap(),
            })
            .route("/", web::get().to(index))
            .route("/stations", web::get().to(stations))
            .route("/upcoming-trips", web::get().to(upcoming_trips))
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .start();

    eprintln!("Listening on 0.0.0.0:8088");

    let _ = sys.run();
}

fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}

fn get_trip_ids_that_hit_stations(
    connection: &sqlite::Connection,
    start_station: i64,
    end_station: i64,
) -> Result<Vec<i64>> {
    let mut statement = connection.prepare(
        "
        select trip_id from stop_times where stop_id=?
        intersect
        select trip_id from stop_times where stop_id=?
        ",
    )?;

    statement.bind(1, start_station)?;
    statement.bind(2, end_station)?;

    let mut trip_ids = Vec::new();

    while let Ok(sqlite::State::Row) = statement.next() {
        trip_ids.push(statement.read(0)?);
    }

    Ok(trip_ids)
}

fn load_station(connection: &sqlite::Connection, station_id: i64) -> Result<Station> {
    let mut stmt = connection.prepare(
        "
            select distinct stop_name, stops.stop_id, direction_id
            from trips
            join stop_times on stop_times.trip_id = trips.trip_id
            join stops on stops.stop_id = stop_times.stop_id
            where stops.stop_id=?;
        ",
    )?;

    stmt.bind(1, station_id)?;

    match stmt.next()? {
        sqlite::State::Row => Ok(Station {
            name: stmt.read(0)?,
            station_id: stmt.read(1)?,
            direction: stmt.read::<i64>(2)?.into(),
        }),
        sqlite::State::Done => Err(Error::NoSuchStation(station_id)),
    }
}

fn load_all_stations(connection: &sqlite::Connection) -> Result<Vec<Station>> {
    let mut stmt = connection.prepare(
        "

            select distinct stop_name, stops.stop_id, direction_id
            from trips
            join stop_times on stop_times.trip_id = trips.trip_id
            join stops on stops.stop_id = stop_times.stop_id
            order by stops.stop_id asc

        ",
    )?;

    let mut stations = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        stations.push(Station {
            name: stmt.read(0)?,
            station_id: stmt.read(1)?,
            direction: stmt.read::<i64>(2)?.into(),
        });
    }

    Ok(stations)
}

fn get_upcoming_trips(
    connection: &sqlite::Connection,
    start_station_id: i64,
    end_station_id: i64,
) -> Result<TripList> {
    let trip_ids = get_trip_ids_that_hit_stations(connection, start_station_id, end_station_id)?;

    let service_ids = get_active_service_ids(connection)?;

    let mut stmt = connection.prepare(format!(
        "
        select departure_minute, arrival_minute, stop_times.trip_id, stop_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        where (stop_id=? or stop_id=?) and stop_times.trip_id=? and service_id in ({})
        order by stop_times.trip_id
        ",
        bind_placeholders(service_ids.len()),
    ))?;

    let start_station = load_station(connection, start_station_id)?;
    let end_station = load_station(connection, end_station_id)?;
    let minute = current_minute();

    let mut trips = HashMap::new();

    for id in trip_ids {
        stmt.bind(1, start_station_id)?;
        stmt.bind(2, end_station_id)?;
        stmt.bind(3, id)?;
        for (idx, i) in service_ids.iter().enumerate() {
            stmt.bind(idx + 4, &sqlite::Value::String(i.clone()))?;
        }

        while let sqlite::State::Row = stmt.next()? {
            let station_name = match stmt.read::<i64>(3)? {
                x if x == start_station_id => "start",
                x if x == end_station_id => "end",
                x => panic!("Got unexpected station ID: {}", x),
            };

            let trip_id = stmt.read(2)?;

            let departure_minute = stmt.read(0)?;

            if departure_minute < minute {
                continue;
            }

            trips.entry(trip_id).or_insert_with(HashMap::new).insert(
                station_name,
                Departure {
                    departure_minute,
                    arrival_minute: stmt.read(1)?,
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

            Some(Trip {
                trip_id,
                start,
                end,
            })
        })
        .collect();

    trips.sort_by(|a, b| a.start.departure_minute.cmp(&b.start.departure_minute));

    Ok(TripList {
        trips,
        start: start_station,
        end: end_station,
    })
}

fn get_active_service_ids(connection: &sqlite::Connection) -> Result<Vec<String>> {
    let today = chrono::Local::now();
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

fn upcoming_trips(
    query: web::Query<UpcomingTripsQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let trips = get_upcoming_trips(&data.connection, query.start, query.end)?;

    Ok(HttpResponse::Ok().json(trips))
}

fn stations(_req: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(load_all_stations(&data.connection)?))
}

pub fn current_minute() -> i64 {
    let now = chrono::Local::now();
    (now.hour() * 60 + now.minute()) as i64
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
