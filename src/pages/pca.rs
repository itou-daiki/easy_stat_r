use leptos::*;
use crate::state::AppData;
use nalgebra::{DMatrix, DVector, SymmetricEigen};

#[component]
pub fn Pca() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");
    
    // UI State
    let (target_cols, set_target_cols) = create_signal(std::collections::HashSet::<String>::new());
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

    let calculate = move |_| {
         let df_opt = app_data.df.get_untracked();
        if df_opt.is_none() { return; }
        let df = df_opt.unwrap();
        
        let cols_set = target_cols.get();
        let cols_vec: Vec<String> = cols_set.into_iter().collect();
        
        if cols_vec.len() < 2 {
             set_result_summary.set(Some(vec!["Error: Select at least 2 variables.".to_string()]));
             return;
        }

        // 1. Extract Data & Standardize
        let n_vars = cols_vec.len();
        let mut n_samples = 0;

        // First pass: validation and constructing matrix
        let mut col_vecs = vec![];
        for col_name in &cols_vec {
            if let Some(s) = df.column(col_name).ok().and_then(|c| c.f64().ok()) {
                let v: Vec<f64> = s.into_no_null_iter().collect();
                if n_samples == 0 { n_samples = v.len(); }
                if v.len() != n_samples {
                     set_result_summary.set(Some(vec![format!("Length mismatch: {}", col_name)]));
                     return;
                }
                col_vecs.push(v);
            } else {
                 set_result_summary.set(Some(vec![format!("{} is not numeric", col_name)]));
                 return;
            }
        }
        
        if n_samples < 2 {
              set_result_summary.set(Some(vec!["Error: Not enough data".to_string()]));
              return;
        }

        // Standardize (Z-score)
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

        // Calculate Correlation Matrix (R = Z'Z / (n-1))
        // nalgebra DMatrix is column-major.
        // We want (n_samples x n_vars).
        // Flatten column by column.
        let mut flat_z = vec![]; // [col1...col1, col2...col2]
        for z in &z_cols {
            flat_z.extend(z.iter().cloned());
        }
        
        let z_mat = DMatrix::from_vec(n_samples, n_vars, flat_z);
        let r_mat = (z_mat.transpose() * &z_mat) / (n_samples - 1) as f64;
        
        // 2. Eigen Decomposition
        let eig = SymmetricEigen::new(r_mat);
        let eigenvalues = eig.eigenvalues;
        let eigenvectors = eig.eigenvectors;
        
        // Sort by eigenvalue descending
        let mut pairs: Vec<(f64, DVector<f64>)> = eigenvalues.iter().zip(eigenvectors.column_iter())
            .map(|(val, vec)| (*val, vec.into_owned()))
            .collect();
            
        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        let total_var = n_vars as f64; // Sum of eigenvalues of correlation matrix = number of variables
        let mut summary_lines = vec![];
        summary_lines.push("--- PCA Results ---".to_string());
        summary_lines.push(format!("Variables: {:?}", cols_vec));
        summary_lines.push("".to_string());
        summary_lines.push("Eigenvalues & Explained Variance:".to_string());
        
        let mut cumul_var = 0.0;
        
        for (i, (val, vec)) in pairs.iter().enumerate() {
            let variance_ratio = val / total_var;
            cumul_var += variance_ratio;
            
            summary_lines.push(format!("PC{}: Eigenvalue={:.4}, Explained={:.2}%, Cumulative={:.2}%", 
                i+1, val, variance_ratio * 100.0, cumul_var * 100.0));
            
            // Loadings (Eigenvector * sqrt(Eigenvalue))
            // Only show for first few components
            if i < 3 {
                 let loading: DVector<f64> = vec * val.sqrt();
                 let loading_strs: Vec<String> = loading.iter().map(|v| format!("{:.3}", v)).collect();
                 summary_lines.push(format!("  Loadings: {:?}", loading_strs));
            }
        }
        
        set_result_summary.set(Some(summary_lines));
    };

    view! {
        <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-compress-arrows-alt"></i></div>
                "主成分分析 (PCA)"
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
