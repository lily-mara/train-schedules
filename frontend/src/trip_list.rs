use crate::{context::host, trip_display::TripDisplay};
use anyhow::Result;
use gloo::timers::callback::Interval;
use log::error;
use serde::Serialize;
use train_schedules_common::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, Clone, Serialize, PartialEq)]
pub struct TripListProps {
    pub start: i32,

    pub end: i32,
}

async fn fetch_trip_list(host: &str, props: &TripListProps) -> Result<TripList> {
    let url = format!(
        "{host}/api/upcoming-trips?start={}&end={}",
        props.start, props.end
    );

    let response = reqwest::get(url).await?;

    let body = response.text().await?;
    let list = serde_json::from_str(&body)?;

    Ok(list)
}

#[function_component(Model)]
pub fn trip_list(props: &TripListProps) -> Html {
    let trip_list = use_state_eq(|| TripList::default());
    let host = host();

    let trip_list_for_interval = trip_list.clone();
    let props_for_interval = props.clone();
    let host_for_initial = host.clone();

    let trip_list_for_initial = trip_list.clone();
    let props_for_initial = props.clone();

    spawn_local(async move {
        match fetch_trip_list(&host_for_initial, &props_for_initial).await {
            Ok(list) => trip_list_for_initial.set(list),
            Err(e) => error!("failed to fetch trip list {}", e),
        }
    });

    let _interval = use_state(|| {
        Interval::new(60_000, move || {
            let props_for_interval = props_for_interval.clone();
            let trip_list_for_interval = trip_list_for_interval.clone();
            let host = host.clone();

            spawn_local(async move {
                match fetch_trip_list(&host, &props_for_interval).await {
                    Ok(list) => trip_list_for_interval.set(list),
                    Err(e) => error!("failed to fetch trip list {}", e),
                }
            });
        })
    });

    if trip_list.trips.is_empty() {
        return html! {
            <h1>{ "No trips found" }</h1>
        };
    }

    let flipped_url = format!(
        "/c/{}/{}",
        trip_list.end.station_id, trip_list.start.station_id
    );

    html! {
        <div class="TripList">
            <h1>{ "Upcoming trains" }</h1>
            <h2>{ format!("{} → {}", trip_list.start.name, trip_list.end.name) } </h2>
            <h3>
                <a classes="DirectionFlip" href={flipped_url}>
                    {"Change Direction"}
                </a>
            </h3>
            { for trip_list.trips.iter().map(view_trip) }
        </div>
    }
}

fn view_trip(trip: &Trip) -> Html {
    let trip = trip.clone();

    html! {
        <TripDisplay {trip}></TripDisplay>
    }
}
