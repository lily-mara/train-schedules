use crate::{context::host, trip_display::TripDisplay};
use gloo::timers::callback::Interval;
use serde::Serialize;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, Serialize, PartialEq)]
pub struct TripListProps {
    pub start: i64,

    pub end: i64,
}

#[function_component(Model)]
pub fn trip_list(props: &TripListProps) -> Html {
    let trip_list = use_state_eq(TripList::default);
    let host = host();

    let url = format!(
        "{host}/api/upcoming-trips?start={}&end={}",
        props.start, props.end
    );

    crate::fetch::fetch(url.clone(), trip_list.clone());

    let trip_list_interval = trip_list.clone();
    let _interval = use_state(|| {
        Interval::new(60_000, move || {
            crate::fetch::fetch(url.clone(), trip_list_interval.clone());
        })
    });

    if trip_list.trips.is_empty() {
        return html! {
            <h1>{ "No trips found" }</h1>
        };
    }

    let flipped_url = format!(
        "/c/station/{}/{}",
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
