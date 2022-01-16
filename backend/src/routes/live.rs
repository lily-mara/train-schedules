use crate::{error::Error, routes::twostops::load_station, types, AppState, Result};
use actix_web::{client::Client, http::StatusCode, web, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, Local};
use log::debug;
use serde::Deserialize;
use train_schedules_common::Stop;

#[derive(Deserialize, Debug, Clone)]
pub struct Query {
    station_id: i64,
}

pub async fn live_station(
    query: web::Query<Query>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(
        get_station_live_status(
            &data.client,
            &data.api_key,
            &data.connection,
            query.station_id,
        )
        .await?,
    ))
}

async fn get_station_live_status(
    client: &Client,
    api_key: &str,
    connection: &sqlite::Connection,
    station_id: i64,
) -> Result<Vec<Stop>> {
    let station = load_station(connection, station_id)?;

    let url = format!(
        "https://api.511.org/transit/StopMonitoring?api_key={api_key}&agency=CT&format=json&stopCode={stop_code}",
        api_key=api_key,
        stop_code=station_id
    );

    let mut response = client.get(&url).send().await?;

    let body = response.body().await?;

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
        if let Some(vehicle_ref) = visit.MonitoredVehicleJourney.VehicleRef {
            if let Ok(trip_id) = vehicle_ref.parse() {
                trips.push(Stop {
                    station_id,
                    station_name: station.name.clone(),
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
                });
            }
        }
    }

    Ok(trips)
}

fn to_local_time(time: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    time.with_timezone(Local::now().offset())
}
