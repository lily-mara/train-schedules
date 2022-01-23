use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct TwoStopList {
    pub start: Station,
    pub end: Station,
    pub trips: Vec<TwoStop>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Station {
    pub name: String,
    pub station_id: i64,
    pub stop_codes: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TwoStop {
    pub trip_id: i64,
    pub start: Stop,
    pub end: Stop,
}

pub fn time_str(minute: i64) -> String {
    let hour = minute / 60;
    let min = minute % 60;

    format!("{:02}:{:02}", hour, min)
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Trip {
    pub trip_id: i64,
    pub stops: Vec<Stop>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Stop {
    pub station_id: i64,
    pub trip_id: i64,
    pub station_name: String,
    pub arrival: DateTime<FixedOffset>,
    pub departure: DateTime<FixedOffset>,
    pub service_id: String,
}
