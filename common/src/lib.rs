use chrono::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub departure: DateTime<FixedOffset>,
    pub arrival: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Times {
    pub scheduled: Departure,
    pub estimated: Option<EstimatedDeparture>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EstimatedDeparture {
    pub departure: Departure,
    pub last_updated: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trip {
    pub trip_id: i64,
    pub start: Times,
    pub end: Times,
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

pub fn time_str(minute: i64) -> String {
    let hour = minute / 60;
    let min = minute % 60;

    format!("{:02}:{:02}", hour, min)
}
