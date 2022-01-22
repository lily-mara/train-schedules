use std::time::Duration;

use crate::{time, views::time_display::TimeDisplay};
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct TwostopProperties {
    pub twostop: TwoStop,
}

#[function_component(Twostop)]
pub fn view(props: &TwostopProperties) -> Html {
    let _refresher = crate::timer::refresh_periodically(Duration::from_secs(30));

    let twostop = &props.twostop;

    let now = time::now();
    let time_to_departure = (twostop.start.departure - now).num_minutes();

    let transit_time = (twostop.end.arrival - twostop.start.departure)
        .num_minutes()
        .abs();

    html! {
        <div class={ classes!("TripDisplay") }>
            <TripId id={ twostop.trip_id } />
            <div class="MinsToDepart">{ format!("{} min.", time_to_departure) }</div>
            <div class="DepartTime">{"Departing "}<TimeDisplay scheduled={ twostop.start.departure } /></div>
            <div class="ArrivalTime">{"Arriving "}<TimeDisplay scheduled={ twostop.end.arrival } /></div>
            <div class="TransitTime">{ format!("{} min. in transit", transit_time) }</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TripIdProps {
    pub id: i64,
}

#[function_component(TripId)]
pub fn train_id(props: &TripIdProps) -> Html {
    let href = format!("/c/trip/{}", props.id);
    let class = match props.id / 100 {
        1 | 4 => "local",
        2 => "limited",
        3 | 8 => "bullet",
        _ => "",
    };

    html! {
        <a {href}><div class={ classes!("TrainID", class) }>{ props.id }</div></a>
    }
}
