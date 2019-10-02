use serde::Serialize;
use stdweb::js;

#[macro_export]
macro_rules! log {
    ($value:expr) => {{
        ::log::log_internal($value, file!(), line!())
    }};
}

pub fn log_internal<T>(value: T, filename: &str, line: u32) -> T
where
    T: Serialize,
{
    let json = serde_json::to_string(&value).unwrap();

    js! { @(no_return)
        const value = { value: JSON.parse(@{json}) };
        value.filename = @{filename};
        value.line = @{line};
        console.dir(value);
    };

    value
}
