use leptos::*;
use crate::state::AppData;
use polars::prelude::*;

#[component]
pub fn Eda() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    view! {
        <div class="fade-in">
            <h2 class="section-title">
                <div class="section-icon"><i class="fas fa-search"></i></div>
                "探索的データ分析 (EDA)"
            </h2>

            {move || match app_data.df.get() {
                Some(df) => {
                    let summary = calculate_summary(&df);
                    view! {
                        <div>
                            <h3 class="subsection-title">"データフレーム"</h3>
                            {render_dataframe(df.clone())}
                            <h3 class="subsection-title">"要約統計量"</h3>
                            {render_dataframe(summary)}
                        </div>
                    }.into_view()
                }
                None => {
                    view! {
                        <p>"データを表示するには、ファイルをアップロードしてください。"</p>
                    }.into_view()
                }
            }}
        </div>
    }
}

fn calculate_summary(df: &DataFrame) -> DataFrame {
    let mut summary_cols = vec![];

    let descriptions = Series::new("description", vec!["count", "mean", "std", "min", "max"]);
    summary_cols.push(descriptions);

    for col_name in df.get_column_names() {
        let series = df.column(col_name).unwrap();
        
        let (count, mean, std, min, max) = if series.dtype().is_numeric() {
            let series_f64 = series.to_physical_repr().cast(&DataType::Float64).unwrap();
            let s_f64 = series_f64.f64().unwrap();
            
            let count = s_f64.len() as f64;
            let mean = s_f64.mean().unwrap_or(0.0);
            let std = s_f64.std(1).unwrap_or(0.0);
            let min = s_f64.min().unwrap_or(0.0);
            let max = s_f64.max().unwrap_or(0.0);

            (count, mean, std, min, max)
        } else {
            (series.len() as f64, 0.0, 0.0, 0.0, 0.0)
        };

        let summary_series = Series::new(col_name, vec![count, mean, std, min, max]);
        summary_cols.push(summary_series);
    }

    DataFrame::new(summary_cols).unwrap()
}

fn render_dataframe(df: DataFrame) -> impl IntoView {
    let headers = df.get_column_names().into_iter().map(|name| view! { <th>{name.to_string()}</th> }).collect::<Vec<_>>();
    let rows = df.iter().map(|row| {
        let cells = row.iter().map(|value| {
            let val_str = match value {
                AnyValue::Float32(v) => format!("{:.2}", v),
                AnyValue::Float64(v) => format!("{:.2}", v),
                AnyValue::Null => "".to_string(),
                _ => format!("{}", value),
            };
            view! { <td>{val_str}</td> }
        }).collect::<Vec<_>>();
        view! { <tr>{cells}</tr> }
    }).collect::<Vec<_>>();

    view! {
        <div class="table-container">
            <table class="dataframe">
                <thead>
                    <tr>{headers}</tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>
    }
}
