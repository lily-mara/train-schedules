use crate::context::host;
use train_schedules_common::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq, Default)]
pub struct Properties {
    pub start_station_id: Option<i64>,
}

#[function_component(StationList)]
pub fn station_list(props: &Properties) -> Html {
    let stations = use_state_eq::<Vec<Station>, _>(Vec::new);

    let host = host();
    crate::fetch::fetch(format!("{host}/api/stations"), stations.clone());

    let start_station_name = props.start_station_id.and_then(|start_station_id| {
        stations
            .iter()
            .find(|s| s.station_id == start_station_id)
            .map(|s| s.name.clone())
    });

    let title = if props.start_station_id.is_some() {
        "Choose an ending station"
    } else {
        "Choose a starting station"
    };

    let start_station = match &start_station_name {
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
            { for stations.iter().map(|station| view_station(station, &props.start_station_id)) }
            </ul>
        </div>
    }
}

fn view_station(station: &Station, start_station_id: &Option<i64>) -> Html {
    match start_station_id {
        Some(start_station_id) if *start_station_id == station.station_id => {
            html! {}
        }
        Some(start_station_id) => {
            let href = format!("/c/station/{}/{}", start_station_id, station.station_id);

            html! {
                <li>
                    <a { href }> { &station.name } </a>
                </li>
            }
        }
        None => {
            let href = format!("/c/station/{}", station.station_id);

            html! {
                <li>
                    <a { href }> { &station.name } </a>
                </li>
            }
        }
    }
}
