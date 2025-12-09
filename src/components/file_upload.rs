use leptos::*;
use web_sys::{HtmlInputElement, FileReader};
use wasm_bindgen::JsCast;
use crate::state::AppData;
use crate::utils::excel_to_polars::convert_excel_to_df;
use polars::prelude::*;
use std::io::Cursor;

#[component]
pub fn FileUpload() -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");
    // Clone for the view closure
    let app_data_for_view = app_data.clone();

    let on_file_change = move |ev: ev::Event| {
        // AppData is moved here
        let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
        
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let file_name = file.name();
                app_data.file_name.set(file_name.clone());
                app_data.error_msg.set(None);

                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let file_name_clone = file_name.clone();


                let on_load_handler = move |_: web_sys::Event| {
                    let result_js = reader_clone.result().unwrap();
                    let array_buffer = result_js.dyn_into::<js_sys::ArrayBuffer>().unwrap();
                    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                    let bytes = uint8_array.to_vec();

                    let df_result = if file_name_clone.ends_with(".csv") {
                        CsvReader::new(Cursor::new(bytes))
                            .finish()
                            .map_err(|e| anyhow::anyhow!("CSV Parse Error: {}", e))
                    } else if file_name_clone.ends_with(".xlsx") || file_name_clone.ends_with(".xls") {
                        convert_excel_to_df(&bytes)
                    } else {
                        Err(anyhow::anyhow!("Unsupported file format"))
                    };

                    match df_result {
                        Ok(df) => {
                            app_data.df.set(Some(df));
                        },
                        Err(e) => {
                            app_data.error_msg.set(Some(format!("Error: {}", e)));
                            app_data.df.set(None);
                        }
                    }
                };

                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(on_load_handler) as Box<dyn FnMut(_)>);
                reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                closure.forget(); // Leak memory for simplicity in this example, or manage cleanup properly

                reader.read_as_array_buffer(&file).unwrap();
            }
        }
    };

    view! {
        <div class="file-upload-container" style="margin-bottom: 2rem; padding: 2rem; border: 2px dashed var(--border-color); border-radius: 12px; text-align: center; background: var(--surface);">
            <i class="fas fa-cloud-upload-alt" style="font-size: 3rem; color: var(--primary-color); margin-bottom: 1rem;"></i>
            <h3 style="margin-bottom: 1rem; color: var(--text-primary);">"データをアップロード"</h3>
            <p style="color: var(--text-secondary); margin-bottom: 1.5rem;">"CSV または Excel (.xlsx) ファイルを選択してください"</p>
            
            <label class="link-button" style="cursor: pointer; display: inline-block;">
                "ファイルを選択"
                <input 
                    type="file" 
                    accept=".csv,.xlsx,.xls" 
                    on:change=on_file_change
                    style="display: none;"
                />
            </label>

            {move || {
                if let Some(err) = app_data_for_view.error_msg.get() {
                    view! { <div style="color: red; margin-top: 1rem;">{err}</div> }.into_view()
                } else if !app_data_for_view.file_name.get().is_empty() {
                    view! { <div style="color: green; margin-top: 1rem;">"選択されたファイル: " {app_data_for_view.file_name.get()}</div> }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }
            }}
        </div>
    }
}
