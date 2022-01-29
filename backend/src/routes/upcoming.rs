use std::{collections::HashMap, sync::Arc};

use crate::{db::Service, error::HttpResult, State};
use axum::{
    extract::{Extension, Query},
    Json,
};
use chrono::{Datelike, FixedOffset, Local};
use eyre::Result;
use serde::{Deserialize, Serialize};
use train_schedules_common::{Station, Stop, TwoStop, TwoStopList};

#[derive(Deserialize, Debug, Clone)]
pub struct UpcomingTripsQuery {
    start: i64,
    end: Option<i64>,
}

fn as_json_value<T>(x: &T) -> Result<serde_json::Value>
where
    T: Serialize,
{
    let str = serde_json::to_string(x)?;
    let value = serde_json::from_str(&str)?;

    Ok(value)
}

pub async fn upcoming_trips(
    Query(query): Query<UpcomingTripsQuery>,
    Extension(data): Extension<Arc<State>>,
) -> HttpResult<serde_json::Value> {
    match query.end {
        Some(end) => Ok(Json(as_json_value(&get_twostops(
            &*data,
            query.start,
            end,
        )?)?)),
        None => Ok(Json(as_json_value(&get_upcoming(&*data, query.start))?)),
    }
}

fn get_upcoming(data: &State, station_id: i64) -> Vec<Stop> {
    let services = active_service_ids(&data.services);

    let now = Local::now().with_timezone(&FixedOffset::east(0));

    let mut stops = data
        .stops
        .iter()
        .filter(|s| {
            s.station_id == station_id && s.departure > now && services.contains(&s.service_id)
        })
        .cloned()
        .collect::<Vec<_>>();

    stops.sort_by(|a, b| a.departure.cmp(&b.departure));

    stops
}

fn station(id: i64, stations: &[Station]) -> Result<Station> {
    stations
        .iter()
        .find(|s| s.station_id == id)
        .cloned()
        .ok_or_else(|| eyre::eyre!("no station found with id {id}"))
}

fn get_twostops(data: &State, start_station_id: i64, end_station_id: i64) -> Result<TwoStopList> {
    let services = active_service_ids(&data.services);

    let trips = twostops(&data.stops, start_station_id, end_station_id, &services);

    let start_station = station(start_station_id, &data.stations)?;
    let end_station = station(end_station_id, &data.stations)?;

    Ok(TwoStopList {
        trips,
        start: start_station,
        end: end_station,
    })
}

fn active_service_ids(services: &[Service]) -> Vec<String> {
    let today = Local::today().with_timezone(&FixedOffset::east(0));

    services
        .iter()
        .filter(|s| {
            s.start_date < today && today < s.end_date && s.weekdays.contains(&today.weekday())
        })
        .map(|s| s.id.clone())
        .collect()
}

fn twostops(
    stops: &[Stop],
    start_station: i64,
    end_station: i64,
    services: &[String],
) -> Vec<TwoStop> {
    let mut trips = HashMap::new();

    for stop in stops {
        if stop.station_id != start_station && stop.station_id != end_station {
            continue;
        }

        if !services.contains(&stop.service_id) {
            continue;
        }

        trips
            .entry(stop.trip_id)
            .or_insert(Vec::new())
            .push(stop.clone());
    }

    let mut stops = Vec::new();

    for (trip_id, mut trip) in trips {
        if trip.len() != 2 {
            continue;
        }

        trip.sort_by(|a, b| a.departure.cmp(&b.departure));

        let end = trip.pop().unwrap();
        let start = trip.pop().unwrap();

        stops.push(TwoStop {
            trip_id,
            start,
            end,
        });
    }

    stops.sort_by(|a, b| a.start.departure.cmp(&b.start.departure));

    stops
}
