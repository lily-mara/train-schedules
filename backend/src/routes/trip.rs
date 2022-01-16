use crate::{read_stops, AppState, Result};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use train_schedules_common::Trip;

#[derive(Deserialize, Debug, Clone)]
pub struct TripQuery {
    id: i64,
}

pub async fn trip(query: web::Query<TripQuery>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let trip = get_trip(&data.connection, query.id)?;

    Ok(HttpResponse::Ok().json(trip))
}

fn get_trip(connection: &sqlite::Connection, trip_id: i64) -> Result<Trip> {
    let mut stmt = connection.prepare(
        "
        select distinct stop_name, station_id, departure_time, arrival_time, stop_times.trip_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        join stops on stop_times.stop_id = stops.stop_id
        where
            stop_times.trip_id = ?
        ",
    )?;

    stmt.bind(1, trip_id)?;

    let stops = read_stops(&mut stmt)?;

    Ok(Trip { trip_id, stops })
}
