use std::sync::Arc;

use crate::AppState;
use serde::Deserialize;
use train_schedules_common::Trip;
use warp::reply::Json;

#[derive(Deserialize, Debug, Clone)]
pub struct TripQuery {
    id: i64,
}

pub fn trip(query: TripQuery, data: Arc<AppState>) -> Json {
    let stops = data
        .stops
        .iter()
        .filter(|s| s.trip_id == query.id)
        .cloned()
        .collect();

    warp::reply::json(&Trip {
        trip_id: query.id,
        stops,
    })
}
