use crate::time::{self, local};
use chrono::prelude::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Properties {
    pub scheduled: DateTime<FixedOffset>,
    pub live: Option<DateTime<FixedOffset>>,

    #[prop_or_default]
    pub now: Option<DateTime<FixedOffset>>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            scheduled: time::now(),
            live: None,
            now: None,
        }
    }
}

#[function_component(TimeDisplay)]
pub fn time_display(props: &Properties) -> Html {
    let scheduled = local(props.scheduled);
    let live = props.live.map(local);

    let formatted = live.unwrap_or(scheduled).format("%l:%M %p");

    let title = if live.is_some() {
        format!("Scheduled for {}", scheduled.format("%l:%M %p"))
    } else {
        String::from("")
    };

    let time_display_relative = if live.is_some() {
        Some("TimeDisplay--realtime")
    } else {
        None
    };

    html! {
        <span class={ classes!("TimeDisplay", time_display_relative) } { title }>
        { format!("{}", formatted) }
        { date_diff_tooltip(live.unwrap_or(scheduled), props.now.unwrap_or_else(time::now)) }
        </span>
    }
}

fn date_diff_tooltip(time: DateTime<FixedOffset>, now: DateTime<FixedOffset>) -> Html {
    let today = now.date();
    let date = time.date();

    if today == date {
        return html! {};
    }

    let date_diff = (date - today).num_days();
    let sign = if date_diff > 0 { '+' } else { '-' };

    html! {
        <sup class="TimeDisplay-dateDiff">{ format!("{}{}", sign, date_diff) }</sup>
    }
}
