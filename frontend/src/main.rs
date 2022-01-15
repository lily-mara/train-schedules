#![recursion_limit = "2048"]

use log::Level;

mod router;
mod station_list;
mod time;
mod time_display;
mod trip_display;
mod trip_list;
mod util;

static mut HOST: String = String::new();

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let host = document.location().unwrap().host().unwrap();
    unsafe {
        HOST = host;
    }

    yew::start_app_in_element::<router::Main>(
        document.query_selector("#app-container").unwrap().unwrap(),
    );
}
