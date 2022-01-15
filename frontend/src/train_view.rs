use crate::{time::now, time_display::TimeDisplay, trip_display::TrainId};
use chrono::Duration;
use train_schedules_common::{IndividualTrip, Time};
use yew::{function_component, html, use_state_eq, Properties};

use crate::context::host;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub train_id: i64,
}

#[function_component(TrainView)]
pub fn train_view(props: &Props) -> Html {
    let trip = use_state_eq(|| IndividualTrip {
        id: props.train_id,
        stations: Vec::new(),
    });
    let host = host();
    let train_id = props.train_id;

    crate::fetch::fetch(format!("{host}/api/trip?id={train_id}"), trip.clone());

    html! {
        <div>
            <h1><TrainId id={ props.train_id } /></h1>

            <ul>
            { for trip.stations.iter().map(|s| html!{
                <li class={ time_class(s.departure.departure) }>
                    <TimeDisplay time={ s.departure.departure } />
                    <a href={format!("/c/station/{}", s.id)}>
                        { &s.name }
                    </a>
                </li>
            }) }
            </ul>
        </div>
    }
}

fn time_class(time: Time) -> &'static str {
    if time
        .estimated
        .unwrap_or(time.scheduled)
        .signed_duration_since(now())
        > Duration::seconds(0)
    {
        ""
    } else {
        "TrainView--timePast"
    }
}
