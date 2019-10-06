use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TripList {
    pub start: Station,
    pub end: Station,
    pub trips: Vec<Trip>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Station {
    pub name: String,
    pub station_id: i64,
    pub direction: Direction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Direction {
    #[serde(rename = "north")]
    North,

    #[serde(rename = "south")]
    South,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::North
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Departure {
    pub departure: Time,
    pub arrival: Time,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Time {
    pub scheduled: DateTime<FixedOffset>,
    pub estimated: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trip {
    pub trip_id: i64,
    pub start: Departure,
    pub end: Departure,
}

impl From<i64> for Direction {
    fn from(f: i64) -> Direction {
        match f {
            0 => Direction::North,
            1 => Direction::South,
            _ => panic!("{} is not a valid direction", f),
        }
    }
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
