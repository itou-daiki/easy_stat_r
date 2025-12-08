use leptos::*;
use crate::state::AppData;
use statrs::distribution::{FisherSnedecor, ContinuousCDF};
use nalgebra::{DMatrix, DVector};
use std::collections::{HashSet, HashMap};
use serde_json::json;

#[derive(Clone, PartialEq)]
enum AnovaType {
    OneWay,
    TwoWay,
}

#[component]
pub fn AnovaUnified() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    let (test_type, set_test_type) = create_signal(AnovaType::OneWay);
    let (target_col, set_target_col) = create_signal(String::new());
    let (factor1_col, set_factor1_col) = create_signal(String::new());
    let (factor2_col, set_factor2_col) = create_signal(String::new());
    
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None);
    let (interpretation, set_interpretation) = create_signal(String::new());
    
    let plot_id = "anova_plot";

     let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Helper: Draw Plot
    let draw_plot = move |data: serde_json::Value, layout: serde_json::Value| {
        #[cfg(target_arch = "wasm32")]
        {
             let d_str = data.to_string();
             let l_str = layout.to_string();
             let _ = js_sys::eval(&format!("window.drawPlot('{}', '{}', '{}')", plot_id, d_str, l_str));
        }
    };

    // Helper: OLS fitting
    fn fit_ols(y: &DVector<f64>, x: &DMatrix<f64>) -> Result<(f64, usize), String> {
        let xt = x.transpose();
        let xtx = &xt * x;
        let xty = &xt * y;
        
        let chol = nalgebra::linalg::Cholesky::new(xtx.clone());
        if let Some(decomp) = chol {
            let beta = decomp.solve(&xty);
            let y_pred = x * &beta;
            let residuals = y - y_pred;
            let sse = residuals.dot(&residuals);
            let df_resid = y.len() - x.ncols();
            Ok((sse, df_resid))
        } else {
            Err("Matrix singular".to_string())
        }
    }

    fn create_design_matrix(
        rows: usize, 
        factors: Vec<(&String, &Vec<String>)> 
    ) -> DMatrix<f64> {
        let mut mat_data = vec![1.0; rows];
        let mut cols = 1;
        for (_, values) in factors {
            let mut unique_levels: Vec<_> = values.iter().collect::<HashSet<_>>().into_iter().collect();
            unique_levels.sort();
            if unique_levels.len() > 1 {
                for level in &unique_levels[1..] {
                    let col_vec: Vec<f64> = values.iter().map(|v| if v == *level { 1.0 } else { 0.0 }).collect();
                    mat_data.extend(col_vec);
                    cols += 1;
                }
            }
        }
        DMatrix::from_vec(rows, cols, mat_data)
    }
    
    fn get_dummies(n: usize, data: &Vec<String>) -> (Vec<Vec<f64>>, Vec<&String>) {
         let mut unique: Vec<_> = data.iter().collect::<HashSet<_>>().into_iter().collect();
         unique.sort();
         let mut dummies = vec![];
         let mut level_names = vec![];
         if unique.len() > 1 {
             for level in &unique[1..] {
                 let col: Vec<f64> = data.iter().map(|v| if v == *level { 1.0 } else { 0.0 }).collect();
                 dummies.push(col);
                 level_names.push(*level);
             }
         }
         (dummies, level_names)
    }

    let calculate = move |_| {
         let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let target = target_col.get();
        let f1 = factor1_col.get();
        
        if target.is_empty() || f1.is_empty() { return; }
        
        set_interpretation.set(String::new());
        set_result_summary.set(None);

        // Extract Y
        let y_vec_opt: Option<Vec<f64>> = df.column(&target).ok().and_then(|c| c.f64().ok()).map(|s| s.into_no_null_iter().collect());
        // Extract F1
        let f1_vec_opt: Option<Vec<String>> = df.column(&f1).ok().map(|s| s.iter().map(|v| v.to_string().replace("\"", "")).collect());
        
        if y_vec_opt.is_none() || f1_vec_opt.is_none() { 
            set_result_summary.set(Some(vec!["Error reading data".to_string()]));
            return; 
        }
        
        let y_data = y_vec_opt.unwrap();
        let f1_data = f1_vec_opt.unwrap();
        let n = y_data.len();
        let y_dvec = DVector::from_vec(y_data.clone());

        match test_type.get() {
            AnovaType::OneWay => {
                let x0 = DMatrix::from_element(n, 1, 1.0);
                
                let mut levels: Vec<_> = f1_data.iter().collect::<HashSet<_>>().into_iter().collect();
                levels.sort();
                let k = levels.len();
                
                let mut x1_data = vec![1.0; n];
                let mut x1_cols = 1;
                for lvl in &levels[1..] {
                   let col: Vec<f64> = f1_data.iter().map(|v| if v == *lvl { 1.0 } else { 0.0 }).collect();
                   x1_data.extend(col);
                   x1_cols += 1;
                }
                let x1 = DMatrix::from_vec(n, x1_cols, x1_data);

                let res0 = fit_ols(&y_dvec, &x0);
                let res1 = fit_ols(&y_dvec, &x1);
                
                if let (Ok((sse0, _)), Ok((sse1, df_resid1))) = (res0, res1) {
                    let df_model = k - 1;
                    let ss_model = sse0 - sse1;
                    let ss_error = sse1;
                    let ms_model = ss_model / df_model as f64;
                    let ms_error = ss_error / df_resid1 as f64;
                    let f_val = ms_model / ms_error;
                    
                    let p_val = match FisherSnedecor::new(df_model as f64, df_resid1 as f64) {
                        Ok(dist) => 1.0 - dist.cdf(f_val),
                        Err(_) => f64::NAN,
                    };
                    
                    let sig_text = if p_val < 0.05 { "有意な差が認められました" } else { "有意な差は認められませんでした" };
                    let sig_mark = if p_val < 0.01 { "**" } else if p_val < 0.05 { "*" } else { "n.s." };

                     set_result_summary.set(Some(vec![
                        format!("--- One-Way ANOVA Result ---"),
                        format!("Factor: {} ({} levels)", f1, k),
                        format!("F({}, {}) = {:.4}, p = {:.4} {}", df_model, df_resid1, f_val, p_val, sig_mark),
                    ]));
                    
                     set_interpretation.set(format!(
                        "【解釈の補助】\n要因「{}」による母平均の差について検定を行った結果、{}(p={:.4})。",
                         f1, sig_text, p_val
                    ));

                    // Visualization: Bar Chart of Means
                    // Calculate mean for each level
                    let mut means = HashMap::new();
                    let mut level_order = levels.clone();
                    level_order.sort(); // Alphabetic order

                    for lvl in &level_order {
                        let mut sum = 0.0;
                        let mut count = 0;
                        for i in 0..n {
                            if f1_data[i] == **lvl {
                                sum += y_data[i];
                                count += 1;
                            }
                        }
                        if count > 0 {
                            means.insert(*lvl, sum / count as f64);
                        }
                    }
                    
                    let x_vals: Vec<String> = level_order.iter().map(|s| s.to_string()).collect();
                    let y_vals: Vec<f64> = level_order.iter().map(|s| *means.get(s).unwrap_or(&0.0)).collect();

                    let data_plot = json!([
                        {
                            "type": "bar",
                            "x": x_vals,
                            "y": y_vals,
                            "marker": { "color": "#1f77b4" }
                        }
                    ]);
                    let layout_plot = json!({
                        "title": format!("means: {} by {}", target, f1),
                        "yaxis": { "title": target },
                        "xaxis": { "title": f1 },
                        "margin": { "t": 40, "b": 40, "l": 50, "r": 20 }
                    });
                    
                    draw_plot(data_plot, layout_plot);


                } else {
                    set_result_summary.set(Some(vec!["Error fitting OLS models".to_string()]));
                }
            },
            AnovaType::TwoWay => {
                let f2 = factor2_col.get();
                if f2.is_empty() { return; }
                
                 let f2_vec_opt: Option<Vec<String>> = df.column(&f2).ok().map(|s| s.iter().map(|v| v.to_string().replace("\"", "")).collect());
                 if f2_vec_opt.is_none() { return; }
                 let f2_data = f2_vec_opt.unwrap();
                 
                 let x_a = create_design_matrix(n, vec![(&f1, &f1_data)]);
                 let x_b = create_design_matrix(n, vec![(&f2, &f2_data)]);
                 let x_ab = create_design_matrix(n, vec![(&f1, &f1_data), (&f2, &f2_data)]);
                 
                 let (dummies_a, _) = get_dummies(n, &f1_data);
                 let (dummies_b, _) = get_dummies(n, &f2_data);
                 
                 let mut interaction_cols = vec![];
                 for col_a in &dummies_a {
                     for col_b in &dummies_b {
                         let inter: Vec<f64> = col_a.iter().zip(col_b.iter()).map(|(a, b)| a * b).collect();
                         interaction_cols.push(inter);
                     }
                 }
                 
                 let mut full_data_vec = x_ab.data.as_vec().clone();
                 for col in interaction_cols {
                     full_data_vec.extend(col);
                 }
                 let cols_ab = x_ab.ncols();
                 let cols_inter = dummies_a.len() * dummies_b.len();
                 let cols_full = cols_ab + cols_inter;
                 
                 let x_full = DMatrix::from_vec(n, cols_full, full_data_vec);
                 
                 let res_a = fit_ols(&y_dvec, &x_a);
                 let res_b = fit_ols(&y_dvec, &x_b);
                 let res_ab = fit_ols(&y_dvec, &x_ab);
                 let res_full = fit_ols(&y_dvec, &x_full);
                 
                 if let (Ok((sse_a, _)), Ok((sse_b, _)), Ok((sse_ab, _)), Ok((sse_full, df_resid_full))) = (res_a, res_b, res_ab, res_full) {
                     
                     let ss_a = sse_b - sse_ab;
                     let ss_b = sse_a - sse_ab;
                     let ss_axb = sse_ab - sse_full;
                     let ss_error = sse_full;
                     
                     let k_a = dummies_a.len();
                     let k_b = dummies_b.len();
                     let df_a = k_a as f64;
                     let df_b = k_b as f64;
                     let df_axb = (k_a * k_b) as f64;
                     let df_error = df_resid_full as f64;
                     
                     let ms_a = ss_a / df_a;
                     let ms_b = ss_b / df_b;
                     let ms_axb = ss_axb / df_axb;
                     let ms_error = ss_error / df_error;
                     
                     let f_a = ms_a / ms_error;
                     let f_b = ms_b / ms_error;
                     let f_axb = ms_axb / ms_error;
                     
                     let get_p = |f, df1, df2| {
                         match FisherSnedecor::new(df1, df2) {
                            Ok(dist) => 1.0 - dist.cdf(f),
                            Err(_) => f64::NAN,
                         }
                     };
                     
                     let p_a = get_p(f_a, df_a, df_error);
                     let p_b = get_p(f_b, df_b, df_error);
                     let p_axb = get_p(f_axb, df_axb, df_error);
                     
                     let interpret = |p| if p < 0.05 { "有意 (*)" } else { "有意ではない" };

                     set_result_summary.set(Some(vec![
                        format!("--- Two-Way ANOVA (Type II) Result ---"),
                        format!("Factor A ({}): F={:.4}, p={:.4} ({})", f1, f_a, p_a, interpret(p_a)),
                        format!("Factor B ({}): F={:.4}, p={:.4} ({})", f2, f_b, p_b, interpret(p_b)),
                        format!("Interaction: F={:.4}, p={:.4} ({})", f_axb, p_axb, interpret(p_axb)),
                    ]));
                     
                     set_interpretation.set(format!(
                        "【解釈の補助】\n要因「{}」の主効果は{} (p={:.3})。\n要因「{}」の主効果は{} (p={:.3})。\n交互作用は{} (p={:.3})。",
                         f1, interpret(p_a), p_a,
                         f2, interpret(p_b), p_b,
                         interpret(p_axb), p_axb
                    ));

                    // Visualization: Grouped Bar Chart
                    // X axis: Factor A, Colors: Factor B
                    let levels_a: Vec<_> = f1_data.iter().collect::<HashSet<_>>().into_iter().collect();
                    let levels_b: Vec<_> = f2_data.iter().collect::<HashSet<_>>().into_iter().collect();
                    let mut traces = vec![];
                    
                    for lb in levels_b {
                        let mut x_vals = vec![];
                        let mut y_vals = vec![];
                        for la in &levels_a {
                             // Calculate mean for combination (la, lb)
                             let mut sum = 0.0;
                             let mut count = 0;
                             for i in 0..n {
                                 if &f1_data[i] == *la && &f2_data[i] == lb {
                                     sum += y_data[i];
                                     count += 1;
                                 }
                             }
                             if count > 0 {
                                 x_vals.push(la.to_string());
                                 y_vals.push(sum / count as f64);
                             }
                        }
                        traces.push(json!({
                            "type": "bar",
                            "name": lb,
                            "x": x_vals,
                            "y": y_vals
                        }));
                    }
                    
                    let layout_plot = json!({
                        "title": "Grouped Means",
                         "barmode": "group",
                         "xaxis": { "title": f1 },
                         "yaxis": { "title": target }
                    });
                     draw_plot(serde_json::Value::Array(traces), layout_plot);

                 } else {
                      set_result_summary.set(Some(vec!["Error fitting OLS models for Two-Way".to_string()]));
                 }
                 
            }
        }
    };

    view! {
        <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-layer-group"></i></div>
                "分散分析 (統合版)"
            </h2>
             <div class="description-box">
                <p>"※ 注: 二要因分散分析は Type II Sum of Squares アルゴリズムを使用しています。SPSSのデフォルト(Type III)とは異なる場合があります。"</p>
            </div>
            
             <div class="control-panel">
                <div class="radio-group">
                    <label>
                        <input type="radio" name="anova_type" 
                             on:click=move |_| set_test_type.set(AnovaType::OneWay)
                             checked=move || test_type.get() == AnovaType::OneWay
                        />
                        "一要因 (One-Way)"
                    </label>
                    <label>
                         <input type="radio" name="anova_type" 
                             on:click=move |_| set_test_type.set(AnovaType::TwoWay)
                             checked=move || test_type.get() == AnovaType::TwoWay
                        />
                        "二要因 (Two-Way)"
                    </label>
                </div>

                 <div class="input-group">
                    <label>"従属変数 (数値)"</label>
                    <select on:change=move |ev| set_target_col.set(event_target_value(&ev))>
                        <option value="">"選択してください"</option>
                        {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                    </select>
                
                    <label>"因子1 (カテゴリ)"</label>
                    <select on:change=move |ev| set_factor1_col.set(event_target_value(&ev))>
                        <option value="">"選択してください"</option>
                        {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                    </select>

                    {move || match test_type.get() {
                        AnovaType::TwoWay => view! {
                            <div class="input-group">
                                <label>"因子2 (カテゴリ)"</label>
                                <select on:change=move |ev| set_factor2_col.set(event_target_value(&ev))>
                                    <option value="">"選択してください"</option>
                                    {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                                </select>
                            </div>
                        }.into_view(),
                        _ => view! { <div/> }.into_view()
                    }}
                </div>
                
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
                         <div class="interpretation-box" style="margin-top: 20px; padding: 15px; background-color: #f9f9f9; border-left: 5px solid #007bff;">
                             <h4 style="margin-top: 0;">"解釈の補助"</h4>
                             <p style="white-space: pre-wrap;">{interpretation.get()}</p>
                         </div>
                    }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }}

                <div id=plot_id style="width: 100%; height: 400px; margin-top: 20px;"></div>
            </div>
        </div>
    }
}
