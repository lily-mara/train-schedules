use crate::error::{Error, Result};
use actix_web::{client::Client, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{prelude::*, Weekday};
use futures::{future, Future};
use serde::Deserialize;
use std::{collections::HashMap, env};
use train_schedules_common::*;

mod error;
mod types;

#[derive(Deserialize, Debug, Clone)]
struct UpcomingTripsQuery {
    start: i64,
    end: i64,
}

struct AppState {
    connection: sqlite::Connection,
    client: Client,
    api_key: String,
}

fn main() {
    let sys = actix_rt::System::new("example");

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "schedules.db".to_owned());
    let api_key = env::var("API_KEY").expect("API_KEY environment variable is required");

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                connection: sqlite::Connection::open(&db_path).unwrap(),
                api_key: api_key.clone(),
                client: Client::new(),
            })
            .route("/", web::get().to(index))
            .route("/stations", web::get().to(stations))
            .route("/upcoming-trips", web::get().to_async(upcoming_trips))
    })
    .bind("0.0.0.0:8088")
    .unwrap()
    .start();

    eprintln!("Listening on 0.0.0.0:8088");

    let _ = sys.run();
}

fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}

fn get_trip_ids_that_hit_stations(
    connection: &sqlite::Connection,
    start_station: i64,
    end_station: i64,
) -> Result<Vec<i64>> {
    let mut statement = connection.prepare(
        "
        select trip_id from stop_times where stop_id=?
        intersect
        select trip_id from stop_times where stop_id=?
        ",
    )?;

    statement.bind(1, start_station)?;
    statement.bind(2, end_station)?;

    let mut trip_ids = Vec::new();

    while let Ok(sqlite::State::Row) = statement.next() {
        trip_ids.push(statement.read(0)?);
    }

    Ok(trip_ids)
}

fn load_station(connection: &sqlite::Connection, station_id: i64) -> Result<Station> {
    let mut stmt = connection.prepare(
        "
            select distinct stop_name, stops.stop_id, direction_id
            from trips
            join stop_times on stop_times.trip_id = trips.trip_id
            join stops on stops.stop_id = stop_times.stop_id
            where stops.stop_id=?;
        ",
    )?;

    stmt.bind(1, station_id)?;

    match stmt.next()? {
        sqlite::State::Row => Ok(Station {
            name: stmt.read(0)?,
            station_id: stmt.read(1)?,
            direction: stmt.read::<i64>(2)?.into(),
        }),
        sqlite::State::Done => Err(Error::NoSuchStation(station_id)),
    }
}

fn load_all_stations(connection: &sqlite::Connection) -> Result<Vec<Station>> {
    let mut stmt = connection.prepare(
        "

            select distinct stop_name, stops.stop_id, direction_id
            from trips
            join stop_times on stop_times.trip_id = trips.trip_id
            join stops on stops.stop_id = stop_times.stop_id
            order by stops.stop_id asc

        ",
    )?;

    let mut stations = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        stations.push(Station {
            name: stmt.read(0)?,
            station_id: stmt.read(1)?,
            direction: stmt.read::<i64>(2)?.into(),
        });
    }

    Ok(stations)
}

fn get_upcoming_trips(
    connection: &sqlite::Connection,
    start_station_id: i64,
    end_station_id: i64,
) -> Result<TripList> {
    let trip_ids = get_trip_ids_that_hit_stations(connection, start_station_id, end_station_id)?;

    let service_ids = get_active_service_ids(connection)?;

    let mut stmt = connection.prepare(format!(
        "
        select departure_time, arrival_time, stop_times.trip_id, stop_id
        from stop_times
        join trips on trips.trip_id=stop_times.trip_id
        where (stop_id=? or stop_id=?) and stop_times.trip_id=? and service_id in ({})
        order by stop_times.trip_id
        ",
        bind_placeholders(service_ids.len()),
    ))?;

    let start_station = load_station(connection, start_station_id)?;
    let end_station = load_station(connection, end_station_id)?;
    let now = Local::now().with_timezone(&FixedOffset::east(0));

    let mut trips = HashMap::new();

    for id in trip_ids {
        stmt.bind(1, start_station_id)?;
        stmt.bind(2, end_station_id)?;
        stmt.bind(3, id)?;
        for (idx, i) in service_ids.iter().enumerate() {
            stmt.bind(idx + 4, &sqlite::Value::String(i.clone()))?;
        }

        while let sqlite::State::Row = stmt.next()? {
            let station_name = match stmt.read::<i64>(3)? {
                x if x == start_station_id => "start",
                x if x == end_station_id => "end",
                x => panic!("Got unexpected station ID: {}", x),
            };

            let trip_id = stmt.read(2)?;

            let departure_str: String = stmt.read(0)?;
            let departure = parse_time(&departure_str)?;

            let arrival_str: String = stmt.read(1)?;
            let arrival = parse_time(&arrival_str)?;

            if departure < now || arrival < now {
                continue;
            }

            trips.entry(trip_id).or_insert_with(HashMap::new).insert(
                station_name,
                Departure {
                    departure: Time::new(departure),
                    arrival: Time::new(departure),
                },
            );
        }
        stmt.reset()?;
    }

    let mut trips: Vec<_> = trips
        .into_iter()
        .filter_map(|(trip_id, mut stations)| {
            let start = stations.remove("start")?;
            let end = stations.remove("end")?;

            Some(Trip {
                trip_id,
                start,
                end,
            })
        })
        .collect();

    trips.sort_by(|a, b| a.start.departure.cmp(&b.start.departure));

    Ok(TripList {
        trips,
        start: start_station,
        end: end_station,
    })
}

fn parse_time(time: &str) -> Result<DateTime<FixedOffset>> {
    let mut parts = time.split(':');

    let mut add_days = 0;
    let mut hour = parts.next().unwrap().parse()?;
    while hour >= 24 {
        hour -= 24;
        add_days += 1;
    }

    let minute = parts.next().unwrap().parse()?;
    let second = parts.next().unwrap().parse()?;

    let time = Local::today().and_hms(hour, minute, second) + chrono::Duration::days(add_days);

    Ok(time.with_timezone(&time.offset()))
}

fn to_local_time(time: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    time.with_timezone(&Local::now().offset())
}

fn get_station_estimated_stuff(
    client: Client,
    api_key: String,
    station_id: i64,
) -> impl Future<
    Item = (
        Client,
        HashMap<i64, (DateTime<FixedOffset>, DateTime<FixedOffset>)>,
    ),
    Error = Error,
> {
    let url = format!(
        "https://api.511.org/transit/StopMonitoring?api_key={api_key}&agency=CT&format=json&stopCode={stop_code}",
        api_key=api_key,
        stop_code=station_id
    );

    let resp = client
        .get(&url)
        .send()
        .map_err(Error::from)
        .and_then(|mut response| response.body().map_err(Error::from))
        .and_then(|body| match serde_json::from_slice(&body[3..]) {
            Ok(data) => future::ok(data),
            Err(e) => future::err(e.into()),
        })
        .map(|resp: types::ApiResponse| {
            let mut trips = HashMap::new();

            for visit in resp
                .ServiceDelivery
                .StopMonitoringDelivery
                .MonitoredStopVisit
            {
                if let Some(vehicle_ref) = visit.MonitoredVehicleJourney.VehicleRef {
                    if let Ok(trip_id) = vehicle_ref.parse() {
                        trips.insert(
                            trip_id,
                            (
                                to_local_time(
                                    visit
                                        .MonitoredVehicleJourney
                                        .MonitoredCall
                                        .ExpectedDepartureTime,
                                ),
                                to_local_time(
                                    visit
                                        .MonitoredVehicleJourney
                                        .MonitoredCall
                                        .ExpectedArrivalTime,
                                ),
                            ),
                        );
                    }
                }
            }

            trips
        });

    resp.map(|trips| (client, trips))
}

fn add_live_status(
    client: Client,
    api_key: &str,
    mut trips: TripList,
) -> impl Future<Item = TripList, Error = Error> {
    let key = api_key.to_owned();

    let start = trips.start.station_id;
    let end = trips.end.station_id;

    let trips_inner = trips.clone();

    get_station_estimated_stuff(client, api_key.into(), start)
        .and_then(move |(client, start_data)| {
            get_station_estimated_stuff(client, key, end)
                .map(move |(_, end_data)| (start_data, end_data))
        })
        .map(move |(start_data, end_data)| {
            for trip in &mut trips.trips {
                if let Some((departure, arrival)) = start_data.get(&trip.trip_id) {
                    trip.start.departure.estimated = Some(*departure);
                    trip.start.arrival.estimated = Some(*arrival);
                }

                if let Some((departure, arrival)) = end_data.get(&trip.trip_id) {
                    trip.end.departure.estimated = Some(*departure);
                    trip.end.arrival.estimated = Some(*arrival);
                }
            }

            trips
        })
        .or_else(|e| {
            eprintln!("Error fetching realtime data: {}", e);

            future::ok(trips_inner)
        })
}

fn get_active_service_ids(connection: &sqlite::Connection) -> Result<Vec<String>> {
    let today = Local::now();
    let weekday = match today.weekday() {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
    };

    let today_num = (today.year() as i64 * 100 + today.month() as i64) * 100 + today.day() as i64;

    let mut stmt = connection.prepare(format!(
        "
        select service_id
        from calendar
        where
            {} = 1 and ? >= start_date and ? <= end_date
        ",
        weekday
    ))?;

    stmt.bind(1, today_num)?;
    stmt.bind(2, today_num)?;

    let mut ids = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        ids.push(stmt.read(0)?);
    }

    Ok(ids)
}

fn upcoming_trips(
    query: web::Query<UpcomingTripsQuery>,
    data: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let trips = match get_upcoming_trips(&data.connection, query.start, query.end) {
        Ok(trips) => trips,
        Err(e) => return future::Either::A(future::err(e)),
    };

    future::Either::B(
        add_live_status(data.client.clone(), &data.api_key, trips)
            .map(|trips| HttpResponse::Ok().json(trips)),
    )
}

fn stations(_req: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(load_all_stations(&data.connection)?))
}

fn bind_placeholders(num: usize) -> String {
    let mut placeholders = String::with_capacity(num * 2);

    for _ in 0..num {
        placeholders.push('?');
        placeholders.push(',');
    }

    placeholders.pop();

    placeholders
}
