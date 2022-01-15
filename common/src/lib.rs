use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

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
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Time {
    pub scheduled: DateTime<FixedOffset>,
    pub estimated: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TwoStop {
    pub trip_id: i64,
    pub start: Stop,
    pub end: Stop,
}

impl Time {
    pub fn new(scheduled: DateTime<FixedOffset>) -> Self {
        Self {
            scheduled,
            estimated: None,
        }
    }

    pub fn is_live(&self) -> bool {
        self.estimated.is_some()
    }
}

impl Deref for Time {
    type Target = DateTime<FixedOffset>;

    fn deref(&self) -> &Self::Target {
        match &self.estimated {
            Some(e) => e,
            None => &self.scheduled,
        }
    }
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
    pub arrival: Time,
    pub departure: Time,
}
