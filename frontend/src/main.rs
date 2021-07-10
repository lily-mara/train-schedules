#![recursion_limit = "2048"]

use log::Level;
use yew::prelude::*;

mod router;
mod station_list;
mod time;
mod time_display;
mod trip_display;
mod trip_list;
mod util;

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    yew::initialize();
    App::<router::Model>::new().mount(document.query_selector("#app-container").unwrap().unwrap());
    yew::run_loop();
}
