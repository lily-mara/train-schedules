use std::collections::HashMap;

use crate::{
    error::{Error, HttpResult},
    read_stops, AppState, Result,
};
use actix_web::{web, HttpResponse};
use chrono::{Datelike, FixedOffset, Local, Weekday};
use eyre::{bail, Context};
use serde::Deserialize;
use train_schedules_common::{Station, Stop, TwoStop, TwoStopList};

#[derive(Deserialize, Debug, Clone)]
pub struct UpcomingTripsQuery {
    start: i64,
    end: Option<i64>,
}

pub async fn upcoming_trips(
    query: web::Query<UpcomingTripsQuery>,
    data: web::Data<AppState>,
) -> HttpResult<HttpResponse> {
    match query.end {
        Some(end) => Ok(HttpResponse::Ok().json(get_twostops(data, query.start, end).await?)),
        None => Ok(HttpResponse::Ok().json(get_upcoming(data, query.start).await?)),
    }
}

async fn get_upcoming(data: web::Data<AppState>, station_id: i64) -> Result<Vec<Stop>> {
    let service_ids = get_active_service_ids(&data.connection)?;

    let trip_ids = get_trip_ids_that_hit_station(&data.connection, station_id, &service_ids)?;

    let mut stmt = data.connection.prepare(
        "
        select distinct stop_name, station_id, departure_time, arrival_time, stop_times.trip_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        join stops on stop_times.stop_id = stops.stop_id
        where
            stop_times.trip_id = ? and
            station_id in (?)
        ",
    )?;

    let now = Local::now().with_timezone(&FixedOffset::east(0));

    let mut trips = Vec::new();

    for id in trip_ids {
        stmt.bind(1, id)
            .wrap_err("failed to bind trip ID to sqlite statement")?;
        stmt.bind(2, station_id)
            .wrap_err("failed to bind station ID to sqlite statement")?;

        for stop in read_stops(&mut stmt)? {
            if stop.departure < now || stop.arrival < now {
                continue;
            }

            trips.push(stop);
        }

        stmt.reset().wrap_err("failed to reset sqlite statement")?;
    }

    trips.sort_by(|a, b| a.departure.cmp(&b.departure));

    Ok(trips)
}

async fn get_twostops(
    data: web::Data<AppState>,
    start_station_id: i64,
    end_station_id: i64,
) -> Result<TwoStopList> {
    let service_ids = get_active_service_ids(&data.connection)?;

    let trip_ids = get_trip_ids_that_hit_stations(
        &data.connection,
        start_station_id,
        end_station_id,
        &service_ids,
    )?;

    let mut stmt = data.connection.prepare(
        "
        select distinct stop_name, station_id, departure_time, arrival_time, stop_times.trip_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        join stops on stop_times.stop_id = stops.stop_id
        where
            stop_times.trip_id = ? and
            station_id in (?, ?)
        ",
    )?;

    let start_station = load_station(&data.connection, start_station_id)?;
    let end_station = load_station(&data.connection, end_station_id)?;
    let now = Local::now().with_timezone(&FixedOffset::east(0));

    let mut trips = HashMap::new();

    for id in trip_ids {
        stmt.bind(1, id).wrap_err("failed to bind trip id")?;
        stmt.bind(2, start_station_id)
            .wrap_err("failed to bind start station id")?;
        stmt.bind(3, end_station_id)
            .wrap_err("failed to bind end station id")?;

        for stop in read_stops(&mut stmt)? {
            let station_name = match stop.station_id {
                x if x == start_station_id => "start",
                x if x == end_station_id => "end",
                _ => continue,
            };

            if stop.departure < now || stop.arrival < now {
                continue;
            }

            trips
                .entry(stop.trip_id)
                .or_insert_with(HashMap::new)
                .insert(station_name, stop);
        }

        stmt.reset()?;
    }

    let mut trips: Vec<_> = trips
        .into_iter()
        .filter_map(|(trip_id, mut stations)| {
            let start = stations.remove("start")?;
            let end = stations.remove("end")?;

            if start.arrival > end.departure {
                return None;
            }

            Some(TwoStop {
                trip_id,
                start,
                end,
            })
        })
        .collect();

    trips.sort_by(|a, b| a.start.departure.cmp(&b.start.departure));

    Ok(TwoStopList {
        trips,
        start: start_station,
        end: end_station,
    })
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

    let mut stmt = connection
        .prepare(format!(
            "
        select service_id
        from calendar
        where
            {} = 1 and ? >= start_date and ? <= end_date
        ",
            weekday
        ))
        .wrap_err("prepare service query")?;

    stmt.bind(1, today_num).wrap_err("bind today_num index 1")?;
    stmt.bind(2, today_num).wrap_err("bind today_num index 2")?;

    let mut ids = Vec::new();

    while let sqlite::State::Row = stmt.next().wrap_err("read row for service")? {
        ids.push(stmt.read(0).wrap_err("read column 0 for service_id")?);
    }

    Ok(ids)
}

pub fn load_station(connection: &sqlite::Connection, station_id: i64) -> Result<Station> {
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

    match stmt.next().wrap_err("read row for station")? {
        sqlite::State::Row => Ok(Station {
            name: stmt.read(0).wrap_err("read name")?,
            station_id: stmt.read::<i64>(1).wrap_err("read station_id")?,
        }),
        sqlite::State::Done => bail!("No station with ID {station_id} found"),
    }
}

fn get_trip_ids_that_hit_station(
    connection: &sqlite::Connection,
    station: i64,
    service_ids: &[String],
) -> Result<Vec<i64>> {
    let mut statement = connection.prepare(
        &format!("
        select trip_id from stop_times join stops on stops.stop_id =stop_times.stop_id where station_id=?
        intersect
        select trip_id from trips where service_id in ({})
        ",         bind_placeholders(service_ids.len())),
    )?;

    statement.bind(1, station)?;

    for (idx, i) in service_ids.iter().enumerate() {
        statement.bind(idx + 2, &sqlite::Value::String(i.clone()))?;
    }

    let mut trip_ids = Vec::new();

    while let Ok(sqlite::State::Row) = statement.next() {
        trip_ids.push(statement.read(0)?);
    }

    Ok(trip_ids)
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

fn bind_placeholders(num: usize) -> String {
    let mut placeholders = String::with_capacity(num * 2);

    for _ in 0..num {
        placeholders.push('?');
        placeholders.push(',');
    }

    placeholders.pop();

    placeholders
}
