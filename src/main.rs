use leptos::*;

mod app;
mod components;
mod state;
mod utils;
mod pages;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
