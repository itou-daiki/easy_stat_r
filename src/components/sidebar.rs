use leptos::*;
use leptos_router::*;

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
                        <A href="/" exact=true class="nav-link" active_class="active">
                            <i class="fas fa-home" style="width: 24px;"></i> "ホーム"
                        </A>
                    </li>
                    <li style="margin-bottom: 0.5rem;">
                        <A href="/data_cleaning" class="nav-link" active_class="active">
                            <i class="fas fa-broom" style="width: 24px;"></i> "データクレンジング"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/eda" class="nav-link" active_class="active">
                            <i class="fas fa-search" style="width: 24px;"></i> "EDA"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/correlation" class="nav-link" active_class="active">
                            <i class="fas fa-project-diagram" style="width: 24px;"></i> "相関分析"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/chi_square" class="nav-link" active_class="active">
                            <i class="fas fa-border-all" style="width: 24px;"></i> "カイ二乗検定"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/t_test" class="nav-link" active_class="active">
                            <i class="fas fa-balance-scale" style="width: 24px;"></i> "t検定 (統合版)"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/anova" class="nav-link" active_class="active">
                            <i class="fas fa-layer-group" style="width: 24px;"></i> "分散分析 (統合版)"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/regression" class="nav-link" active_class="active">
                            <i class="fas fa-chart-line" style="width: 24px;"></i> "回帰分析 (統合版)"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/pca" class="nav-link" active_class="active">
                            <i class="fas fa-compress-arrows-alt" style="width: 24px;"></i> "主成分分析"
                        </A>
                    </li>
                     <li style="margin-bottom: 0.5rem;">
                        <A href="/factor_analysis" class="nav-link" active_class="active">
                            <i class="fas fa-project-diagram" style="width: 24px;"></i> "因子分析"
                        </A>
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
