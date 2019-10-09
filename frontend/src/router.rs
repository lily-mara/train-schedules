use crate::{station_list, trip_list};
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Routes;

impl Component for Routes {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<Self> for Routes {
    fn view(&self) -> Html<Self> {
        html! {
            <Router>
                <Route matcher=route!("/c/{start}/{end}") render=component::<trip_list::Model>() />
                <Route matcher=route!("/") render=component::<station_list::StationList>() />
                <Route matcher=route!("/c/") render=component::<station_list::StationList>() />
                <Route matcher=route!("/c/{start_station_id}") render=component::<station_list::StationList>() />
            </Router>
        }
    }
}
