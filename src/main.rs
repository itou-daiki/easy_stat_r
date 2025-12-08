use leptos::*;

fn main() {
    // Basic main for potentially running local dev server or just satisfying bin target
    // In a pure CSR (Trunk) setup, the entry point is often via lib.rs -> index.html
    // But having a main.rs is fine.
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <p>"Hello World from easy_stat_r!"</p> });
}
