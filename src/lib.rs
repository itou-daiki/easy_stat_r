use leptos::*;
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod state;
mod utils;
mod pages;

use app::App;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    
    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());
    
    // Mount the App component
    mount_to_body(|| view! { <App/> });
}
