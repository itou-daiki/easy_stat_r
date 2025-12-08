use leptos::*;
use crate::state::AppData;
use nalgebra::{DMatrix, DVector, SymmetricEigen};
use std::collections::HashSet;
use serde_json::json;

#[component]
pub fn Pca() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    // UI State
    let (target_cols, set_target_cols) = create_signal(HashSet::<String>::new());
    let (result_summary, set_result_summary) = create_signal(Option::<Vec<String>>::None);

    let plot_id = "pca_scree_plot";

     let columns = create_memo(move |_| {
        if let Some(df) = app_data.df.get() {
            df.get_column_names().into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let draw_plot = move |data: serde_json::Value, layout: serde_json::Value| {
        #[cfg(target_arch = "wasm32")]
        {
             let d_str = data.to_string();
             let l_str = layout.to_string();
             let _ = js_sys::eval(&format!("window.drawPlot('{}', '{}', '{}')", plot_id, d_str, l_str));
        }
    };

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
        
        let mut flat_z = vec![]; 
        for z in &z_cols { flat_z.extend(z.iter().cloned()); }
        let z_mat = DMatrix::from_vec(n_samples, n_vars, flat_z);
        let r_mat = (z_mat.transpose() * &z_mat) / (n_samples - 1) as f64;
        
        let eig = SymmetricEigen::new(r_mat);
        let mut pairs: Vec<(f64, DVector<f64>)> = eig.eigenvalues.iter().zip(eig.eigenvectors.column_iter())
            .map(|(val, vec)| (*val, vec.into_owned()))
            .collect();
        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Scree Plot
        let eigen_vals: Vec<f64> = pairs.iter().map(|p| p.0).collect();
        let x_axis: Vec<usize> = (1..=n_vars).collect();
        
        let data_plot = json!([
             {
                "x": x_axis,
                "y": eigen_vals,
                "mode": "lines+markers",
                "type": "scatter",
                "name": "Eigenvalues",
                "marker": { "color": "#2ca02c" }
            },
             {
                 "x": [1, n_vars],
                 "y": [1, 1],
                 "mode": "lines",
                 "type": "scatter",
                 "name": "Eigenvalue=1",
                 "line": { "dash": "dash", "color": "gray" }
            }
        ]);
        let layout_plot = json!({
            "title": "Scree Plot",
            "xaxis": { "title": "Component Number" },
            "yaxis": { "title": "Eigenvalue" },
            "margin": { "t": 40, "b": 40, "l": 50, "r": 20 }
        });
        draw_plot(data_plot, layout_plot);

        // Results Table
        let mut result_lines = vec![];
        result_lines.push("--- Principal Component Analysis (PCA) ---".to_string());
        result_lines.push("Eigenvalues & Explained Variance:".to_string());
        
        let total_variance: f64 = pairs.iter().map(|p| p.0).sum();
        let mut cumulative_var = 0.0;
        
        for (i, (eig_val, vec)) in pairs.iter().enumerate() {
            let variance_ratio = eig_val / total_variance;
            cumulative_var += variance_ratio;
            
            result_lines.push(format!("PC{}: Eigen={:.4}, Var={:.2}%, Cum={:.2}%", 
                i+1, eig_val, variance_ratio * 100.0, cumulative_var * 100.0));
            
             // Loadings for top 3 components usually shown
             if i < 3 {
                 let _loading: Vec<String> = vec.iter().map(|x| format!("{:.3}", x)).collect();
                 // result_lines.push(format!("  Loadings: {:?}", loading)); 
             }
        }
        
        set_result_summary.set(Some(result_lines));
    };

    view! {
         <div class="fade-in">
             <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-microchip"></i></div>
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
                
                 <div id=plot_id style="width: 100%; height: 400px; margin-top: 20px;"></div>
            </div>
        </div>
    }
}
