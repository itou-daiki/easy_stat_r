use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer style="margin-top: 4rem; padding-top: 2rem; border-top: 1px solid var(--border-color); text-align: center;">
            <div style="margin-bottom: 2rem;">
                <h3 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1rem;">"リンク"</h3>
                <ul style="list-style: none; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; align-items: center;">
                    <li><a href="https://dit-lab.notion.site/Dit-Lab-da906d09d3cf42a19a011cf4bf25a673?pvs=4" target="_blank" style="color: var(--primary-color); text-decoration: none;">"中の人のページ（Dit-Lab.）"</a></li>
                    <li><a href="https://easy-base-converter.streamlit.app" target="_blank" style="color: var(--primary-color); text-decoration: none;">"進数変換学習アプリ"</a></li>
                    <li><a href="https://easy-rsa.streamlit.app/" target="_blank" style="color: var(--primary-color); text-decoration: none;">"easyRSA"</a></li>
                    <li><a href="https://huggingface.co/spaces/itou-daiki/pycaret_datascience_streamlit" target="_blank" style="color: var(--primary-color); text-decoration: none;">"easyAutoML（回帰）"</a></li>
                    <li><a href="https://huggingface.co/spaces/itou-daiki/pkl_predict_reg" target="_blank" style="color: var(--primary-color); text-decoration: none;">"pkl_predict_reg"</a></li>
                    <li><a href="https://audiovisualizationanalysis-bpeekdjwymuf6nkqcb4cqy.streamlit.app" target="_blank" style="color: var(--primary-color); text-decoration: none;">"音のデータサイエンス"</a></li>
                    <li><a href="https://boxplot-4-mysteams.streamlit.app" target="_blank" style="color: var(--primary-color); text-decoration: none;">"3D RGB Cube Visualizer"</a></li>
                    <li><a href="https://sailing-mark-angle.streamlit.app" target="_blank" style="color: var(--primary-color); text-decoration: none;">"上マーク角度計算補助ツール"</a></li>
                    <li><a href="https://factor-score-calculator.streamlit.app/" target="_blank" style="color: var(--primary-color); text-decoration: none;">"Factor Score Calculator"</a></li>
                    <li><a href="https://easy-xl-merge.streamlit.app" target="_blank" style="color: var(--primary-color); text-decoration: none;">"easy Excel Merge"</a></li>
                    <li><a href="https://forms.gle/G5sMYm7dNpz2FQtU9" target="_blank" style="color: var(--primary-color); text-decoration: none;">"フィードバックはこちらまで"</a></li>
                    <li><a href="https://github.com/itou-daiki/easy_stat" target="_blank" style="color: var(--primary-color); text-decoration: none;">"ソースコードはこちら（GitHub）"</a></li>
                </ul>
            </div>
            
            <div style="margin-bottom: 2rem;">
                <p>"ご意見・ご要望は→ "<a href="https://forms.gle/G5sMYm7dNpz2FQtU9" target="_blank" style="color: var(--primary-color);">"https://forms.gle/G5sMYm7dNpz2FQtU9"</a>" まで"</p>
                <div style="margin-top: 1rem;">
                    <h4 style="font-size: 1rem; color: var(--text-primary);">"© 2022-2025 Dit-Lab.(Daiki Ito). All Rights Reserved."</h4>
                    <p style="color: var(--text-secondary);">"easyStat: Open Source for Ubiquitous Statistics"</p>
                    <p style="color: var(--text-secondary);">"Democratizing data, everywhere."</p>
                </div>
            </div>

            <div style="margin-bottom: 1rem;">
                <h4 style="font-size: 0.9rem; margin-bottom: 0.5rem;">"In collaboration with our esteemed contributors:"</h4>
                <p style="color: var(--text-secondary);">"・Toshiyuki"</p>
                <p style="font-size: 0.8rem; color: var(--text-secondary);">"With heartfelt appreciation for their dedication and support."</p>
            </div>
        </footer>
    }
}
