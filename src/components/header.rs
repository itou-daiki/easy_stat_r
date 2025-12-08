use leptos::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header style="padding: 1rem 0; text-align: center;">
            <div style="font-size: 0.9rem; color: var(--text-secondary);">
                "Created by Dit-Lab.(Daiki Ito)"
            </div>
        </header>
    }
}
