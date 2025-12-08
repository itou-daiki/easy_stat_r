use leptos::*;
use polars::prelude::*;

#[derive(Clone, Debug)]
pub struct AppData {
    pub df: RwSignal<Option<DataFrame>>,
    pub file_name: RwSignal<String>,
    pub error_msg: RwSignal<Option<String>>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            df: create_rw_signal(None),
            file_name: create_rw_signal(String::new()),
            error_msg: create_rw_signal(None),
        }
    }
}
