use crate::util;
use failure::Error;
use log::log;
use train_schedules_common::*;
use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::*;
use yew_router::{components::RouterLink, prelude::*};

pub struct StationList {
    start_station_id: Option<i32>,
    task: Option<FetchTask>,
    stations: Vec<Station>,
}

#[derive(Properties, FromCaptures)]
pub struct Properties {
    pub start_station_id: Option<i32>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            start_station_id: None,
        }
    }
}

pub enum Message {
    FetchFinished(Vec<Station>),
    FetchError(Error),
}

fn view_station(station: &Station, start_station_id: &Option<i32>) -> Html<StationList> {
    match start_station_id {
        Some(start_station_id) if *start_station_id == station.station_id => {
            html! {}
        }
        Some(start_station_id) => {
            let link = format!("/c/{}/{}", start_station_id, station.station_id);

            html! {
                <li>
                    <RouterLink link=link, text=&station.name />
                </li>
            }
        }
        None => html! {
            let link = format!("/c/{}", station.station_id);

            html! {
                <li>
                    <RouterLink link=link, text=&station.name />
                </li>
            }
        },
    }
}

fn send_back(response: Response<Result<String, Error>>) -> Result<Vec<Station>, Error> {
    let (_, body) = response.into_parts();
    let list = serde_json::from_str(&body?)?;

    Ok(list)
}

impl Component for StationList {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let task = FetchService::new().fetch(
            Request::get("/api/stations").body(Nothing).unwrap(),
            link.send_back(
                |response: Response<Result<String, Error>>| match send_back(response) {
                    Ok(trips) => Message::FetchFinished(trips),
                    Err(e) => Message::FetchError(e),
                },
            ),
        );

        Self {
            start_station_id: props.start_station_id,
            task: Some(task),
            stations: Vec::new(),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        util::state_changed(&mut self.start_station_id, props.start_station_id)
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.task = None;
        match msg {
            Message::FetchFinished(stations) => {
                self.stations = stations;
                true
            }
            Message::FetchError(e) => {
                log!(format!("{}", e));
                false
            }
        }
    }
}

impl Renderable<Self> for StationList {
    fn view(&self) -> Html<Self> {
        let title = if self.start_station_id.is_some() {
            "Choose an ending station"
        } else {
            "Choose a starting station"
        };

        html! {
            <div class="StationList">
                <h1>{ title }</h1>
                <ul>
                { for self.stations.iter().map(|station| view_station(station, &self.start_station_id)) }
                </ul>
            </div>
        }
    }
}
