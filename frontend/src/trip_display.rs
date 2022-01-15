use crate::{time, time_display::TimeDisplay};
use gloo::timers::callback::Interval;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct TripProperties {
    pub trip: Trip,
}

#[function_component(TripDisplay)]
pub fn view(props: &TripProperties) -> Html {
    let updater = use_state(|| ());
    let _interval = use_state(|| {
        Interval::new(30_000, move || {
            updater.set(());
        })
    });

    let trip = &props.trip;

    let now = time::now();
    let time_to_departure = (*trip.start.departure - now).num_minutes();

    let transit_time = (*trip.end.departure - *trip.start.arrival)
        .num_minutes()
        .abs();

    html! {
        <div class={ classes!("TripDisplay") }>
            <TrainId id={ trip.trip_id } />
            <div class="MinsToDepart">{ format!("{} min.", time_to_departure) }</div>
            <div class="DepartTime">{"Departing "}<TimeDisplay time={ trip.start.departure } /></div>
            <div class="ArrivalTime">{"Arriving "}<TimeDisplay time={ trip.end.arrival } /></div>
            <div class="TransitTime">{ format!("{} min. in transit", transit_time) }</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TrainIdProps {
    pub id: i64,
}

#[function_component(TrainId)]
pub fn train_id(props: &TrainIdProps) -> Html {
    let href = format!("/c/train/{}", props.id);
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
