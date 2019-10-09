use crate::{time, util};
use chrono::prelude::*;
use train_schedules_common::*;
use yew::prelude::*;

pub struct TimeDisplay {
    time: Time,
    now: Option<DateTime<FixedOffset>>,
}

#[derive(Properties)]
pub struct Properties {
    #[props(required)]
    pub time: Time,

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

impl Component for TimeDisplay {
    type Message = ();
    type Properties = Properties;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            time: props.time,
            now: props.now,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        util::state_changed(&mut self.time, props.time)
            || util::state_changed(&mut self.now, props.now)
    }

    fn update(&mut self, _: ()) -> ShouldRender {
        false
    }
}

impl Renderable<Self> for TimeDisplay {
    fn view(&self) -> Html<Self> {
        let formatted = self.time.format("%l:%M %p");

        let title = if self.time.is_live() {
            format!("Scheduled for {}", self.time.scheduled.format("%l:%M %p"))
        } else {
            String::from("")
        };

        html! {
            <span class=classes!("TimeDisplay", "TimeDisplay--realtime" => self.time.is_live()) title=title>
            { format!("{}", formatted) }
            { self.date_diff_tooltip() }
            </span>
        }
    }
}

impl TimeDisplay {
    fn date_diff_tooltip(&self) -> Html<Self> {
        let now = self.now.unwrap_or_else(time::now);
        let today = now.date();
        let date = self.time.date();

        if today == date {
            return html! {};
        }

        let date_diff = (date - today).num_days();
        let sign = if date_diff > 0 { '+' } else { '-' };

        html! {
            <sup class="TimeDisplay-dateDiff">{ format!("{}{}", sign, date_diff) }</sup>
        }
    }
}
