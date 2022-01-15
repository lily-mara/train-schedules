use crate::context::host;
use crate::twostop::Twostop;
use gloo::timers::callback::Interval;
use serde::Serialize;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, Serialize, PartialEq)]
pub struct TwostopListProps {
    pub start: i64,

    pub end: i64,
}

#[function_component(Model)]
pub fn view(props: &TwostopListProps) -> Html {
    let twostops = use_state_eq(TwoStopList::default);
    let host = host();

    let url = format!(
        "{host}/api/upcoming-trips?start={}&end={}",
        props.start, props.end
    );

    crate::fetch::fetch(url.clone(), twostops.clone());

    let trip_list_interval = twostops.clone();
    let _interval = use_state(|| {
        Interval::new(60_000, move || {
            crate::fetch::fetch(url.clone(), trip_list_interval.clone());
        })
    });

    if twostops.trips.is_empty() {
        return html! {
            <h1>{ "No trips found" }</h1>
        };
    }

    let flipped_url = format!(
        "/c/station/{}/{}",
        twostops.end.station_id, twostops.start.station_id
    );

    html! {
        <div class="TripList">
            <h1>{ "Upcoming trains" }</h1>
            <h2>{ format!("{} → {}", twostops.start.name, twostops.end.name) } </h2>
            <h3>
                <a classes="DirectionFlip" href={flipped_url}>
                    {"Change Direction"}
                </a>
            </h3>
            { for twostops.trips.iter().map(|twostop| html! {
                <Twostop twostop={twostop.clone()} />
            })}
        </div>
    }
}
