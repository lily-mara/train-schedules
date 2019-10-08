#![allow(non_snake_case)]

use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiResponse {
    pub ServiceDelivery: ServiceDelivery,
}

#[derive(Deserialize)]
pub struct ServiceDelivery {
    pub StopMonitoringDelivery: StopMonitoringDelivery,
}

#[derive(Deserialize)]
pub struct StopMonitoringDelivery {
    pub MonitoredStopVisit: Vec<MonitoredStopVisit>,
}

#[derive(Deserialize)]
pub struct MonitoredStopVisit {
    pub MonitoredVehicleJourney: MonitoredVehicleJourney,
}

#[derive(Deserialize)]
pub struct MonitoredVehicleJourney {
    pub VehicleRef: Option<String>,
    pub MonitoredCall: MonitoredCall,
}

#[derive(Deserialize)]
pub struct MonitoredCall {
    pub ExpectedArrivalTime: DateTime<FixedOffset>,
    pub ExpectedDepartureTime: DateTime<FixedOffset>,
}
