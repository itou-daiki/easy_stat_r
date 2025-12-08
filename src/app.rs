use leptos::*;
use leptos_router::*;
use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::sidebar::Sidebar;
use crate::components::guide::Guide;
use crate::components::file_upload::FileUpload;
use crate::state::AppData;
use crate::pages::{
    data_cleaning::DataCleaning, 
    eda::Eda, 
    correlation::Correlation, 
    chi_square::ChiSquare,
    t_test_unified::TTestUnified,
    anova_unified::AnovaUnified
};

#[component]
pub fn App() -> impl IntoView {
    // Provide global state
    let app_data = AppData::new();
    provide_context(app_data.clone());

    view! {
        <Router>
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
            // Display Data Info (Debugging/Status)
            {move || {
                    if let Some(df) = app_data.df.get() {
                    let shape = df.shape();
                    view! {
                        <div class="section fade-in">
                            <h2 class="section-title">"データ情報"</h2>
                            <p>"行数: " {shape.0} ", 列数: " {shape.1}</p>
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
                </div>
            </div>
        </div>
    }
}
