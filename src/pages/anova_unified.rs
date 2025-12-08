use leptos::*;
use crate::state::AppData;
use statrs::distribution::{FisherSnedecor, ContinuousCDF};
use nalgebra::{DMatrix, DVector};
use std::collections::HashSet;

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

     let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Helper: OLS fitting
    // Returns SSE (Sum of Squared Errors) and df_resid
    fn fit_ols(y: &DVector<f64>, x: &DMatrix<f64>) -> Result<(f64, usize), String> {
        // beta = (X^T X)^-1 X^T y
        let xt = x.transpose();
        let xtx = &xt * x;
        let xty = &xt * y;
        
        // Solve with Cholesky or LU. xtx is symmetric positive definite generally.
        // Using cholesky:
        let chol = nalgebra::linalg::Cholesky::new(xtx.clone());
        if let Some(decomp) = chol {
            let beta = decomp.solve(&xty);
            let y_pred = x * &beta;
            let residuals = y - y_pred;
            let sse = residuals.dot(&residuals);
            let df_resid = y.len() - x.ncols();
            Ok((sse, df_resid))
        } else {
            Err("Matrix singular or ill-conditioned".to_string())
        }
    }

    // Helper: Create Dummy Matrix (One-Hot excluding first level to avoid collinearity with intercept, or use Contrast coding)
    // For Type II SS, we typically use "Sum Contrasts" or just dummy with care. 
    // Actually, for Type II SS comparison of models, simple dummy coding (Treatment coding) is sufficient IF we compare correct models.
    // e.g. SS(A|B) = SSE(B) - SSE(A+B). Both B and A+B models will interpret dummies consistently.
    fn create_design_matrix(
        rows: usize, 
        factors: Vec<(&String, &Vec<String>)> // (ColName, Data)
    ) -> DMatrix<f64> {
        // Implementation of basic dummy coding
        // Always include Intercept column (all 1s)
        let mut mat_data = vec![1.0; rows]; // Col 0: Intercept
        let mut cols = 1;

        // For each factor, find unique levels, sort them. 
        // Create k-1 columns (k = levels).
        for (_, values) in factors {
            let mut unique_levels: Vec<_> = values.iter().collect::<HashSet<_>>().into_iter().collect();
            unique_levels.sort();
            
            // Skip first level as reference
            if unique_levels.len() > 1 {
                for level in &unique_levels[1..] {
                    // Create column
                    let col_vec: Vec<f64> = values.iter().map(|v| if v == *level { 1.0 } else { 0.0 }).collect();
                    mat_data.extend(col_vec);
                    cols += 1;
                }
            }
        }
        
        DMatrix::from_vec(rows, cols, mat_data)
    }
    
    // More complex: Interaction design matrix
    // A:B needs columns for each combination of levels (excluding reference).
    // Actually, easier to construct full model strings if using a formula library, but we don't have one in Rust Wasm easily.
    // Manual construction:
    // Interaction cols = (LevelA_i * LevelB_j). i \in [1..Ka], j \in [1..Kb] (skipping 0s).
    
    let calculate = move |_| {
         let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let target = target_col.get();
        let f1 = factor1_col.get();
        
        if target.is_empty() || f1.is_empty() { return; }
        
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
                // One-Way Logic (Simple F-test or OLS)
                // Model 0: Intercept only
                // Model 1: Intercept + F1
                
                // Design Matrix M0 (Intercept)
                let x0 = DMatrix::from_element(n, 1, 1.0);
                
                // Design Matrix M1 (Full)
                // Use a simplified helper for one factor
                // Find unique levels
                let mut levels: Vec<_> = f1_data.iter().collect::<HashSet<_>>().into_iter().collect();
                levels.sort();
                let k = levels.len();
                
                // Construct M1 manually to be safe
                let mut x1_data = vec![1.0; n];
                let mut x1_cols = 1;
                for lvl in &levels[1..] {
                   let col: Vec<f64> = f1_data.iter().map(|v| if v == *lvl { 1.0 } else { 0.0 }).collect();
                   x1_data.extend(col);
                   x1_cols += 1;
                }
                // row-major to col-major issue? DMatrix::from_vec takes column-major.
                // Our construction above: [Col0...Col0, Col1...Col1]. This is correct for from_vec.
                let x1 = DMatrix::from_vec(n, x1_cols, x1_data);

                let res0 = fit_ols(&y_dvec, &x0);
                let res1 = fit_ols(&y_dvec, &x1);
                
                if let (Ok((sse0, _)), Ok((sse1, df_resid1))) = (res0, res1) {
                    let df_total = n - 1;
                    let df_model = k - 1;
                    let _df_error = n - k; // same as df_resid1
                    
                    let ss_total = sse0; // approx (if mean is 0? No. SSE of intercept model IS SS_total centered)
                    // Correct: SSE(Intercept) = sum((y - mean)^2) = SSTotal.
                    
                    let ss_model = sse0 - sse1;
                    let ss_error = sse1;
                    
                    let ms_model = ss_model / df_model as f64;
                    let ms_error = ss_error / df_resid1 as f64;
                    
                    let f_val = ms_model / ms_error;
                    
                    let p_val = match FisherSnedecor::new(df_model as f64, df_resid1 as f64) {
                        Ok(dist) => 1.0 - dist.cdf(f_val),
                        Err(_) => f64::NAN,
                    };
                    
                     set_result_summary.set(Some(vec![
                        format!("--- One-Way ANOVA Result ---"),
                        format!("Factor: {} ({} levels)", f1, k),
                        format!("F-statistic: {:.4}", f_val),
                        format!("p-value: {:.4}", p_val),
                        format!("df(between): {}", df_model),
                        format!("df(within): {}", df_resid1),
                        format!("SS(between): {:.4}", ss_model),
                        format!("SS(within): {:.4}", ss_error),
                        format!("Significance: {}", if p_val < 0.05 { "Significant (*)" } else { "Not Significant" })
                    ]));

                } else {
                    set_result_summary.set(Some(vec!["Error fitting OLS models".to_string()]));
                }
            },
            AnovaType::TwoWay => {
                let f2 = factor2_col.get();
                if f2.is_empty() { return; }
                
                // Extract F2
                 let f2_vec_opt: Option<Vec<String>> = df.column(&f2).ok().map(|s| s.iter().map(|v| v.to_string().replace("\"", "")).collect());
                 if f2_vec_opt.is_none() { return; }
                 let f2_data = f2_vec_opt.unwrap();
                 
                 // 1. Prepare Data
                 // Factors: A=f1_data, B=f2_data
                 
                 // 2. Define Models for Type II SS
                 // SS(A|B) = SSE(B) - SSE(A+B)
                 // SS(B|A) = SSE(A) - SSE(A+B)
                 // SS(A:B) = SSE(A+B) - SSE(A+B+A:B)
                 // Residual = SSE(A+B+A:B)
                 
                 // Matrices needed:
                 // X_A  (for SSE(A) - NOT NEEDED for Type II? Wait. SS(B|A) needs SSE(A))
                 // X_B  (for SSE(B))
                 // X_AB (for SSE(A+B))
                 // X_Full (for SSE(A+B+A:B))
                 
                 let x_a = create_design_matrix(n, vec![(&f1, &f1_data)]);
                 let x_b = create_design_matrix(n, vec![(&f2, &f2_data)]);
                 let x_ab = create_design_matrix(n, vec![(&f1, &f1_data), (&f2, &f2_data)]);
                 
                 // Construct X_Full (X_AB + Interaction Dummies)
                 // Interaction Dummies: Product of DummyA_i * DummyB_j
                 // Need to reconstruct dummies to multiply them.
                 // Hack: Reuse create_design_matrix logic but return the dummy blocks.
                 // Easier: Just generate interaction cols manually here.
                 
                 fn get_dummies(_n: usize, data: &Vec<String>) -> Vec<Vec<f64>> {
                      let mut unique: Vec<_> = data.iter().collect::<HashSet<_>>().into_iter().collect();
                      unique.sort();
                      let mut dummies = vec![];
                      if unique.len() > 1 {
                          for lvl in &unique[1..] {
                              let col: Vec<f64> = data.iter().map(|v| if v == *lvl { 1.0 } else { 0.0 }).collect();
                              dummies.push(col);
                          }
                      }
                      dummies
                 }
                 
                 let dummies_a = get_dummies(n, &f1_data);
                 let dummies_b = get_dummies(n, &f2_data);
                 
                 // Create Interaction Cols
                 let mut interaction_cols = vec![];
                 for col_a in &dummies_a {
                     for col_b in &dummies_b {
                         let inter: Vec<f64> = col_a.iter().zip(col_b.iter()).map(|(a, b)| a * b).collect();
                         interaction_cols.push(inter);
                     }
                 }
                 
                 // Assemble X_Full
                 // X_AB columns + Interaction Cols
                 
                 let mut full_data_vec = x_ab.data.as_vec().clone(); // This is column-major vector of X_AB
                 // Append interaction cols
                 for col in interaction_cols {
                     full_data_vec.extend(col);
                 }
                 let cols_ab = x_ab.ncols();
                 let cols_inter = dummies_a.len() * dummies_b.len();
                 let cols_full = cols_ab + cols_inter;
                 
                 let x_full = DMatrix::from_vec(n, cols_full, full_data_vec);
                 
                 // Run OLS
                 let res_a = fit_ols(&y_dvec, &x_a);
                 let res_b = fit_ols(&y_dvec, &x_b);
                 let res_ab = fit_ols(&y_dvec, &x_ab);
                 let res_full = fit_ols(&y_dvec, &x_full);
                 
                 if let (Ok((sse_a, _)), Ok((sse_b, _)), Ok((sse_ab, df_resid_ab)), Ok((sse_full, df_resid_full))) = (res_a, res_b, res_ab, res_full) {
                     
                     // SS Calculations
                     let ss_a = sse_b - sse_ab;
                     let ss_b = sse_a - sse_ab;
                     let ss_axb = sse_ab - sse_full;
                     let ss_error = sse_full;
                     
                     // Degrees of Freedom
                     let k_a = dummies_a.len();
                     let k_b = dummies_b.len();
                     let df_a = k_a as f64;
                     let df_b = k_b as f64;
                     let df_axb = (k_a * k_b) as f64;
                     let df_error = df_resid_full as f64;
                     
                     // MS
                     let ms_a = ss_a / df_a;
                     let ms_b = ss_b / df_b;
                     let ms_axb = ss_axb / df_axb;
                     let ms_error = ss_error / df_error;
                     
                     // F-ratios
                     let f_a = ms_a / ms_error;
                     let f_b = ms_b / ms_error;
                     let f_axb = ms_axb / ms_error;
                     
                     // P-values
                     let get_p = |f, df1, df2| {
                         match FisherSnedecor::new(df1, df2) {
                            Ok(dist) => 1.0 - dist.cdf(f),
                            Err(_) => f64::NAN,
                         }
                     };
                     
                     let p_a = get_p(f_a, df_a, df_error);
                     let p_b = get_p(f_b, df_b, df_error);
                     let p_axb = get_p(f_axb, df_axb, df_error);
                     
                     set_result_summary.set(Some(vec![
                        format!("--- Two-Way ANOVA (Type II) Result ---"),
                        format!("Factor A: {}", f1),
                        format!("  F={:.4}, p={:.4}, df={}, SS={:.4}", f_a, p_a, df_a, ss_a),
                        format!("Factor B: {}", f2),
                        format!("  F={:.4}, p={:.4}, df={}, SS={:.4}", f_b, p_b, df_b, ss_b),
                        format!("Interaction A:B"),
                        format!("  F={:.4}, p={:.4}, df={}, SS={:.4}", f_axb, p_axb, df_axb, ss_axb),
                        format!("Error"),
                        format!("  df={}, SS={:.4}, MS={:.4}", df_error, ss_error, ms_error)
                    ]));
                     
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
            </div>
        </div>
    }
}
