use crate::{context::Context, station_list, trip_list};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Routable, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Route {
    #[at("/c/:start/:end")]
    TripList { start: i32, end: i32 },

    #[at("/c/:start")]
    StationList { start: i32 },

    #[at("/c/")]
    StationListRoot,

    #[at("/")]
    Index,
}

impl Default for Route {
    fn default() -> Self {
        Self::Index
    }
}

#[function_component(Main)]
pub fn main(props: &Context) -> Html {
    let context = use_state(|| props.clone());

    html! {
        <ContextProvider<Context> context={(*context).clone()} >
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </ContextProvider<Context>>
    }
}

fn switch(route: &Route) -> Html {
    match route {
        Route::StationList { start } => {
            html! { <station_list::StationList start_station_id={*start} /> }
        }
        Route::StationListRoot => html! { <station_list::StationList start_station_id={None} /> },
        Route::Index => html! { <station_list::StationList start_station_id={None} /> },
        Route::TripList { start, end } => html! { <trip_list::Model start={*start} end={*end} /> },
    }
}
