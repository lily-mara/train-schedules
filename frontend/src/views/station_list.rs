use crate::context::host;
use crate::views::station_upcoming::StationUpcoming;
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

    let start_station = props.start_station_id.and_then(|start_station_id| {
        stations
            .iter()
            .find(|s| s.station_id == start_station_id)
            .map(|s| (s.name.clone(), start_station_id))
    });

    let start_station = match start_station {
        Some((name, station_id)) => html! {
            <>
                <h1>{ name }</h1>
                <h2>{"Next 3 departures"}</h2>
                <StationUpcoming {station_id} count={3} />
                <h2>{ "Filter by ending station" }</h2>
            </>
        },
        None => html! {
            <h1>{ "Choose a station" }</h1>
        },
    };

    html! {
        <div class="StationList">
            { start_station }
            <StationFilterList stations={(*stations).clone()} start_station_id={props.start_station_id} />
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

#[derive(Properties, PartialEq, Clone)]
pub struct StationFilterListProps {
    pub start_station_id: Option<i64>,
    pub stations: Vec<Station>,
}

#[function_component(StationFilterList)]
pub fn station_filter_list(props: &StationFilterListProps) -> Html {
    html! {
        <ul>
        { for props.stations.iter().map(|station| view_station(station, &props.start_station_id)) }
        </ul>
    }
}
