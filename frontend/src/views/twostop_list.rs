use crate::context::host;
use crate::live_status::live_status;
use crate::views::twostop::Twostop;
use log::info;
use serde::Serialize;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, Serialize, PartialEq, Debug)]
pub struct TwostopListProps {
    pub start: i64,

    pub end: i64,
}

#[function_component(TwostopList)]
pub fn view(props: &TwostopListProps) -> Html {
    let twostops = use_state_eq(TwoStopList::default);
    let host = host();

    let live = live_status(&host);

    crate::fetch::fetch_once(
        format!(
            "{host}/api/upcoming-trips?start={}&end={}",
            props.start, props.end
        ),
        twostops.clone(),
    );

    // TODO: hide twostops that already completed with some kind of time filtering and interval

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
            { for twostops.trips.iter().map(|twostop| {
                let twostop = twostop.clone();
                let start_live = live.get(twostop.start.station_id, twostop.trip_id);
                let end_live = live.get(twostop.end.station_id, twostop.trip_id);

                html! {
                    <Twostop {twostop} {start_live} {end_live} />
                }
            })}
        </div>
    }
}
