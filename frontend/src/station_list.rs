use crate::util;
use anyhow::Error;
use log::error;
use train_schedules_common::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

pub struct StationList {
    start_station_id: Option<i32>,
    start_station_name: Option<String>,
    stations: Vec<Station>,
}

#[derive(Properties, Clone, PartialEq)]
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
}

fn view_station(station: &Station, start_station_id: &Option<i32>) -> Html {
    match start_station_id {
        Some(start_station_id) if *start_station_id == station.station_id => {
            html! {}
        }
        Some(start_station_id) => {
            let href = format!("/c/{}/{}", start_station_id, station.station_id);

            html! {
                <li>
                    <a { href }> { &station.name } </a>
                </li>
            }
        }
        None => {
            let href = format!("/c/{}", station.station_id);

            html! {
                <li>
                    <a { href }> { &station.name } </a>
                </li>
            }
        }
    }
}

impl StationList {
    fn update_station_name(&mut self) {
        if let Some(station_id) = self.start_station_id {
            for station in &self.stations {
                if station.station_id == station_id {
                    self.start_station_name = Some(station.name.clone())
                }
            }
        }
    }
}

async fn fetch_stations() -> Result<Vec<Station>, Error> {
    let host = util::host();
    let response = reqwest::get(format!("{host}/api/stations")).await?;

    let body = response.text().await?;
    let list = serde_json::from_str(&body)?;

    Ok(list)
}

impl Component for StationList {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        spawn_local(async move {
            match fetch_stations().await {
                Ok(trips) => link.send_message(Message::FetchFinished(trips)),
                Err(e) => {
                    error!("failed to fetch station list: {}", e);
                }
            };
        });
        dbg!();

        Self {
            start_station_id: ctx.props().start_station_id,
            start_station_name: None,
            stations: Vec::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if util::state_changed(&mut self.start_station_id, ctx.props().start_station_id) {
            self.update_station_name();

            true
        } else {
            false
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::FetchFinished(stations) => {
                self.stations = stations;

                self.update_station_name();

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let title = if self.start_station_id.is_some() {
            "Choose an ending station"
        } else {
            "Choose a starting station"
        };

        let start_station = match &self.start_station_name {
            Some(name) => html! {
                <h2>{ format!("Leaving from {}", name) }</h2>
            },
            None => html! {},
        };

        html! {
            <div class="StationList">
                <h1>{ title }</h1>
                { start_station }
                <ul>
                { for self.stations.iter().map(|station| view_station(station, &self.start_station_id)) }
                </ul>
            </div>
        }
    }
}
