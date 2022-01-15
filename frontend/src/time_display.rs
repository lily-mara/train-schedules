use crate::time;
use chrono::prelude::*;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Properties {
    pub time: Time,

    #[prop_or_default]
    pub now: Option<DateTime<FixedOffset>>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            time: Time {
                scheduled: time::now(),
                estimated: None,
            },
            now: None,
        }
    }
}

#[function_component(TimeDisplay)]
pub fn time_display(props: &Properties) -> Html {
    let formatted = props.time.format("%l:%M %p");

    let title = if props.time.is_live() {
        format!("Scheduled for {}", props.time.scheduled.format("%l:%M %p"))
    } else {
        String::from("")
    };

    let time_display_relative = if props.time.is_live() {
        Some("TimeDisplay--realtime")
    } else {
        None
    };

    html! {
        <span class={ classes!("TimeDisplay", time_display_relative) } { title }>
        { format!("{}", formatted) }
        { date_diff_tooltip(props) }
        </span>
    }
}

fn date_diff_tooltip(props: &Properties) -> Html {
    let now = props.now.unwrap_or_else(time::now);
    let today = now.date();
    let date = props.time.date();

    if today == date {
        return html! {};
    }

    let date_diff = (date - today).num_days();
    let sign = if date_diff > 0 { '+' } else { '-' };

    html! {
        <sup class="TimeDisplay-dateDiff">{ format!("{}{}", sign, date_diff) }</sup>
    }
}
