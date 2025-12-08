use leptos::*;
use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::sidebar::Sidebar;
use crate::components::guide::Guide;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div style="display: flex;">
            <Sidebar/>
            <main style="margin-left: 250px; width: calc(100% - 250px); min-height: 100vh;">
                <div class="main-container">
                    <div class="hero-section fade-in">
                        <h1 class="hero-title">"easyStat"</h1>
                        <p class="hero-subtitle">"ブラウザ上で簡単かつ高速に統計分析　-データ駆動型探究を促進-　"</p>
                    </div>

                    <Header/>

                    <div class="section fade-in">
                        <h2 class="section-title">
                            <div class="section-icon"><i class="fas fa-info-circle"></i></div>
                            "About easyStat"
                        </h2>
                        <p style="color: var(--text-secondary); line-height: 1.8;">
                            "easyStatは、ブラウザ上で手軽に統計分析を行えるウェブアプリケーションです。"
                            <br/>
                            "PCやタブレット端末など、様々な環境に対応しています。"
                        </p>
                    </div>

                    <Guide/>

                    <div class="section fade-in">
                        <h2 class="section-title">
                            <div class="section-icon"><i class="fas fa-chart-line"></i></div>
                            "利用可能な分析機能"
                        </h2>
                        <div class="feature-grid">
                            <div class="feature-card">
                                <h3 class="feature-card-title">
                                    <i class="fas fa-broom feature-card-icon"></i>
                                    "データクレンジング"
                                </h3>
                                <p class="feature-card-description">"データの前処理と品質向上"</p>
                            </div>
                            <div class="feature-card">
                                <h3 class="feature-card-title">
                                    <i class="fas fa-search feature-card-icon"></i>
                                    "探索的データ分析（EDA）"
                                </h3>
                                <p class="feature-card-description">"データの特徴と傾向を視覚的に把握"</p>
                            </div>
                            <div class="feature-card">
                                <h3 class="feature-card-title">
                                    <i class="fas fa-project-diagram feature-card-icon"></i>
                                    "相関分析"
                                </h3>
                                <p class="feature-card-description">"変数間の関係性を分析"</p>
                            </div>
                             // ... Other features can be added similarly ...
                             <div class="feature-card">
                                <h3 class="feature-card-title">
                                    <i class="fas fa-chart-area feature-card-icon"></i>
                                    "二要因分散分析"
                                    <span class="badge-new"> "New!" </span>
                                </h3>
                                <p class="feature-card-description">"2つの要因の効果を同時に分析"</p>
                            </div>
                        </div>
                    </div>
                    
                    // Simple updates section placeholder
                     <div class="section fade-in">
                        <h2 class="section-title">
                            <div class="section-icon"><i class="fas fa-history"></i></div>
                            "更新履歴"
                        </h2>
                        <div class="update-timeline">
                            <div class="update-item">
                                <div class="update-date">"2025/6/4"</div>
                                <div class="update-content">
                                    <ul>
                                        <li>"トップページのデザインを刷新しました。"</li>
                                    </ul>
                                </div>
                            </div>
                             <div class="update-item">
                                <div class="update-date">"2025/4/18"</div>
                                <div class="update-content">
                                    <ul>
                                        <li>"テキストマイニングの共起ネットワーク描画機能を実装しました。"</li>
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </div>

                    <Footer/>
                </div>
            </main>
        </div>
    }
}
