use chrono::prelude::*;
use stdweb::{js, unstable::TryInto};

pub fn now() -> DateTime<FixedOffset> {
    let timestamp = js! {
        return new Date().getTime();
    };

    let timestamp: i64 = timestamp.try_into().unwrap();

    let timestamp_secs = timestamp / 1_000;
    let timestamp_millis = timestamp % 1_000;
    let timestamp_nanos = timestamp_millis * 1_000_000;

    let time = NaiveDateTime::from_timestamp(timestamp_secs, timestamp_nanos as u32);

    local_offset().from_utc_datetime(&time)
}

pub fn local_offset() -> FixedOffset {
    let offset = js! {
        return new Date().getTimezoneOffset();
    };

    let offset: i32 = offset.try_into().unwrap();

    FixedOffset::west(offset * 60)
}
