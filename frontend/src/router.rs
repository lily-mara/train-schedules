use crate::trip_list;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Routes;

impl Component for Routes {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<Self> for Routes {
    fn view(&self) -> Html<Self> {
        html! {
            <Router>
                <Route
                    matcher=route!("/c/{start}/{end}")
                    render=Render::new(|matches: &Captures| {
                        let start: i32 = matches["start"].parse().ok()?;
                        let end: i32 = matches["end"].parse().ok()?;

                        Some(html! {
                            <trip_list::Model start=start end=end />
                        })
                    }) />
            </Router>
        }
    }
}
