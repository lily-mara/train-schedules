use crate::{time::now, time_display::TimeDisplay, twostop::TripId};
use chrono::{DateTime, Duration, FixedOffset};
use train_schedules_common::Trip;
use yew::{function_component, html, use_state_eq, Properties};

use crate::context::host;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub trip_id: i64,
}

#[function_component(TripView)]
pub fn train_view(props: &Props) -> Html {
    let _refresher = crate::timer::refresh_periodically(std::time::Duration::from_secs(30));
    let trip = use_state_eq(|| Trip {
        trip_id: props.trip_id,
        stops: Vec::new(),
    });
    let host = host();
    let trip_id = props.trip_id;

    crate::fetch::fetch(format!("{host}/api/trip?id={trip_id}"), trip.clone());

    html! {
        <div class="TripView">
            <h1><TripId id={ props.trip_id } /></h1>

            <ul>
            { for trip.stops.iter().map(|s| html!{
                <li class={ time_class(s.departure) }>
                    <TimeDisplay scheduled={ s.departure } />
                    <div class="TripView-box"></div>
                    <a href={format!("/c/station/{}", s.station_id)}>
                        { &s.station_name }
                    </a>
                </li>
            }) }
            </ul>
        </div>
    }
}

fn time_class(time: DateTime<FixedOffset>) -> &'static str {
    if time.signed_duration_since(now()) > Duration::seconds(0) {
        ""
    } else {
        "TrainView--timePast"
    }
}
