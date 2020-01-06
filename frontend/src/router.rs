use crate::{station_list, trip_list};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Switch, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Routes {
    #[to = "/c/{start}/{end}"]
    TripList { start: i32, end: i32 },

    #[to = "/c/{start}"]
    StationList { start: Option<i32> },

    #[to = "/"]
    Index,
}

impl Default for Routes {
    fn default() -> Self {
        Self::Index
    }
}

pub struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html<Self> {
        html! {
            <Router<Routes, ()>
                render = Router::render(|switch: Routes| {
                    match switch {
                        Routes::StationList{start} => html! { <station_list::StationList start_station_id=start /> },
                        Routes::Index => html! { <station_list::StationList start_station_id=None /> },
                        Routes::TripList{start, end} => html! { <trip_list::Model start=start, end=end /> },
                    }
                })
            />
        }
    }
}
