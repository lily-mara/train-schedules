use crate::{AppState, Result};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use train_schedules_common::Station;

pub async fn stations(_req: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(load_all_stations(&data.connection)?))
}

fn load_all_stations(connection: &sqlite::Connection) -> Result<Vec<Station>> {
    let mut stmt = connection.prepare(
        "
            select distinct stop_name, station_id
            from stops
            order by stops.stop_id asc
        ",
    )?;

    let mut stations = Vec::new();

    while let sqlite::State::Row = stmt.next()? {
        stations.push(Station {
            name: stmt.read(0)?,
            station_id: stmt.read::<i64>(1)?,
        });
    }

    Ok(stations)
}
