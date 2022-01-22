use std::time::Duration;

use gloo::timers::callback::Interval;
use train_schedules_common::Stop;
use yew::{use_state_eq, UseStateHandle};

use crate::fetch::fetch_repeating_interval;

pub fn live_status(host: &str, station_id: i64) -> LiveStatus {
    let stops = use_state_eq(Vec::new);

    let _interval = fetch_repeating_interval(
        format!("{host}/api/stations/live?station_id={station_id}"),
        stops.clone(),
        Duration::from_secs(60),
    );

    LiveStatus { stops, _interval }
}

pub struct LiveStatus {
    stops: UseStateHandle<Vec<Stop>>,
    _interval: UseStateHandle<Interval>,
}

impl LiveStatus {
    pub fn trip(&self, trip_id: i64) -> Option<Stop> {
        self.stops.iter().find(|s| s.trip_id == trip_id).cloned()
    }

    pub fn all(&self) -> &[Stop] {
        &self.stops
    }
}
