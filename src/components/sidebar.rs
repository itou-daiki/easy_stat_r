use leptos::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside style="width: 250px; background: var(--surface); border-right: 1px solid var(--border-color); height: 100vh; position: fixed; left: 0; top: 0; padding: 2rem 1rem; overflow-y: auto;">
            <div style="margin-bottom: 2rem; font-weight: 700; font-size: 1.5rem; color: var(--primary-color);">
                "easyStat"
            </div>
            
            <nav>
                <ul style="list-style: none; padding: 0;">
                    <li style="margin-bottom: 0.5rem;">
                        <a href="#" style="display: block; padding: 0.75rem 1rem; color: var(--text-primary); text-decoration: none; border-radius: 8px; background: rgba(99, 102, 241, 0.1); font-weight: 500;">
                            <i class="fas fa-home" style="width: 24px;"></i> "ホーム"
                        </a>
                    </li>
                    <li style="margin-bottom: 0.5rem;">
                        <a href="#" style="display: block; padding: 0.75rem 1rem; color: var(--text-secondary); text-decoration: none; border-radius: 8px; transition: background 0.2s;">
                            <i class="fas fa-broom" style="width: 24px;"></i> "データクレンジング"
                        </a>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <a href="#" style="display: block; padding: 0.75rem 1rem; color: var(--text-secondary); text-decoration: none; border-radius: 8px; transition: background 0.2s;">
                            <i class="fas fa-search" style="width: 24px;"></i> "EDA"
                        </a>
                    </li>
                    // Add more dummy links based on feature list
                </ul>
            </nav>

            <div style="margin-top: 2rem;">
                 <h3 style="font-size: 0.9rem; color: var(--text-secondary); margin-bottom: 1rem;">"🎓 統計学習のコツ"</h3>
                 <ul style="padding-left: 1.2rem; color: var(--text-secondary); font-size: 0.85rem;">
                    <li>"📊 まずはグラフで視覚化"</li>
                    <li>"🔢 数値だけでなく意味も考える"</li>
                    <li>"❓ 「なぜ？」を常に問いかける"</li>
                    <li>"📈 複数の分析を組み合わせる"</li>
                    <li>"📝 結果を文章で説明してみる"</li>
                 </ul>
            </div>
        </aside>
    }
}
