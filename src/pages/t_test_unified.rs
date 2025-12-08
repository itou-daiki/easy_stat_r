use leptos::*;
use polars::prelude::*;
use crate::state::AppData;

// Math / Stats imports
use statrs::distribution::{StudentsT, ContinuousCDF};
use statrs::statistics::Statistics;
use serde_json::json;

#[derive(Clone, PartialEq)]
enum TTestType {
    Independent, // Welch's t-test
    Paired,      // Paired t-test
}

#[component]
pub fn TTestUnified() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");
    
    // UI State
    let (test_type, set_test_type) = create_signal(TTestType::Independent);
    let (target_col, set_target_col) = create_signal(String::new());
    let (group_col, set_group_col) = create_signal(String::new());
    let (pair_col_1, set_pair_col_1) = create_signal(String::new());
    let (pair_col_2, set_pair_col_2) = create_signal(String::new());
    
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None); 
    // Interpretation Text
    let (interpretation, set_interpretation) = create_signal(String::new());

    // Visualization ID
    let plot_id = "ttest_plot";

    // Derived signals for column options
    let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let numeric_cols = create_memo(move |_| {
         if let Some(df) = app_data.df.get() {
             df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Helper to draw plot
    let draw_plot = move |data: serde_json::Value, layout: serde_json::Value| {
        #[cfg(target_arch = "wasm32")]
        {
            let d_str = data.to_string();
            let l_str = layout.to_string();
            let _ = js_sys::eval(&format!("window.drawPlot('{}', '{}', '{}')", plot_id, d_str, l_str));
        }
    };

    // Calculation Logic
    let calculate = move |_| {
        let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();

        set_interpretation.set(String::new());
        set_result_summary.set(None);

        match test_type.get() {
            TTestType::Independent => {
                let num_col = target_col.get();
                let cat_col = group_col.get();
                if num_col.is_empty() || cat_col.is_empty() { return; }
                
                // Extract unique groups
                let unique_groups = df.column(&cat_col).ok()
                    .and_then(|c| c.unique().ok())
                    .map(|s| s.iter().map(|v| v.to_string()).collect::<Vec<_>>());

                if let Some(groups) = unique_groups {
                    if groups.len() != 2 {
                         set_result_summary.set(Some(vec![
                            format!("Error: Group variable must have exactly 2 levels. Found: {:?}", groups)
                        ]));
                        return;
                    }
                    
                    let g1_name = groups[0].replace("\"", "");
                    let g2_name = groups[1].replace("\"", "");

                    // Data extraction
                    let get_data = |g_name: &str| -> Option<Vec<f64>> {
                        // Handle quotes in filtering if needed. Assuming clean matching for now or robust string match.
                        // Best way: iterate rows and split.
                         let mut vals = vec![];
                         let cat_s = df.column(&cat_col).ok()?;
                         let num_s = df.column(&num_col).ok()?;
                         
                         for i in 0..cat_s.len() {
                             if let (Ok(c_val), Ok(n_val)) = (cat_s.get(i), num_s.get(i)) {
                                 let c_str = c_val.to_string().replace("\"", "");
                                 if c_str == g_name {
                                      // Attempt to extract as f64 directly
                                      if let Ok(f) = n_val.try_extract::<f64>() {
                                          vals.push(f);
                                      } else {
                                          // Fallback: Check for Int or Cast logic manually if needed
                                          // For now, strict extraction. If cast is needed, Series cast is better before iteration.
                                          match n_val {
                                              AnyValue::Int32(v) => vals.push(v as f64),
                                              AnyValue::Int64(v) => vals.push(v as f64),
                                              AnyValue::UInt32(v) => vals.push(v as f64),
                                              AnyValue::UInt64(v) => vals.push(v as f64),
                                              _ => {}
                                          }
                                      }
                                 }
                             }
                         }
                         if vals.is_empty() { None } else { Some(vals) }
                    };

                    let d1 = get_data(&g1_name);
                    let d2 = get_data(&g2_name);

                    if let (Some(v1), Some(v2)) = (d1, d2) {
                        let n1 = v1.len() as f64;
                        let n2 = v2.len() as f64;
                        
                        let mean1 = v1.clone().mean();
                        let mean2 = v2.clone().mean();
                        let std1 = v1.clone().std_dev();
                        let std2 = v2.clone().std_dev();
                        let var1 = v1.clone().variance();
                        let var2 = v2.clone().variance();
                        let se1 = std1 / n1.sqrt();
                        let se2 = std2 / n2.sqrt();

                        // Welch's t-test
                        let t_den = ((var1 / n1) + (var2 / n2)).sqrt();
                        let t_val = (mean1 - mean2) / t_den;
                        let df_num = ((var1 / n1) + (var2 / n2)).powi(2);
                        let df_den = ((var1 / n1).powi(2) / (n1 - 1.0)) + ((var2 / n2).powi(2) / (n2 - 1.0));
                        let df_val = df_num / df_den;
                        
                        let p_val = match StudentsT::new(0.0, 1.0, df_val) {
                             Ok(dist) => 2.0 * (1.0 - dist.cdf(t_val.abs())),
                             Err(_) => f64::NAN,
                        };
                        
                        // Effect Size (Cohen's d) - using pooled std for d
                        let pooled_std = (((n1 - 1.0)*var1 + (n2 - 1.0)*var2) / (n1 + n2 - 2.0)).sqrt();
                        let d_val = (mean1 - mean2).abs() / pooled_std;

                        let sig_label = if p_val < 0.01 { "**" } else if p_val < 0.05 { "*" } else if p_val < 0.1 { "†" } else { "n.s." };

                        set_result_summary.set(Some(vec![
                            format!("--- Welch's t-test Result ---"),
                            format!("Group 1 ({}): Mean={:.4}, SD={:.4}, N={}", g1_name, mean1, std1, n1),
                            format!("Group 2 ({}): Mean={:.4}, SD={:.4}, N={}", g2_name, mean2, std2, n2),
                            format!("t({:.2}) = {:.4}, p = {:.4} {}", df_val, t_val, p_val, sig_label),
                            format!("Effect Size (d) = {:.4}", d_val),
                        ]));

                        // Interpretation Text
                        let comp_str = if mean1 > mean2 { format!("{} > {}", g1_name, g2_name) } else { format!("{} < {}", g1_name, g2_name) };
                        let sig_text = 
                            if p_val < 0.05 { format!("有意な差が認められました ({}) 。", comp_str) }
                            else if p_val < 0.1 { format!("有意な差がある傾向が認められました ({}) 。", comp_str) }
                            else { "有意な差は認められませんでした。".to_string() };
                            
                        set_interpretation.set(format!(
                            "【解釈の補助】\n分析の結果、{}変数「{}」を用いた場合、{}変数「{}」について、{}\n(t({:.2})={:.2}, p={:.3}, d={:.2})",
                             cat_col, cat_col, num_col, num_col, sig_text, df_val, t_val, p_val, d_val
                        ));

                        // Visualization (Bar Chart with Error Bars)
                        let data_plot = json!([
                            {
                                "type": "bar",
                                "x": [g1_name, g2_name],
                                "y": [mean1, mean2],
                                "error_y": {
                                    "type": "data",
                                    "array": [se1, se2],
                                    "visible": true
                                },
                                "marker": { "color": ["#1e90ff", "#ff7f0e"] }
                            }
                        ]);
                        
                        let layout_plot = json!({
                            "title": format!("Mean Comparison: {} by {}", num_col, cat_col),
                            "yaxis": { "title": num_col },
                            "xaxis": { "title": cat_col },
                            "margin": { "t": 40, "b": 40, "l": 50, "r": 20 }
                        });
                        
                        draw_plot(data_plot, layout_plot);

                    }
                }
            },
            TTestType::Paired => {
                 let p1_col = pair_col_1.get();
                 let p2_col = pair_col_2.get();
                 if p1_col.is_empty() || p2_col.is_empty() { return; }

                 let get_vec = |c_name: &str| -> Option<Vec<f64>> {
                    df.column(c_name).ok()?.f64().ok()?.into_no_null_iter().collect::<Vec<_>>().into()
                 };

                 if let (Some(v1), Some(v2)) = (get_vec(&p1_col), get_vec(&p2_col)) {
                      if v1.len() != v2.len() { return; }
                      let n = v1.len() as f64;
                      
                      let diffs: Vec<f64> = v1.iter().zip(v2.iter()).map(|(a, b)| a - b).collect();
                      let d_mean = diffs.clone().mean();
                      let d_std = diffs.clone().std_dev();
                      let d_se = d_std / n.sqrt();
                      
                      let t_val = d_mean / d_se;
                      let df_val = n - 1.0;
                      
                      let p_val = match StudentsT::new(0.0, 1.0, df_val) {
                             Ok(dist) => 2.0 * (1.0 - dist.cdf(t_val.abs())),
                             Err(_) => f64::NAN,
                      };
                      
                      // For paired, effect size usually d_z or d_avg. d_z = t / sqrt(n)
                      let d_val = t_val.abs() / n.sqrt();
                      let sig_label = if p_val < 0.01 { "**" } else if p_val < 0.05 { "*" } else if p_val < 0.1 { "†" } else { "n.s." };
                      
                      let mean1 = v1.clone().mean();
                      let mean2 = v2.clone().mean();
                      let se1 = v1.clone().std_dev() / n.sqrt();
                      let se2 = v2.clone().std_dev() / n.sqrt();

                      set_result_summary.set(Some(vec![
                            format!("--- Paired t-test Result ---"),
                            format!("Variable 1 ({}): Mean={:.4}, SE={:.4}", p1_col, mean1, se1),
                            format!("Variable 2 ({}): Mean={:.4}, SE={:.4}", p2_col, mean2, se2),
                            format!("Mean Diff: {:.4} (SD={:.4})", d_mean, d_std),
                            format!("t({:.0}) = {:.4}, p = {:.4} {}", df_val, t_val, p_val, sig_label),
                            format!("Effect Size (d) = {:.4}", d_val),
                      ]));

                      let comp_str = if mean1 > mean2 { format!("{} > {}", p1_col, p2_col) } else { format!("{} < {}", p1_col, p2_col) };
                      let sig_text = 
                            if p_val < 0.05 { format!("有意な差が認められました ({}) 。", comp_str) }
                            else if p_val < 0.1 { format!("有意な差がある傾向が認められました ({}) 。", comp_str) }
                            else { "有意な差は認められませんでした。".to_string() };

                       set_interpretation.set(format!(
                            "【解釈の補助】\n分析の結果、「{}」と「{}」の間には、{}\n(t({:.0})={:.2}, p={:.3}, d={:.2})",
                             p1_col, p2_col, sig_text, df_val, t_val, p_val, d_val
                        ));

                      // Plot
                      let data_plot = json!([
                            {
                                "type": "bar",
                                "x": [p1_col, p2_col],
                                "y": [mean1, mean2],
                                "error_y": {
                                    "type": "data",
                                    "array": [se1, se2],
                                    "visible": true
                                },
                                "marker": { "color": ["#2ca02c", "#d62728"] }
                            }
                      ]);
                       let layout_plot = json!({
                            "title": "Paired Comparison",
                             "margin": { "t": 40, "b": 40, "l": 50, "r": 20 }
                        });
                        draw_plot(data_plot, layout_plot);
                 }
            }
        }
    };

    view! {
        <div class="fade-in">
            <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-balance-scale"></i></div>
                "t検定 (統合版)"
            </h2>

            <div class="control-panel">
                 <div class="radio-group">
                    <label>
                        <input type="radio" name="ttest_type" 
                            on:click=move |_| set_test_type.set(TTestType::Independent)
                            checked=move || test_type.get() == TTestType::Independent
                        />
                        "対応なし (Welch's method)"
                    </label>
                    <label>
                        <input type="radio" name="ttest_type" 
                            on:click=move |_| set_test_type.set(TTestType::Paired)
                            checked=move || test_type.get() == TTestType::Paired
                        />
                        "対応あり (Paired t-test)"
                    </label>
                </div>

                {move || match test_type.get() {
                    TTestType::Independent => view! {
                        <div class="input-group">
                            <label>"群分け変数 (カテゴリ)"</label>
                            <select on:change=move |ev| set_group_col.set(event_target_value(&ev))>
                                <option value="">"選択してください"</option>
                                {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                            </select>
                            
                            <label>"従属変数 (数値)"</label>
                            <select on:change=move |ev| set_target_col.set(event_target_value(&ev))>
                                <option value="">"選択してください"</option>
                                {numeric_cols.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_view(),
                    TTestType::Paired => view! {
                        <div class="input-group">
                            <label>"変数1 (Pre)"</label>
                            <select on:change=move |ev| set_pair_col_1.set(event_target_value(&ev))>
                                <option value="">"選択してください"</option>
                                {numeric_cols.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                            </select>
                            
                            <label>"変数2 (Post)"</label>
                           <select on:change=move |ev| set_pair_col_2.set(event_target_value(&ev))>
                                <option value="">"選択してください"</option>
                                {numeric_cols.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_view()
                }}

                <button class="primary-btn" on:click=calculate>
                    "検定を実行"
                </button>
            </div>

            <div class="result-area">
                {move || result_summary.get().map(|lines| view! {
                    <div class="result-box">
                        <h3>"分析結果"</h3>
                        <ul>
                            {lines.into_iter().map(|l| view! { <li>{l}</li> }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                })}
                
                {move || if !interpretation.get().is_empty() {
                    view! {
                         <div class="interpretation-box" style="margin-top: 20px; padding: 15px; background-color: #f9f9f9; border-left: 5px solid #1e90ff;">
                             <h4 style="margin-top: 0;">"解釈の補助"</h4>
                             <p style="white-space: pre-wrap;">{interpretation.get()}</p>
                         </div>
                    }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }}
                
                // Visualization Area
                <div id=plot_id style="width: 100%; height: 400px; margin-top: 20px;"></div>
            </div>
        </div>
    }
}
