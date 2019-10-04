use serde::Serialize;
use stdweb::js;

#[macro_export]
macro_rules! log {
    ($value:expr) => {{
        ::log::log_internal($value, file!(), line!(), stringify!($value))
    }};
}

pub fn log_internal<T>(value: T, filename: &str, line: u32, expression: &str) -> T
where
    T: Serialize,
{
    let json = serde_json::to_string(&value).unwrap();

    js! { @(no_return)
        const value = { value: JSON.parse(@{json}) };
        value.filename = @{filename};
        value.line = @{line};
        value.expression = @{expression};
        console.dir(value);
    };

    value
}
