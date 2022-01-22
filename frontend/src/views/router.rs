use crate::{context::Context, views::*};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Routable, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Route {
    #[at("/c/station/:start/:end")]
    Twostop { start: i64, end: i64 },

    #[at("/c/station/:start")]
    StationList { start: i64 },

    #[at("/c/trip/:trip_id")]
    Trip { trip_id: i64 },

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
        Route::Twostop { start, end } => {
            html! { <twostop_list::Model start={*start} end={*end} /> }
        }
        Route::Trip { trip_id } => html! { <trip_view::TripView trip_id={*trip_id} /> },
    }
}
