use std::collections::HashMap;

use chrono::prelude::*;
use chrono_tz::US::Pacific;
use eyre::{Context, Result};
use train_schedules_common::{Station, Stop};

#[derive(Clone, Debug)]
pub struct Service {
    pub start_date: Date<FixedOffset>,
    pub end_date: Date<FixedOffset>,
    pub id: String,
    pub weekdays: Vec<Weekday>,
}

pub fn all_stations(connection: &sqlite::Connection) -> Result<Vec<Station>> {
    let mut stmt = connection.prepare(
        "
            select station_id, stop_name, stop_code
            from stops
        ",
    )?;

    let mut stations = HashMap::new();

    while let sqlite::State::Row = stmt.next()? {
        let station_id = stmt.read(0)?;
        let name = stmt.read(1)?;
        let stop_code = stmt.read(2)?;

        stations
            .entry(station_id)
            .or_insert(Station {
                name,
                station_id,
                stop_codes: Vec::new(),
            })
            .stop_codes
            .push(stop_code);
    }

    Ok(stations.into_values().collect())
}

pub fn all_stops(connection: &sqlite::Connection) -> Result<Vec<Stop>> {
    let mut stmt = connection.prepare(
        "
        select distinct stop_name, station_id, departure_time, arrival_time, stop_times.trip_id, service_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        join stops on stop_times.stop_id = stops.stop_id
        ",
    )?;

    let mut stops = Vec::new();

    while let sqlite::State::Row = stmt.next().wrap_err("error reading row from sqlite")? {
        let station_name: String = stmt.read(0)?;

        let station_id: i64 = stmt.read(1)?;

        let departure_str: String = stmt.read(2)?;

        let departure = parse_time(&departure_str)?;

        let arrival_str: String = stmt.read(3)?;

        let arrival = parse_time(&arrival_str)?;

        let trip_id = stmt.read(4)?;

        let service_id = stmt.read(5)?;

        stops.push(Stop {
            trip_id,
            station_id,
            station_name,
            arrival,
            departure,
            service_id,
        });
    }

    Ok(stops)
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    let mut parts = time.split(':');

    let mut add_days = 0;
    let mut hour = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse hour part from time value {time}"))?;

    while hour >= 24 {
        hour -= 24;
        add_days += 1;
    }

    let minute = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse minute part from time value {time}"))?;
    let second = parts
        .next()
        .unwrap()
        .parse()
        .wrap_err_with(|| format!("failed to parse second part from time value {time}"))?;

    let time = Pacific
        .from_utc_datetime(&Utc::now().naive_utc())
        .date()
        .and_hms(hour, minute, second)
        + chrono::Duration::days(add_days);

    Ok(time.with_timezone(&FixedOffset::west(0)))
}

pub fn services(connection: &sqlite::Connection) -> Result<Vec<Service>> {
    // let today_num = (today.year() as i64 * 100 + today.month() as i64) * 100 + today.day() as i64;

    let mut stmt = connection
        .prepare(
            "
        select service_id, start_date, end_date, monday, tuesday, wednesday, thursday, friday, saturday, sunday
        from calendar
        ",
        )
        .wrap_err("prepare service query")?;

    let mut services = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        let id = stmt.read(0)?;
        let start_date = date_from_num(stmt.read(1)?);
        let end_date = date_from_num(stmt.read(2)?);

        let mut weekdays = Vec::new();

        if stmt.read::<i64>(3)? == 1 {
            weekdays.push(Weekday::Mon);
        }
        if stmt.read::<i64>(4)? == 1 {
            weekdays.push(Weekday::Tue);
        }
        if stmt.read::<i64>(5)? == 1 {
            weekdays.push(Weekday::Wed);
        }
        if stmt.read::<i64>(6)? == 1 {
            weekdays.push(Weekday::Thu);
        }
        if stmt.read::<i64>(7)? == 1 {
            weekdays.push(Weekday::Fri);
        }
        if stmt.read::<i64>(8)? == 1 {
            weekdays.push(Weekday::Sat);
        }
        if stmt.read::<i64>(9)? == 1 {
            weekdays.push(Weekday::Sun);
        }

        services.push(Service {
            weekdays,
            start_date,
            end_date,
            id,
        })
    }

    Ok(services)
}

fn date_from_num(x: i64) -> Date<FixedOffset> {
    let year = x / 10_000;
    let month = (x / 100) % 100;
    let day = x % 100;

    Pacific
        .ymd(year as i32, month as u32, day as u32)
        .with_timezone(&FixedOffset::west(0))
}
