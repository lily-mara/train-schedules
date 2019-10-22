use crate::{time, time_display::TimeDisplay, util};
use std::time::Duration;
use train_schedules_common::*;
use yew::{prelude::*, services::interval::*};

pub struct TripDisplay {
    trip: Trip,
    _interval_task: IntervalTask,
}

#[derive(Properties)]
pub struct TripProperties {
    #[props(required)]
    pub trip: Trip,
}

impl Component for TripDisplay {
    type Message = ();
    type Properties = TripProperties;

    fn create(props: TripProperties, mut link: ComponentLink<Self>) -> Self {
        Self {
            trip: props.trip,
            _interval_task: IntervalService::new()
                .spawn(Duration::from_secs(30), link.send_back(|_| ())),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        util::state_changed(&mut self.trip, props.trip)
    }

    fn update(&mut self, _: ()) -> ShouldRender {
        true
    }
}

impl Renderable<TripDisplay> for TripDisplay {
    fn view(&self) -> Html<Self> {
        let now = time::now();
        let time_to_departure = (*self.trip.start.departure - now).num_minutes();

        let service_class = match self.trip.trip_id / 100 {
            1 | 4 => "local",
            2 => "limited",
            3 | 8 => "bullet",
            _ => "",
        };

        let transit_time = (*self.trip.end.departure - *self.trip.start.arrival)
            .num_minutes()
            .abs();

        html! {
            <div class=classes!("TripDisplay")>
                <div class=classes!("TrainID", service_class)>{ self.trip.trip_id }</div>
                <div class="MinsToDepart">{ format!("{} min.", time_to_departure) }</div>
                <div class="DepartTime">{"Departing "}<TimeDisplay time=self.trip.start.departure /></div>
                <div class="ArrivalTime">{"Arriving "}<TimeDisplay time=self.trip.end.arrival /></div>
                <div class="TransitTime">{ format!("{} min. in transit", transit_time) }</div>
            </div>
        }
    }
}
