use std::{sync::Arc, time::Duration};

use crate::{
    error::HttpResult,
    types::{self, MonitoredStopVisit},
    State,
};
use axum::{extract::Extension, Json};
use chrono::{DateTime, FixedOffset, Local};
use eyre::Result;
use eyre::{bail, Context};
use reqwest::StatusCode;
use tracing::{debug, info};
use train_schedules_common::{Station, Stop};

pub async fn live_station(Extension(data): Extension<Arc<State>>) -> HttpResult<Vec<Stop>> {
    Ok(Json(get_station_live_status(&data).await?))
}

async fn get_station_live_status(data: &State) -> Result<Vec<Stop>> {
    if let Some(cached) = data.live_status_cache.read().await.get(&()).cloned() {
        return Ok(cached);
    }

    let mut lock = data.live_status_cache.write().await;
    // Check the cache again in case some other coroutine wrote while we were
    // waiting to take the lock
    if let Some(cached) = lock.get(&()).cloned() {
        return Ok(cached);
    }

    let api_key = &data.api_key;
    let url = format!(
        "https://api.511.org/transit/StopMonitoring?api_key={api_key}&agency=CT&format=json"
    );

    let response = data.client.get(&url).send().await?;

    let status = response.status();

    let body = response.text().await?;

    if status == StatusCode::TOO_MANY_REQUESTS {
        info!("Recieved HTTP 429 from 511.org API - bypassing requests for next minute");
        lock.insert((), Vec::new(), Duration::from_secs(60));

        return Ok(Vec::new());
    }

    if status != StatusCode::OK {
        bail!("Received HTTP {status} from 511.org API: {body}");
    }

    let resp: types::ApiResponse = serde_json::from_str(&body[3..])
        .wrap_err_with(|| format!("failed to parse 511.org API response as json. body: {body}"))?;

    debug!("Parsed API response: {:?}", resp);
    let mut trips = Vec::new();

    for visit in resp
        .ServiceDelivery
        .StopMonitoringDelivery
        .MonitoredStopVisit
    {
        if let Some(stop) = try_find_stop(visit, &data.stations) {
            trips.push(stop);
        }
    }

    lock.insert((), trips.clone(), Duration::from_secs(120));

    Ok(trips)
}

fn try_find_stop(visit: MonitoredStopVisit, stations: &[Station]) -> Option<Stop> {
    let vehicle_ref = visit.MonitoredVehicleJourney.VehicleRef?;
    let trip_id = vehicle_ref.parse().ok()?;

    let stopcode = visit
        .MonitoredVehicleJourney
        .MonitoredCall
        .StopPointRef
        .parse()
        .ok()?;

    let station = stations
        .iter()
        .find(|s| s.stop_codes.contains(&stopcode))?
        .clone();

    Some(Stop {
        // TODO: find the service ID here
        service_id: String::new(),
        station_name: station.name.clone(),
        station_id: station.station_id,
        trip_id,
        arrival: to_local_time(
            visit
                .MonitoredVehicleJourney
                .MonitoredCall
                .ExpectedArrivalTime?,
        ),
        departure: to_local_time(
            visit
                .MonitoredVehicleJourney
                .MonitoredCall
                .ExpectedDepartureTime?,
        ),
    })
}

fn to_local_time(time: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    time.with_timezone(Local::now().offset())
}
