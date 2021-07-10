use chrono::prelude::*;

pub fn now() -> DateTime<FixedOffset> {
    let date = js_sys::Date::new_0();
    let timestamp = date.get_time() as i64;

    let timestamp_secs = timestamp / 1_000;
    let timestamp_millis = timestamp % 1_000;
    let timestamp_nanos = timestamp_millis * 1_000_000;

    let time = NaiveDateTime::from_timestamp(timestamp_secs, timestamp_nanos as u32);

    local_offset().from_utc_datetime(&time)
}

pub fn local_offset() -> FixedOffset {
    let date = js_sys::Date::new_0();
    let offset = date.get_timezone_offset() as i32;

    FixedOffset::west(offset * 60)
}
