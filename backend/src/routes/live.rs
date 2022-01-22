use std::{collections::HashMap, time::Duration};

use crate::{
    error::Error,
    types::{self, MonitoredStopVisit},
    AppState, LiveStatusCache, Result,
};
use actix_web::{client::Client, http::StatusCode, web, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, Local};
use log::{debug, info};
use train_schedules_common::{Station, Stop};

pub async fn live_station(data: web::Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        get_station_live_status(
            &data.client,
            &data.api_key,
            &data.live_status_cache,
            &data.connection,
        )
        .await?,
    ))
}

pub fn stopcode_station_mappings(connection: &sqlite::Connection) -> Result<HashMap<i64, Station>> {
    let mut stmt = connection.prepare(
        "
            select stop_code, stop_name, station_id
            from stops
        ",
    )?;

    let mut mapping = HashMap::new();

    while let sqlite::State::Row = stmt.next()? {
        mapping.insert(
            stmt.read(0)?,
            Station {
                name: stmt.read(1)?,
                station_id: stmt.read(2)?,
            },
        );
    }

    Ok(mapping)
}

async fn get_station_live_status(
    client: &Client,
    api_key: &str,
    cache: &LiveStatusCache,
    connection: &sqlite::Connection,
) -> Result<Vec<Stop>> {
    if let Some(cached) = cache.read().await.get(&()).cloned() {
        return Ok(cached);
    }

    let mut lock = cache.write().await;
    // Check the cache again in case some other coroutine wrote while we were
    // waiting to take the lock
    if let Some(cached) = lock.get(&()).cloned() {
        return Ok(cached);
    }

    let stopcode_mappings = stopcode_station_mappings(connection)?;

    let url = format!(
        "https://api.511.org/transit/StopMonitoring?api_key={api_key}&agency=CT&format=json"
    );

    let mut response = client.get(&url).send().await?;

    let body = response.body().await?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        info!("Recieved HTTP 429 from 511.org API - bypassing requests for next minute");
        lock.insert((), Vec::new(), Duration::from_secs(60));

        return Ok(Vec::new());
    }

    if response.status() != StatusCode::OK {
        let body = String::from_utf8_lossy(&body).into_owned();
        return Err(Error::FiveOneOneServer {
            code: response.status(),
            body,
        });
    }

    let resp: types::ApiResponse = serde_json::from_slice(&body[3..])?;

    debug!("Parsed API response: {:?}", resp);
    let mut trips = Vec::new();

    for visit in resp
        .ServiceDelivery
        .StopMonitoringDelivery
        .MonitoredStopVisit
    {
        if let Some(stop) = try_find_stop(visit, &stopcode_mappings) {
            trips.push(stop);
        }
    }

    lock.insert((), trips.clone(), Duration::from_secs(120));

    Ok(trips)
}

fn try_find_stop(
    visit: MonitoredStopVisit,
    stopcode_mappings: &HashMap<i64, Station>,
) -> Option<Stop> {
    let vehicle_ref = visit.MonitoredVehicleJourney.VehicleRef?;
    let trip_id = vehicle_ref.parse().ok()?;

    let stopcode = visit
        .MonitoredVehicleJourney
        .MonitoredCall
        .StopPointRef
        .parse()
        .ok()?;

    let station = stopcode_mappings.get(&stopcode)?;

    Some(Stop {
        station_name: station.name.clone(),
        station_id: station.station_id,
        trip_id,
        arrival: to_local_time(
            visit
                .MonitoredVehicleJourney
                .MonitoredCall
                .ExpectedArrivalTime,
        ),

        departure: to_local_time(
            visit
                .MonitoredVehicleJourney
                .MonitoredCall
                .ExpectedDepartureTime,
        ),
    })
}

fn to_local_time(time: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    time.with_timezone(Local::now().offset())
}
