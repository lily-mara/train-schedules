#![allow(non_snake_case)]

use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub ServiceDelivery: ServiceDelivery,
}

#[derive(Deserialize, Debug)]
pub struct ServiceDelivery {
    pub StopMonitoringDelivery: StopMonitoringDelivery,
}

#[derive(Deserialize, Debug)]
pub struct StopMonitoringDelivery {
    pub MonitoredStopVisit: Vec<MonitoredStopVisit>,
}

#[derive(Deserialize, Debug)]
pub struct MonitoredStopVisit {
    pub MonitoredVehicleJourney: MonitoredVehicleJourney,
}

#[derive(Deserialize, Debug)]
pub struct MonitoredVehicleJourney {
    pub VehicleRef: Option<String>,
    pub MonitoredCall: MonitoredCall,
}

#[derive(Deserialize, Debug)]
pub struct MonitoredCall {
    pub ExpectedArrivalTime: DateTime<FixedOffset>,
    pub ExpectedDepartureTime: DateTime<FixedOffset>,
}
