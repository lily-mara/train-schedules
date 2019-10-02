use crate::trip_display::TripDisplay;
use failure::Error;
use log::log;
use serde::de;
use train_schedules_common::*;
use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::*;

pub struct Model {
    trip_list: TripList,
    task: Option<FetchTask>,
}

#[derive(Properties)]
pub struct TripListProps {
    #[props(required)]
    pub start: i32,

    #[props(required)]
    pub end: i32,
}

pub enum Message {
    FetchFinished(TripList),
    FetchError(Error),
}

fn deserialize<T>(req: Response<Result<String, Error>>) -> Result<Response<T>, Error>
where
    for<'de> T: de::Deserialize<'de>,
{
    let (parts, body) = req.into_parts();
    let body = serde_json::from_str(&body?)?;
    Ok(Response::from_parts(parts, body))
}

fn process_trips(response: Response<Result<String, Error>>) -> Result<TripList, Error> {
    log!(format!("{:?}", response));
    let response = deserialize(response)?;
    let (_, body) = response.into_parts();

    Ok(body)
}

impl Component for Model {
    type Message = Message;
    type Properties = TripListProps;

    fn create(props: TripListProps, mut link: ComponentLink<Self>) -> Self {
        let task = FetchService::new().fetch(
            Request::get(format!(
                "/api/upcoming-trips?start={}&end={}",
                props.start, props.end
            ))
            .body(Nothing)
            .unwrap(),
            link.send_back(|response: Response<Result<String, Error>>| {
                match process_trips(response) {
                    Ok(trips) => Message::FetchFinished(trips),
                    Err(e) => Message::FetchError(e),
                }
            }),
        );
        Self {
            trip_list: Default::default(),
            task: Some(task),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::FetchFinished(trip_list) => {
                self.trip_list = trip_list;
                self.task = None;
                true
            }
            Message::FetchError(e) => {
                log!(format!("{}", e));
                self.task = None;
                false
            }
        }
    }
}

impl Model {
    fn view_trip(trip: &Trip) -> Html<Self> {
        html! {
            <TripDisplay trip=trip></TripDisplay>
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        if self.trip_list.trips.is_empty() {
            return html! {
                <h1>{ "No trips found" }</h1>
            };
        }

        html! {
            <div class="TripList">
                <h1>{ "Upcoming trains" }</h1>
                <h2>{ format!("{} → {}", self.trip_list.start.name, self.trip_list.end.name) }</h2>
                { for self.trip_list.trips.iter().map(Self::view_trip) }
            </div>
        }
    }
}
