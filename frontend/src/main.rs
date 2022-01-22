#![recursion_limit = "2048"]

use crate::context::Context;
use log::Level;

mod context;
mod fetch;
mod live_status;
mod time;
mod timer;
mod views;

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let location = document.location().unwrap();

    let host = format!(
        "{}//{}",
        location.protocol().unwrap(),
        location.host().unwrap()
    );

    yew::start_app_with_props_in_element::<views::router::Main>(
        document.query_selector("#app-container").unwrap().unwrap(),
        Context { host },
    );
}
