use crate::context::host;
use crate::live_status::live_status;
use crate::views::{station_list::StationFilterList, twostop::Twostop};
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

    let stations = use_state_eq::<Vec<Station>, _>(Vec::new);

    crate::fetch::fetch(format!("{host}/api/stations"), stations.clone());

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
            <h1>
                {twostops.start.name.clone()}
                {" "}
                <a classes="DirectionFlip" href={flipped_url}>
                    {"→"}
                </a>
                {" "}
                {twostops.end.name.clone()}
            </h1>
            <h2>{ "Next 5 trips" }</h2>
            { for twostops.trips.iter().take(5).map(|twostop| {
                let twostop = twostop.clone();
                let start_live = live.get(twostop.start.station_id, twostop.trip_id);
                let end_live = live.get(twostop.end.station_id, twostop.trip_id);

                html! {
                    <Twostop {twostop} {start_live} {end_live} />
                }
            })}

            <h2>{ "Filter by ending station" }</h2>
            <StationFilterList start_station_id={props.start} stations={(*stations).clone()} />
        </div>
    }
}
