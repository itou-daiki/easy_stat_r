use leptos::*;
use crate::state::AppData;
use nalgebra::{DMatrix, DVector, RowDVector, SymmetricEigen};
use std::collections::HashSet;

#[component]
pub fn FactorAnalysis() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    // UI State
    let (target_cols, set_target_cols) = create_signal(HashSet::<String>::new());
    let (n_factors, set_n_factors) = create_signal(2); // Default to 2 factors
    let (do_rotation, set_do_rotation) = create_signal(true); // Default Varimax
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None);

     let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

     let toggle_col = move |col: String| {
        set_target_cols.update(|cols| {
            if cols.contains(&col) {
                cols.remove(&col);
            } else {
                cols.insert(col);
            }
        });
    };

    // Varimax Rotation Helper
    // maximizing sum of variances of squared loadings
    fn varimax(loadings: &DMatrix<f64>, max_iter: usize, tol: f64) -> DMatrix<f64> {
        let (p, k) = loadings.shape();
        let mut r = DMatrix::identity(k, k);
        let mut d = 0.0;
        
        
        for _ in 0..max_iter {
            let d_old = d;
            
            // Lambda = Loadings * R
            let lambda = loadings * &r;
            
            // Transform logic: standard varimax Kaiser normalization usually done? 
            // Simplified "raw" varimax:
            // U, S, V = SVD of (Lambda^3 - Lambda * diag(sum(Lambda^2)/p))' * Lambda
            
            // Let's implement simpler gradient-like approach or standard Procrustes-like step
            // Reference: "The varimax criterion for analytic rotation in factor analysis" (Kaiser 1958)
            // Or use the simplified matrix iteration:
            // A = Lambda
            // B = A .^ 3 - A * diag(sum(sq, 0)/p)
            // U, S, V = SVD(A' B)
            // R = U * V'
            
            // nalgebra SVD
            let lambda_sq = lambda.map(|x| x*x);
             // nalgebra: sum_rows -> DVector (col vector of sums of each row).
             // sum_columns -> RowDVector.
             let c_sums = lambda_sq.column_sum(); // 1 x k
             
             let alpha = c_sums.map(|x| x / p as f64);
             
             // Construct A * diag(...)
             // Multiply each column j by alpha[j]
             let mut term2 = lambda.clone();
             for j in 0..k {
                 let s = alpha[j];
                 for i in 0..p {
                     term2[(i,j)] *= s;
                 }
             }
             
             let b = lambda.map(|x| x.powi(3)) - term2;
             
             let m = loadings.transpose() * b;
             
             let svd = m.svd(true, true);
             if let (Some(u), Some(v_t)) = (svd.u, svd.v_t) {
                 // R_new = U * V'
                 let r_new = u * v_t;
                 d = svd.singular_values.sum(); // Trace of singular values? No, simple convergence check
                 r = r_new;
             } else {
                 break; 
             }
             
             if (d - d_old).abs() < tol {
                 break;
             }
        }
        
        loadings * r
    }

    let calculate = move |_| {
         let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let cols_set = target_cols.get();
        let cols_vec: Vec<String> = cols_set.into_iter().collect();
        let k_factors = n_factors.get();
        
        if cols_vec.len() < k_factors {
             set_result_summary.set(Some(vec!["Error: Number of factors cannot exceed variables.".to_string()]));
             return;
        }

        // Data prep (Correlation Matrix) - reused from PCA logic essentially
        let n_vars = cols_vec.len();
        let mut col_vecs = vec![];
        let mut n_samples = 0;
        
         for col_name in &cols_vec {
            if let Some(s) = df.column(col_name).ok().and_then(|c| c.f64().ok()) {
                let v: Vec<f64> = s.into_no_null_iter().collect();
                if n_samples == 0 { n_samples = v.len(); }
                if v.len() != n_samples { return; }
                col_vecs.push(v);
            } else { return; }
        }
        
         // Standardize
        let mut z_cols = vec![];
        for v in col_vecs {
            let mean = v.iter().sum::<f64>() / n_samples as f64;
            let variance = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_samples - 1) as f64;
            let std_dev = variance.sqrt();
             if std_dev == 0.0 {
                 set_result_summary.set(Some(vec!["Error: Variable has zero variance".to_string()]));
                 return;
            }
            let z: Vec<f64> = v.iter().map(|x| (x - mean) / std_dev).collect();
            z_cols.push(z);
        }
        
        // Corr Matrix
        let mut flat_z = vec![]; 
        for z in &z_cols { flat_z.extend(z.iter().cloned()); }
        let z_mat = DMatrix::from_vec(n_samples, n_vars, flat_z);
        let r_mat = (z_mat.transpose() * &z_mat) / (n_samples - 1) as f64;
        
        // Eigen Decomp (Principal Component Method)
        let eig = SymmetricEigen::new(r_mat);
        let mut pairs: Vec<(f64, DVector<f64>)> = eig.eigenvalues.iter().zip(eig.eigenvectors.column_iter())
            .map(|(val, vec)| (*val, vec.into_owned()))
            .collect();
        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Extract top K factors
        // Unrotated Matrix L = V * sqrt(Lambda)
        let mut l_data = vec![];
        for j in 0..k_factors {
            let (val, vec) = &pairs[j];
            if *val <= 0.0 { continue; } // Handle error
            let scaled_vec = vec * val.sqrt();
            l_data.extend(scaled_vec.iter().cloned());
        }
        
        if l_data.len() != n_vars * k_factors {
             set_result_summary.set(Some(vec!["Error extracting factors".to_string()]));
             return;
        }
        
        // L is n_vars x k_factors. DMatrix::from_vec uses column-major.
        let l_unrotated = DMatrix::from_vec(n_vars, k_factors, l_data);
        
        let final_loadings = if do_rotation.get() {
            varimax(&l_unrotated, 50, 1e-6)
        } else {
            l_unrotated
        };
        
        let mut lines = vec![];
        lines.push(format!("--- Factor Analysis Results ({} Factors) ---", k_factors));
        lines.push(format!("Method: Principal Component, Rotation: {}", if do_rotation.get() { "Varimax" } else { "None" }));
        lines.push("".to_string());
        lines.push("Factor Loadings:".to_string());
        
        for (i, name) in cols_vec.iter().enumerate() {
            let row = final_loadings.row(i);
            let s: Vec<String> = row.iter().map(|x| format!("{:.3}", x)).collect();
            lines.push(format!("{}: {:?}", name, s));
        }

        set_result_summary.set(Some(lines));
    };

    view! {
         <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-project-diagram"></i></div>
                "因子分析 (Factor Analysis)"
            </h2>
            
             <div class="control-panel">
                <div class="input-group">
                    <label>"分析対象変数 (複数選択)"</label>
                     <div class="checkbox-list" style="max_height: 200px; overflow-y: auto; border: 1px solid #ccc; padding: 5px;">
                        {move || columns.get().into_iter().map(|c| {
                            let c_clone = c.clone();
                            view! {
                                <div style="margin-bottom: 4px;">
                                    <label style="display: flex; align-items: center; cursor: pointer;">
                                        <input type="checkbox" 
                                            value=c_clone.clone()
                                            on:change=move |_| toggle_col(c_clone.clone())
                                            checked=target_cols.get().contains(&c)
                                            style="margin-right: 8px;"
                                        />
                                        {c}
                                    </label>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                
                 <div class="input-group">
                    <label>"抽出する因子数"</label>
                    <input type="number" min="1" max="10" 
                        value=move || n_factors.get()
                        on:change=move |ev| set_n_factors.set(event_target_value(&ev).parse().unwrap_or(2))
                    />
                 </div>
                 
                 <div class="input-group">
                    <label>
                        <input type="checkbox"
                            on:change=move |ev| set_do_rotation.set(event_target_checked(&ev))
                            checked=move || do_rotation.get()
                        />
                        "バリマックス回転を行う"
                    </label>
                 </div>
                
                 <button class="primary-btn" on:click=calculate>
                    "分析を実行"
                </button>
            </div>
            
             <div class="result-area">
                {move || result_summary.get().map(|lines| view! {
                    <div class="result-box">
                        <h3>"分析結果"</h3>
                         <ul style="list-style-type: none; padding: 0;">
                            {lines.into_iter().map(|l| view! { <li style="padding: 4px 0; border-bottom: 1px solid #eee;">{l}</li> }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                })}
            </div>
        </div>
    }
}
