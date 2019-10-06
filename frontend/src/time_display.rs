use crate::time;
use chrono::prelude::*;
use yew::prelude::*;

pub struct TimeDisplay {
    time: DateTime<FixedOffset>,
    now: Option<DateTime<FixedOffset>>,
}

#[derive(Properties)]
pub struct Properties {
    #[props(required)]
    pub time: DateTime<FixedOffset>,

    pub now: Option<DateTime<FixedOffset>>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            time: time::now(),
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

    fn update(&mut self, _: ()) -> ShouldRender {
        false
    }
}

impl Renderable<Self> for TimeDisplay {
    fn view(&self) -> Html<Self> {
        let formatted = self.time.format("%l:%M %p");

        html! {
            <span class="TimeDisplay">
            { format!("{}", formatted)}
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
