use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div style="font-family: sans-serif; padding: 20px;">
            <h1>"Easy Stat R"</h1>
            <p>"Ready for statistical analysis in Wasm!"</p>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    
    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());
    
    // Mount the App component
    mount_to_body(|| view! { <App/> });
}
