use leptos::*;

#[component]
pub fn InfoSection() -> impl IntoView {
    let (about_open, set_about_open) = create_signal(false);
    let (usage_open, set_usage_open) = create_signal(false);
    let (updates_open, set_updates_open) = create_signal(false);

    view! {
        <div class="info-section fade-in">
            // アプリの説明
            <div class="collapsible-section">
                <div
                    class="collapsible-header"
                    class:active=move || about_open.get()
                    on:click=move |_| set_about_open.update(|v| *v = !*v)
                >
                    <h3 class="collapsible-title">
                        <i class="fas fa-info-circle"></i>
                        " easyStatについて"
                    </h3>
                    <i class="fas fa-chevron-down collapsible-icon" class:rotate=move || about_open.get()></i>
                </div>
                <div class="collapsible-content" class:open=move || about_open.get()>
                    <p>"easyStatは、ブラウザ上で動作する統計分析Webアプリケーションです。"</p>
                    <ul>
                        <li>"🚀 "<strong>"高速処理"</strong>": WebAssembly (Rust)による高速なデータ処理"</li>
                        <li>"🔒 "<strong>"プライバシー保護"</strong>": データはブラウザ内で処理され、外部に送信されません"</li>
                        <li>"💻 "<strong>"無料・オープンソース"</strong>": 誰でも自由に利用できます"</li>
                        <li>"📊 "<strong>"豊富な分析機能"</strong>": 基本統計から多変量解析まで対応"</li>
                    </ul>
                </div>
            </div>

            // 使い方ガイド
            <div class="collapsible-section">
                <div
                    class="collapsible-header"
                    class:active=move || usage_open.get()
                    on:click=move |_| set_usage_open.update(|v| *v = !*v)
                >
                    <h3 class="collapsible-title">
                        <i class="fas fa-book-open"></i>
                        " 使い方"
                    </h3>
                    <i class="fas fa-chevron-down collapsible-icon" class:rotate=move || usage_open.get()></i>
                </div>
                <div class="collapsible-content" class:open=move || usage_open.get()>
                    <ol class="usage-steps">
                        <li>
                            <strong>"1. データをアップロード"</strong>
                            <p>"CSV形式またはExcel形式のデータファイルをアップロードしてください。"</p>
                        </li>
                        <li>
                            <strong>"2. 分析手法を選択"</strong>
                            <p>"左のサイドバーまたはホーム画面のカードから、実行したい分析を選んでください。"</p>
                        </li>
                        <li>
                            <strong>"3. パラメータを設定"</strong>
                            <p>"分析に必要な変数やオプションを選択してください。"</p>
                        </li>
                        <li>
                            <strong>"4. 結果を確認"</strong>
                            <p>"分析結果が表示されます。グラフや統計量を確認しましょう。"</p>
                        </li>
                    </ol>
                </div>
            </div>

            // 更新履歴
            <div class="collapsible-section">
                <div
                    class="collapsible-header"
                    class:active=move || updates_open.get()
                    on:click=move |_| set_updates_open.update(|v| *v = !*v)
                >
                    <h3 class="collapsible-title">
                        <i class="fas fa-history"></i>
                        " 更新履歴"
                    </h3>
                    <i class="fas fa-chevron-down collapsible-icon" class:rotate=move || updates_open.get()></i>
                </div>
                <div class="collapsible-content" class:open=move || updates_open.get()>
                    <div class="update-timeline">
                        <div class="update-item">
                            <div class="update-date">"2025年12月"</div>
                            <div class="update-content">
                                <ul>
                                    <li>"全機能カードの表示を追加"</li>
                                    <li>"折りたたみ式情報セクションを実装"</li>
                                    <li>"GitHub Pagesへのデプロイ対応"</li>
                                </ul>
                            </div>
                        </div>
                        <div class="update-item">
                            <div class="update-date">"2025年11月"</div>
                            <div class="update-content">
                                <ul>
                                    <li>"テキストマイニング機能を追加"</li>
                                    <li>"因子分析機能を追加"</li>
                                    <li>"主成分分析機能を追加"</li>
                                </ul>
                            </div>
                        </div>
                        <div class="update-item">
                            <div class="update-date">"初期リリース"</div>
                            <div class="update-content">
                                <ul>
                                    <li>"基本的な統計分析機能を実装"</li>
                                    <li>"データクレンジング機能"</li>
                                    <li>"探索的データ分析（EDA）"</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
