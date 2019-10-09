#![recursion_limit = "2048"]

use stdweb::web::{self, IParentNode};
use yew::prelude::*;

#[macro_use]
mod classes;
mod router;
mod station_list;
mod time;
mod time_display;
mod trip_display;
mod trip_list;
mod util;

fn main() {
    yew::initialize();
    App::<router::Routes>::new().mount(
        web::document()
            .query_selector("#app-container")
            .unwrap()
            .unwrap(),
    );
    yew::run_loop();
}
