use stdweb::{js, unstable::TryInto};
use train_schedules_common::*;
use yew::prelude::*;

pub struct TripDisplay {
    trip: Trip,
}

#[derive(Properties)]
pub struct TripProperties {
    #[props(required)]
    pub trip: Trip,
}

impl Component for TripDisplay {
    type Message = ();
    type Properties = TripProperties;

    fn create(props: TripProperties, _: ComponentLink<Self>) -> Self {
        Self { trip: props.trip }
    }

    fn update(&mut self, _: ()) -> ShouldRender {
        false
    }
}

impl Renderable<TripDisplay> for TripDisplay {
    fn view(&self) -> Html<Self> {
        let min = current_minute();
        let time_to_departure = self.trip.start.departure_minute - min;

        let service_class = match self.trip.trip_id / 100 {
            1 | 4 => "local",
            2 => "limited",
            3 | 8 => "bullet",
            _ => "",
        };

        html! {
            <div class="TripDisplay">
                <div class=format!("{} TrainID", service_class)>{ self.trip.trip_id }</div>
                <div class="MinsToDepart">{ format!("{} min.", time_to_departure) }</div>
                <div class="DepartTime">{ format!("Departing {}", time_str(self.trip.start.departure_minute)) }</div>
                <div class="ArrivalTime">{ format!("Arriving {}", time_str(self.trip.end.arrival_minute)) }</div>
                <div class="TransitTime">{ format!("{} min. in transit",self.trip.end.arrival_minute - self.trip.start.departure_minute) }</div>
            </div>
        }
    }
}

fn current_minute() -> i64 {
    let min = js! {
        const d = new Date();
        return d.getHours() * 60 + d.getMinutes();
    };

    min.try_into().unwrap()
}
