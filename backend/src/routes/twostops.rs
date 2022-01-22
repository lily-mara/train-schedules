use std::collections::HashMap;

use crate::{error::Error, read_stops, AppState, Result};
use actix_web::{web, HttpResponse};
use chrono::{Datelike, FixedOffset, Local, Weekday};
use serde::Deserialize;
use train_schedules_common::{Station, TwoStop, TwoStopList};

#[derive(Deserialize, Debug, Clone)]
pub struct UpcomingTripsQuery {
    start: i64,
    end: i64,
}

pub async fn upcoming_trips(
    query: web::Query<UpcomingTripsQuery>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let twostops = get_twostops(data, query.start, query.end).await?;

    Ok(HttpResponse::Ok().json(twostops))
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
        stmt.bind(1, id)?;
        stmt.bind(2, start_station_id)?;
        stmt.bind(3, end_station_id)?;

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

    match stmt.next()? {
        sqlite::State::Row => Ok(Station {
            name: stmt.read(0)?,
            station_id: stmt.read::<i64>(1)?,
        }),
        sqlite::State::Done => Err(Error::NoSuchStation(station_id)),
    }
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