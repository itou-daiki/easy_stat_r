use leptos::*;
use polars::prelude::*;
use crate::state::AppData;

// Math / Stats imports
use statrs::distribution::{StudentsT, ContinuousCDF};
use statrs::statistics::Statistics;

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
    let (target_col, set_target_col) = create_signal(String::new()); // For Independent: Numeric Var
    let (group_col, set_group_col) = create_signal(String::new());   // For Independent: Categorical Var
    let (pair_col_1, set_pair_col_1) = create_signal(String::new()); // For Paired: Pre
    let (pair_col_2, set_pair_col_2) = create_signal(String::new()); // For Paired: Post
    
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None); 

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
            // Simple check: iterate and check dtype. Polars has schema.
            // For now, assuming all columns valid or user knows. 
            // Ideally filter by data type.
             df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Calculation Logic
    let calculate = move |_| {
        let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();

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
                    
                    let g1_name = &groups[0];
                    let g2_name = &groups[1];

                    // Data extraction helper
                    let get_data = |g_name: &str| -> Option<Vec<f64>> {
                        let mask = df.column(&cat_col).ok()?.equal(g_name).ok()?;
                        let filtered = df.filter(&mask).ok()?;
                        let s = filtered.column(&num_col).ok()?;
                        s.f64().ok()?.into_no_null_iter().collect::<Vec<_>>().into()
                    };

                    let d1 = get_data(g1_name);
                    let d2 = get_data(g2_name);

                    if let (Some(v1), Some(v2)) = (d1, d2) {
                        let n1 = v1.len() as f64;
                        let n2 = v2.len() as f64;
                        if n1 < 2.0 || n2 < 2.0 {
                             set_result_summary.set(Some(vec!["Error: Not enough data points (>1 required)".to_string()]));
                             return;
                        }

                        let mean1 = v1.clone().mean();
                        let mean2 = v2.clone().mean();
                        let var1 = v1.clone().variance();
                        let var2 = v2.clone().variance();

                        // Welch's t-test math
                        let t_num = mean1 - mean2;
                        let t_den = ((var1 / n1) + (var2 / n2)).sqrt();
                         if t_den == 0.0 {
                            set_result_summary.set(Some(vec!["Error: Standard error is zero (identical values?)".to_string()]));
                            return;
                        }
                        let t_val = t_num / t_den;

                        // Welch-Satterthwaite df
                        let df_num = ((var1 / n1) + (var2 / n2)).powi(2);
                        let df_den = ((var1 / n1).powi(2) / (n1 - 1.0)) + ((var2 / n2).powi(2) / (n2 - 1.0));
                        let df_val = df_num / df_den;

                        // P-value
                        let p_val = match StudentsT::new(0.0, 1.0, df_val) {
                            Ok(dist) => 2.0 * (1.0 - dist.cdf(t_val.abs())),
                            Err(_) => f64::NAN,
                        };

                        set_result_summary.set(Some(vec![
                            format!("--- Welch's t-test Result ---"),
                            format!("Group 1 ({}): Mean={:.4}, Var={:.4}, N={}", g1_name.replace("\"", ""), mean1, var1, n1),
                            format!("Group 2 ({}): Mean={:.4}, Var={:.4}, N={}", g2_name.replace("\"", ""), mean2, var2, n2),
                            format!("t-statistic: {:.4}", t_val),
                            format!("df: {:.4}", df_val),
                            format!("p-value: {:.4}", p_val),
                            format!("Significance: {}", if p_val < 0.05 { "Significant (*)" } else { "Not Significant" })
                        ]));

                    } else {
                         set_result_summary.set(Some(vec!["Error extracting data. Check types.".to_string()]));
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

                let v1_opt = get_vec(&p1_col);
                let v2_opt = get_vec(&p2_col);

                if let (Some(v1), Some(v2)) = (v1_opt, v2_opt) {
                    if v1.len() != v2.len() {
                        set_result_summary.set(Some(vec!["Error: Vectors must have same length".to_string()]));
                        return;
                    }
                    let n = v1.len() as f64;
                    if n < 2.0 {
                         set_result_summary.set(Some(vec!["Error: Not enough data".to_string()]));
                         return;
                    }
                    
                    // Difference vector
                    let diffs: Vec<f64> = v1.iter().zip(v2.iter()).map(|(a, b)| a - b).collect();
                    let d_mean = diffs.clone().mean();
                    let d_var = diffs.clone().variance();
                    let d_std = d_var.sqrt();

                    let t_val = d_mean / (d_std / n.sqrt());
                    let df_val = n - 1.0;
                    
                     let p_val = match StudentsT::new(0.0, 1.0, df_val) {
                            Ok(dist) => 2.0 * (1.0 - dist.cdf(t_val.abs())),
                            Err(_) => f64::NAN,
                    };

                    set_result_summary.set(Some(vec![
                        format!("--- Paired t-test Result ---"),
                        format!("Mean Difference: {:.4}", d_mean),
                        format!("Std Dev of Diff: {:.4}", d_std),
                        format!("N: {}", n),
                        format!("t-statistic: {:.4}", t_val),
                        format!("df: {:.4}", df_val),
                        format!("p-value: {:.4}", p_val),
                        format!("Significance: {}", if p_val < 0.05 { "Significant (*)" } else { "Not Significant" })
                    ]));

                } else {
                    set_result_summary.set(Some(vec!["Error extracting data".to_string()]));
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
            </div>
        </div>
    }
}
