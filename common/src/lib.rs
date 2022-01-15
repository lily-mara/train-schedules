use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct TripList {
    pub start: Station,
    pub end: Station,
    pub trips: Vec<Trip>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Station {
    pub name: String,
    pub station_id: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Departure {
    pub departure: Time,
    pub arrival: Time,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Time {
    pub scheduled: DateTime<FixedOffset>,
    pub estimated: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Trip {
    pub trip_id: i64,
    pub start: Departure,
    pub end: Departure,
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
pub struct IndividualTrip {
    pub id: i64,
    pub stations: Vec<IndividualStation>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct IndividualStation {
    pub id: i64,
    pub name: String,
    pub departure: Departure,
}
