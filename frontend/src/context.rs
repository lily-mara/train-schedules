use yew::{use_context, Properties};

#[derive(Properties, PartialEq, Clone)]
pub struct Context {
    pub host: String,
}

pub fn get() -> Context {
    use_context::<Context>().unwrap()
}

pub fn host() -> String {
    get().host
}
