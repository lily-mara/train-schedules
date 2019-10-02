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
    pub departure_minute: i64,
    pub arrival_minute: i64,
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

impl Departure {
    fn departure_time(&self) -> String {
        time_str(self.departure_minute)
    }

    fn arrival_time(&self) -> String {
        time_str(self.arrival_minute)
    }
}

pub fn time_str(minute: i64) -> String {
    let hour = minute / 60;
    let min = minute % 60;

    format!("{:02}:{:02}", hour, min)
}
