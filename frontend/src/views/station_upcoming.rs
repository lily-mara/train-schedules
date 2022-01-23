use train_schedules_common::Stop;
use yew::prelude::*;

use crate::{
    context::host,
    fetch::fetch,
    live_status::live_status,
    time,
    views::{time_display::TimeDisplay, twostop::TripId},
};

#[derive(Properties, PartialEq, Clone)]
pub struct StationUpcomingProps {
    pub station_id: i64,
    pub count: usize,
}

#[function_component(StationUpcoming)]
pub fn departures(props: &StationUpcomingProps) -> Html {
    let host = host();

    let stops = use_state(Vec::<Stop>::new);
    fetch(
        format!("{host}/api/upcoming-trips?start={}", props.station_id),
        stops.clone(),
    );
    let live = live_status(&host);

    let now = time::now();

    let stops = stops.iter().filter(|stop| {
        let start_live = live.get(stop.station_id, stop.trip_id);
        let time = start_live
            .as_ref()
            .map(|s| s.departure)
            .unwrap_or(stop.departure);

        time > now
    });

    html! {
        <>
            {for stops.take(props.count).map(|s| html! {
                <Upcoming stop={s.clone()} live={live.get(s.station_id, s.trip_id)} />
            })}
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct UpcomingProps {
    stop: Stop,
    live: Option<Stop>,
}

#[function_component(Upcoming)]
pub fn upcoming(props: &UpcomingProps) -> Html {
    let now = time::now();
    let time_to_departure = (props
        .live
        .as_ref()
        .map(|s| s.departure)
        .unwrap_or(props.stop.departure)
        - now)
        .num_minutes();
    let live = props.live.as_ref().map(|s| s.departure);

    html! {
        <div class={ classes!("TripDisplay") }>
            <TripId id={ props.stop.trip_id } />
            <div class="MinsToDepart">{ format!("{} min.", time_to_departure) }</div>
            <div class="DepartTime">{"Departing "}<TimeDisplay scheduled={ props.stop.departure } {live} /></div>
        </div>
    }
}
