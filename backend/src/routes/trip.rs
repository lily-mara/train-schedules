use std::sync::Arc;

use crate::State;
use axum::{
    extract::{Extension, Query},
    Json,
};
use serde::Deserialize;
use train_schedules_common::Trip;

#[derive(Deserialize, Debug, Clone)]
pub struct TripQuery {
    id: i64,
}

pub async fn trip(
    Query(query): Query<TripQuery>,
    Extension(data): Extension<Arc<State>>,
) -> Json<Trip> {
    let stops = data
        .stops
        .iter()
        .filter(|s| s.trip_id == query.id)
        .cloned()
        .collect();

    Json(Trip {
        trip_id: query.id,
        stops,
    })
}
