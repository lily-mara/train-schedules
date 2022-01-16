use std::time::Duration;

use gloo::timers::callback::Interval;
use yew::{use_state, UseStateHandle};

#[must_use = "You must store this in a local variable to ensure it isn't dropped. The timer will die if it's dropped."]
/// Handle to a timer refreshing a component periodically
pub struct Refresher(UseStateHandle<Interval>);

/// Periodically re-render this component at the interval specified by the
/// duration passed to it.
pub fn refresh_periodically(interval: Duration) -> Refresher {
    let updater = use_state(|| ());
    Refresher(use_state(|| {
        Interval::new(interval.as_millis() as u32, move || {
            updater.set(());
        })
    }))
}
