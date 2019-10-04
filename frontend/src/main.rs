#![recursion_limit = "2048"]

use yew::prelude::*;

mod router;
mod time;
mod time_display;
mod trip_display;
mod trip_list;

fn main() {
    yew::initialize();
    App::<router::Routes>::new().mount_to_body();
    yew::run_loop();
}
