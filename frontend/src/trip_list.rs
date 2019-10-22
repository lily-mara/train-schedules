use crate::trip_display::TripDisplay;
use failure::Error;
use log::log;
use serde::Serialize;
use std::time::Duration;
use train_schedules_common::*;
use yew::{
    format::Nothing,
    prelude::*,
    services::{fetch::*, interval::*},
};
use yew_router::{components::RouterLink, prelude::*};

pub struct Model {
    trip_list: TripList,
    fetch_task: Option<FetchTask>,
    _interval_task: IntervalTask,
    link: ComponentLink<Self>,
}

#[derive(Properties, Serialize, FromCaptures)]
pub struct TripListProps {
    #[props(required)]
    pub start: i32,

    #[props(required)]
    pub end: i32,
}

pub enum Message {
    FetchFinished(TripList),
    FetchError(Error),
    RefetchData,
}

fn process_trips(response: Response<Result<String, Error>>) -> Result<TripList, Error> {
    let (_, body) = response.into_parts();
    let list = serde_json::from_str(&body?)?;

    Ok(list)
}

impl Model {
    fn fetch(&mut self, props: &TripListProps) {
        let fetch_task = FetchService::new().fetch(
            Request::get(format!(
                "/api/upcoming-trips?start={}&end={}",
                props.start, props.end
            ))
            .body(Nothing)
            .unwrap(),
            self.link
                .send_back(|response: Response<Result<String, Error>>| {
                    match process_trips(response) {
                        Ok(trips) => Message::FetchFinished(trips),
                        Err(e) => Message::FetchError(e),
                    }
                }),
        );

        self.fetch_task = Some(fetch_task);
    }
}

impl Component for Model {
    type Message = Message;
    type Properties = TripListProps;

    fn create(props: TripListProps, mut link: ComponentLink<Self>) -> Self {
        let interval_task = IntervalService::new().spawn(
            Duration::from_secs(60),
            link.send_back(|_| Message::RefetchData),
        );

        let mut model = Self {
            trip_list: Default::default(),
            link,
            fetch_task: None,
            _interval_task: interval_task,
        };

        model.fetch(&props);

        model
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.start == self.trip_list.start.station_id
            && props.end == self.trip_list.end.station_id
        {
            return false;
        }

        self.fetch(&props);
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::FetchFinished(trip_list) => {
                self.trip_list = trip_list;
                self.fetch_task = None;
                true
            }
            Message::FetchError(e) => {
                log!(format!("{}", e));
                self.fetch_task = None;
                false
            }
            Message::RefetchData => {
                if self.fetch_task.is_none() {
                    self.fetch(&TripListProps {
                        start: self.trip_list.start.station_id,
                        end: self.trip_list.end.station_id,
                    });
                }
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

        let flipped_url = format!(
            "/c/{}/{}",
            self.trip_list.end.station_id, self.trip_list.start.station_id
        );

        html! {
            <div class="TripList">
                <h1>{ "Upcoming trains" }</h1>
                <h2>{ format!("{} → {}", self.trip_list.start.name, self.trip_list.end.name) } </h2>
                <h3><RouterLink classes="DirectionFlip" text="Change Direction" link=flipped_url /></h3>
                { for self.trip_list.trips.iter().map(Self::view_trip) }
            </div>
        }
    }
}
