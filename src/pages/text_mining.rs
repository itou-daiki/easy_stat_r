use leptos::*;
use polars::prelude::*;
use crate::state::AppData;
use std::collections::HashMap;

#[component]
pub fn TextMining() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    // UI state
    let (target_col, set_target_col) = create_signal(String::new());
    let (min_freq, set_min_freq) = create_signal(2); 
    let (is_processing, set_is_processing) = create_signal(false);
    let (word_cloud_ready, set_word_cloud_ready) = create_signal(false);

    // Columns
    let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let analyze_text = move |_| {
         let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let col = target_col.get();
        if col.is_empty() { return; }

        set_is_processing.set(true);
        set_word_cloud_ready.set(false);

        let processed_col = col.clone();
        
        // Extract text data
        if let Ok(s) = df.column(&processed_col) {
             let mut freq_map: HashMap<String, usize> = HashMap::new();
             
             // Iterate rows and collect text
             let mut full_text = String::new();
             
             for i in 0..s.len() {
                 if let Ok(val) = s.get(i) {
                     match val {
                         AnyValue::String(st) => { full_text.push_str(st); full_text.push(' '); },
                         AnyValue::StringOwned(st) => { full_text.push_str(&st); full_text.push(' '); },
                         _ => continue,
                     }
                 }
             }
             
             // Call JS segmentText
              #[cfg(target_arch = "wasm32")]
             {
                 // Call window.segmentText
                 // Returns JSON string of array of words
                 let result = js_sys::eval(&format!("window.segmentText({})", serde_json::to_string(&full_text).unwrap_or_default()));
                 
                 if let Ok(js_val) = result {
                     if let Some(json_str) = js_val.as_string() {
                         if let Ok(tokens) = serde_json::from_str::<Vec<String>>(&json_str) {
                             // Basic filtering (Stopwords, Particles)
                             // Simple heuristic: exclude Hiragana-only 1-2 chars which are likely particles
                             // actually "word-like" from Segmenter excludes punctuation.
                             for word in tokens {
                                  // Filter:
                                  // 1. Length > 1 (exclude single chars like 'て', 'に', 'を', 'は')
                                  // 2. Exclude common hiragana particles (hardcoded list if needed)
                                  if word.chars().count() > 1 {
                                      // Optional: Extra stop words
                                      if !["それ", "あれ", "これ", "ため", "よう", "もの", "こと"].contains(&word.as_str()) {
                                           *freq_map.entry(word).or_insert(0) += 1;
                                      }
                                  }
                             }
                         }
                     }
                 }
             }

             // Prepare JSON for WordCloud
             let min_f = min_freq.get();
             let mut words_list: Vec<(String, usize)> = freq_map.into_iter()
                 .filter(|(_, count)| *count >= min_f as usize)
                 .collect();
             
             words_list.sort_by(|a, b| b.1.cmp(&a.1));
             
             if words_list.len() > 100 {
                 words_list.truncate(100);
             }
             
             let json_objs: Vec<String> = words_list.iter()
                 .map(|(w, c)| format!("{{\"text\": \"{}\", \"count\": {}}}", w, c))
                 .collect();
             let json_str = format!("[{}]", json_objs.join(","));
             
             // Call JS drawWordCloud
              #[cfg(target_arch = "wasm32")]
             {
                  let _ = js_sys::eval(&format!("window.drawWordCloud('{}')", json_str));
             }
             
             // If not wasm (native check), simulate
              #[cfg(not(target_arch = "wasm32"))]
             {
                 // Mock
             }

             set_word_cloud_ready.set(true);
        }
        
        set_is_processing.set(false);
    };

    view! {
        <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-comments"></i></div>
                "テキストマイニング (Word Cloud)"
            </h2>
             <div class="description-box">
                <p>"※ ブラウザの標準機能(Intl.Segmenter)を使用して単語分割を行っています。品詞判定機能がないため、助詞などが含まれる場合があります。"</p>
            </div>
            
             <div class="control-panel">
                 <div class="input-group">
                    <label>"分析対象カラム (テキスト)"</label>
                    <select on:change=move |ev| set_target_col.set(event_target_value(&ev))>
                        <option value="">"選択してください"</option>
                        {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                    </select>
                </div>
                
                 <div class="input-group">
                    <label>"最小出現頻度"</label>
                    <input type="number" min="1" value=move || min_freq.get() on:change=move |ev| set_min_freq.set(event_target_value(&ev).parse().unwrap_or(2)) />
                 </div>
                
                 <button class="primary-btn" on:click=analyze_text disabled=move || is_processing.get()>
                    {move || if is_processing.get() { "処理中..." } else { "分析を実行" }}
                </button>
            </div>
            
             <div class="result-area">
                <div id="word-cloud-container" style="width: 100%; height: 400px; border: 1px solid #eee; display: flex; align-items: center; justify-content: center; background-color: #fff;">
                     {move || if !word_cloud_ready.get() && !is_processing.get() {
                         view! { <span style="color: #999;">"結果はここに表示されます"</span> }
                     } else {
                         view! { <span/> }
                     }}
                </div>
            </div>
        </div>
    }
}
