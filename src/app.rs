use leptos::*;
use leptos_router::*;
use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::sidebar::Sidebar;
use crate::components::guide::Guide;
use crate::components::file_upload::FileUpload;
use crate::components::info_section::InfoSection;
use crate::state::AppData;
use crate::pages::{
    data_cleaning::DataCleaning, 
    eda::Eda, 
    correlation::Correlation, 
    chi_square::ChiSquare,
    t_test_unified::TTestUnified,
    anova_unified::AnovaUnified,
    regression::Regression,
    pca::Pca,
    factor_analysis::FactorAnalysis,
    text_mining::TextMining
};

#[component]
pub fn App() -> impl IntoView {
    // Provide global state
    let app_data = AppData::new();
    provide_context(app_data.clone());

    view! {
        <Router base="/easy_stat_r">
            <div style="display: flex;">
                <Sidebar/>
                <main style="margin-left: 250px; width: calc(100% - 250px); min-height: 100vh;">
                    <div class="main-container">
                        <div class="hero-section fade-in">
                            <h1 class="hero-title">"easyStat"</h1>
                            <p class="hero-subtitle">"ブラウザ上で簡単かつ高速に統計分析　-データ駆動型探究を促進-　"</p>
                        </div>

                        <Header/>
                        
                        // Data Upload Section (Visible on all pages for now, or move to specific page?)
                        // User request implies global upload. Let's keep it here or put it in Home. 
                        // For typical SPA, upload is often centralized or on Home.
                        // Let's keep it persistent for now as data state is global.
                        <div class="fade-in">
                            <FileUpload/>
                        </div>
                        
                        <Routes>
                            <Route path="/" view=Home />
                            <Route path="/data_cleaning" view=DataCleaning />
                            <Route path="/eda" view=Eda />
                            <Route path="/correlation" view=Correlation />
                            <Route path="/chi_square" view=ChiSquare />
                            <Route path="/t_test" view=TTestUnified />
                            <Route path="/anova" view=AnovaUnified />
                            <Route path="/regression" view=Regression />
                            <Route path="/pca" view=Pca />
                            <Route path="/factor_analysis" view=FactorAnalysis />
                            <Route path="/text_mining" view=TextMining />
                        </Routes>

                        <Footer/>
                    </div>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");
    view! {
        <div>
            // Display Data Info, DataFrame Preview, and Summary Statistics
            {move || {
                if let Some(df) = app_data.df.get() {
                    let shape = df.shape();
                    let column_names: Vec<String> = df.get_column_names()
                        .iter()
                        .map(|s| s.to_string())
                        .collect();

                    // データフレームプレビュー（先頭10行）
                    let preview_df = df.head(Some(10));
                    let num_rows = preview_df.height();

                    view! {
                        <div>
                            <div class="section fade-in">
                                <h2 class="section-title">
                                    <div class="section-icon"><i class="fas fa-database"></i></div>
                                    "データ情報"
                                </h2>
                                <p><strong>"行数: "</strong> {shape.0} " | " <strong>"列数: "</strong> {shape.1}</p>
                            </div>

                            <div class="section fade-in">
                                <h2 class="section-title">
                                    <div class="section-icon"><i class="fas fa-table"></i></div>
                                    "データフレームプレビュー (先頭"{num_rows}"行)"
                                </h2>
                                <div style="overflow-x: auto;">
                                    <table class="data-table">
                                        <thead>
                                            <tr>
                                                {column_names.iter().map(|col_name| {
                                                    view! {
                                                        <th>{col_name.clone()}</th>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {(0..num_rows).map(|row_idx| {
                                                let row_data: Vec<String> = column_names.iter().map(|col_name| {
                                                    preview_df.column(col_name.as_str())
                                                        .ok()
                                                        .and_then(|col| col.get(row_idx).ok())
                                                        .map(|val| val.to_string())
                                                        .unwrap_or_else(|| "N/A".to_string())
                                                }).collect();

                                                view! {
                                                    <tr>
                                                        {row_data.iter().map(|val| {
                                                            view! {
                                                                <td>{val.clone()}</td>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div class="section fade-in">
                                <h2 class="section-title">
                                    <div class="section-icon"><i class="fas fa-calculator"></i></div>
                                    "要約統計量"
                                </h2>
                                <p>"数値列の基本統計量を表示します。"</p>

                                <div style="overflow-x: auto;">
                                    <table class="stats-table">
                                        <thead>
                                            <tr>
                                                <th>"列名"</th>
                                                <th>"平均"</th>
                                                <th>"標準偏差"</th>
                                                <th>"最小値"</th>
                                                <th>"最大値"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {column_names.iter().filter_map(|col_name| {
                                                let col = df.column(col_name.as_str()).ok()?;

                                                // 数値列のみ処理
                                                if !col.dtype().is_numeric() {
                                                    return None;
                                                }

                                                let mean = col.mean().unwrap_or(f64::NAN);
                                                let std = col.std(1).unwrap_or(f64::NAN);
                                                let min = col.min::<f64>().unwrap_or(f64::NAN);
                                                let max = col.max::<f64>().unwrap_or(f64::NAN);

                                                Some(view! {
                                                    <tr>
                                                        <td><strong>{col_name.clone()}</strong></td>
                                                        <td>{format!("{:.3}", mean)}</td>
                                                        <td>{format!("{:.3}", std)}</td>
                                                        <td>{format!("{:.3}", min)}</td>
                                                        <td>{format!("{:.3}", max)}</td>
                                                    </tr>
                                                })
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }
            }}

            <Guide/>

            <div class="section fade-in">
                <h2 class="section-title">
                    <div class="section-icon"><i class="fas fa-chart-line"></i></div>
                    "利用可能な分析機能"
                </h2>
                <div class="feature-grid">
                     <A href="/data_cleaning" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-broom feature-card-icon"></i>
                                "データクレンジング"
                            </h3>
                            <p class="feature-card-description">"データの前処理と品質向上"</p>
                        </div>
                    </A>
                    <A href="/eda" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-search feature-card-icon"></i>
                                "探索的データ分析（EDA）"
                            </h3>
                            <p class="feature-card-description">"データの特徴と傾向を視覚的に把握"</p>
                        </div>
                    </A>
                    <A href="/correlation" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-project-diagram feature-card-icon"></i>
                                "相関分析"
                            </h3>
                            <p class="feature-card-description">"変数間の関係性を分析"</p>
                        </div>
                    </A>
                     <A href="/chi_square" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-border-all feature-card-icon"></i>
                                "カイ二乗検定"
                            </h3>
                            <p class="feature-card-description">"度数の偏りを分析"</p>
                        </div>
                    </A>
                    <A href="/t_test" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-balance-scale feature-card-icon"></i>
                                "t検定（統合版）"
                            </h3>
                            <p class="feature-card-description">"2つのグループの平均値を比較"</p>
                        </div>
                    </A>
                    <A href="/anova" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-layer-group feature-card-icon"></i>
                                "分散分析（統合版）"
                            </h3>
                            <p class="feature-card-description">"複数グループの平均値を比較"</p>
                        </div>
                    </A>
                    <A href="/regression" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-chart-line feature-card-icon"></i>
                                "回帰分析（統合版）"
                            </h3>
                            <p class="feature-card-description">"変数間の因果関係をモデル化"</p>
                        </div>
                    </A>
                    <A href="/pca" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-compress-arrows-alt feature-card-icon"></i>
                                "主成分分析"
                            </h3>
                            <p class="feature-card-description">"データの次元を削減して可視化"</p>
                        </div>
                    </A>
                    <A href="/factor_analysis" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-sitemap feature-card-icon"></i>
                                "因子分析"
                            </h3>
                            <p class="feature-card-description">"潜在的な因子構造を探索"</p>
                        </div>
                    </A>
                    <A href="/text_mining" class="feature-card-link">
                        <div class="feature-card">
                            <h3 class="feature-card-title">
                                <i class="fas fa-comments feature-card-icon"></i>
                                "テキストマイニング"
                            </h3>
                            <p class="feature-card-description">"テキストデータから意味を抽出"</p>
                        </div>
                    </A>
                </div>
            </div>

            <InfoSection/>
        </div>
    }
}
