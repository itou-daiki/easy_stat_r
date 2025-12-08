use leptos::*;

#[component]
pub fn Guide() -> impl IntoView {
    view! {
        <div class="section fade-in">
            <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-book"></i></div>
                "学習ガイド"
            </h2>
            <ul style="line-height: 1.8;">
                <li>
                    <a href="https://dit-lab.notion.site/612d9665350544aa97a2a8514a03c77c?v=85ad37a3275b4717a0033516b9cfd9cc" target="_blank" style="color: var(--primary-color); font-weight: 500;">
                        "情報探究ステップアップガイド"
                    </a>
                </li>
                <li>
                    <a href="https://dit-lab.notion.site/Dit-Lab-da906d09d3cf42a19a011cf4bf25a673?pvs=4" target="_blank" style="color: var(--primary-color); font-weight: 500;">
                        "中の人のページ（Dit-Lab.）"
                    </a>
                </li>
            </ul>
        </div>
    }
}
