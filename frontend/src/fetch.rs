use gloo::timers::callback::Interval;
use log::error;
use serde::de::DeserializeOwned;
use wasm_bindgen_futures::spawn_local;
use yew::{use_state, UseStateHandle};

pub fn fetch_repeating_interval<T>(
    url: String,
    container: UseStateHandle<T>,
    interval: std::time::Duration,
) -> UseStateHandle<Interval>
where
    T: 'static + DeserializeOwned,
{
    fetch_once(url.clone(), container.clone());

    use_state(|| {
        Interval::new(interval.as_millis() as u32, move || {
            fetch(url.clone(), container.clone());
        })
    })
}

pub fn fetch_once<T>(url: String, container: UseStateHandle<T>)
where
    T: 'static + DeserializeOwned,
{
    use_state(|| fetch(url, container));
}

pub fn fetch<T>(url: String, container: UseStateHandle<T>)
where
    T: 'static + DeserializeOwned,
{
    spawn_local(async move {
        match fetch_inner(&url).await {
            Ok(value) => container.set(value),
            Err(e) => {
                error!("failed to fetch: {}", e);
            }
        }
    });
}

async fn fetch_inner<T>(url: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let response = reqwest::get(url).await?.error_for_status()?;

    let body = response.text().await?;
    let list = serde_json::from_str(&body)?;

    Ok(list)
}
