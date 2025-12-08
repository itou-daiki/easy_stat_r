use leptos::*;
use crate::state::AppData;
use nalgebra::{DMatrix, DVector};
use statrs::distribution::{FisherSnedecor, StudentsT, ContinuousCDF};
use std::collections::HashSet;

#[component]
pub fn Regression() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    // UI Signals
    let (target_col, set_target_col) = create_signal(String::new());
    let (explanatory_cols, set_explanatory_cols) = create_signal(HashSet::<String>::new());
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None);
    let (formula_display, set_formula_display) = create_signal(String::new());

    // Available columns
    let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Toggle explanatory variable
    let toggle_explanatory = move |col: String| {
        set_explanatory_cols.update(|cols| {
            if cols.contains(&col) {
                cols.remove(&col);
            } else {
                cols.insert(col);
            }
        });
    };

    // Calculation Logic
    let calculate = move |_| {
        let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let target = target_col.get();
        let expl_set = explanatory_cols.get();
        let expl_vec: Vec<String> = expl_set.into_iter().collect();
        
        if target.is_empty() || expl_vec.is_empty() {
            set_result_summary.set(Some(vec!["Error: Select target and at least one explanatory variable.".to_string()]));
            return;
        }

        // 1. Data Extraction
        // Target Y
        let y_res = df.column(&target).ok().and_then(|c| c.f64().ok()).map(|s| s.into_no_null_iter().collect::<Vec<f64>>());
        if y_res.is_none() {
             set_result_summary.set(Some(vec!["Error extracting target variable (must be numeric)".to_string()]));
             return;
        }
        let y_data = y_res.unwrap();
        let n = y_data.len();
        let y_dvec = DVector::from_vec(y_data.clone());

        // Explanatory X
        // We need to construct Design Matrix X: [1, x1, x2...]
        let mut x_data_flat = vec![];
        // Intercept column
        let mut intercept = vec![1.0; n];
        x_data_flat.append(&mut intercept);
        
        let mut valid_expl_names = vec![];
        
        for name in &expl_vec {
             if let Some(col_data) = df.column(name).ok().and_then(|c| c.f64().ok()) {
                 let mut data_vec: Vec<f64> = col_data.into_no_null_iter().collect();
                 if data_vec.len() != n {
                      set_result_summary.set(Some(vec![format!("Error: Length mismatch for {}", name)]));
                      return;
                 }
                 x_data_flat.append(&mut data_vec);
                 valid_expl_names.push(name.clone());
             } else {
                  set_result_summary.set(Some(vec![format!("Error: {} is not numeric", name)]));
                  return;
             }
        }
        
        let k = valid_expl_names.len(); // number of predictors (excluding intercept)
        let p = k + 1; // total parameters (including intercept)
        
        if n <= p {
             set_result_summary.set(Some(vec!["Error: Not enough data points (n <= k + 1)".to_string()]));
             return;
        }

        let x_mat = DMatrix::from_vec(n, p, x_data_flat);

        // 2. OLS Solver
        // beta = (X'X)^-1 X'Y
        let xt = x_mat.transpose();
        let xtx = &xt * &x_mat;
        let xty = &xt * &y_dvec;

        // Invert X'X
        // Use Cholesky or LU. If fails -> Multicollinearity
        let xtx_inv_opt = xtx.try_inverse(); // nalgebra try_inverse uses LU
        
        if let Some(xtx_inv) = xtx_inv_opt {
            let beta = &xtx_inv * xty;
            
            // 3. Statistics
            let y_pred = &x_mat * &beta;
            let residuals = &y_dvec - &y_pred;
            let sse = residuals.dot(&residuals);
            
            // SST (Total Sum of Squares)
            let y_mean = y_data.iter().sum::<f64>() / n as f64;
            let sst: f64 = y_data.iter().map(|val| (val - y_mean).powi(2)).sum();
            
            // R-squared
            let r2 = 1.0 - (sse / sst);
            // Adj R-squared
            let adj_r2 = 1.0 - (1.0 - r2) * ((n - 1) as f64 / (n - p) as f64);
            
            // F-test
            // MSM = (SST - SSE) / k
            // MSE = SSE / (n - p)
            let msm = (sst - sse) / k as f64;
            let mse = sse / (n - p) as f64;
            let f_val = msm / mse;
            
            let f_dist = FisherSnedecor::new(k as f64, (n - p) as f64).unwrap();
            let p_val_f = 1.0 - f_dist.cdf(f_val);
            
            // Standard Errors for Coefficients
            // Var(beta) = MSE * (X'X)^-1
            // SE(beta_j) = sqrt(Var(beta)_jj)
            let var_beta = xtx_inv.scale(mse);
            
            let mut result_lines = vec![];
            result_lines.push(format!("--- Regression Results (Y: {}) ---", target));
            result_lines.push(format!("R²: {:.4}", r2));
            result_lines.push(format!("Adj R²: {:.4}", adj_r2));
            result_lines.push(format!("F({:.0}, {:.0}): {:.4}, p={:.4e}", k, n-p, f_val, p_val_f));
            result_lines.push("".to_string());
            result_lines.push("Coefficients:".to_string());
            
            let t_dist = StudentsT::new(0.0, 1.0, (n - p) as f64).unwrap();

            let mut formula_str = format!("{} = {:.4}", target, beta[0]);

            // Intercept
            let se_intercept = var_beta[(0,0)].sqrt();
            let t_intercept = beta[0] / se_intercept;
            let p_intercept = 2.0 * (1.0 - t_dist.cdf(t_intercept.abs()));
            
            result_lines.push(format!("Intercept: Coef={:.4}, SE={:.4}, t={:.4}, p={:.4}", beta[0], se_intercept, t_intercept, p_intercept));

            for i in 0..k {
                let name = &valid_expl_names[i];
                let val = beta[i+1];
                let se = var_beta[(i+1, i+1)].sqrt();
                let t = val / se;
                let p = 2.0 * (1.0 - t_dist.cdf(t.abs()));
                let sig = if p < 0.01 { "**" } else if p < 0.05 { "*" } else { "" };
                
                result_lines.push(format!("{}: Coef={:.4}, SE={:.4}, t={:.4}, p={:.4} {}", name, val, se, t, p, sig));
                
                let sign = if val >= 0.0 { "+" } else { "-" };
                formula_str.push_str(&format!(" {} {:.4}*{}", sign, val.abs(), name));
            }
            
            set_formula_display.set(formula_str);
            set_result_summary.set(Some(result_lines));

        } else {
             set_result_summary.set(Some(vec!["Error: Singular Matrix. Multicollinearity likely detected.".to_string()]));
        }
    };

    view! {
        <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-chart-line"></i></div>
                "回帰分析 (単回帰・重回帰)"
            </h2>
            
             <div class="control-panel">
                 <div class="input-group">
                    <label>"目的変数 (Y) [数値]"</label>
                    <select on:change=move |ev| set_target_col.set(event_target_value(&ev))>
                        <option value="">"選択してください"</option>
                        {columns.get().into_iter().map(|c| view! { <option value=c.clone()>{c}</option> }).collect::<Vec<_>>()}
                    </select>
                </div>

                <div class="input-group">
                    <label>"説明変数 (X) [数値] - 複数選択可"</label>
                    <div class="checkbox-list" style="max_height: 200px; overflow-y: auto; border: 1px solid #ccc; padding: 5px;">
                        {move || columns.get().into_iter().map(|c| {
                            let c_clone = c.clone();
                            view! {
                                <div style="margin-bottom: 4px;">
                                    <label style="display: flex; align-items: center; cursor: pointer;">
                                        <input type="checkbox" 
                                            value=c_clone.clone()
                                            on:change=move |_| toggle_explanatory(c_clone.clone())
                                            checked=explanatory_cols.get().contains(&c)
                                            style="margin-right: 8px;"
                                        />
                                        {c}
                                    </label>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                
                <button class="primary-btn" on:click=calculate>
                    "分析を実行"
                </button>
            </div>
            
             <div class="result-area">
                {move || if !formula_display.get().is_empty() {
                    view! {
                        <div class="result-box" style="margin-bottom: 1rem; background-color: #f0f7ff; border-left: 4px solid #007bff;">
                            <h3>"数理モデル"</h3>
                            <p style="font-family: monospace; font-size: 1.1em;">{formula_display.get()}</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }}

                {move || result_summary.get().map(|lines| view! {
                    <div class="result-box">
                        <h3>"分析結果詳細"</h3>
                        <ul style="list-style-type: none; padding: 0;">
                            {lines.into_iter().map(|l| view! { <li style="padding: 4px 0; border-bottom: 1px solid #eee;">{l}</li> }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                })}
            </div>
        </div>
    }
}
