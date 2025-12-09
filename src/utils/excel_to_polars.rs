use anyhow::{anyhow, Result};
use calamine::{Reader, Xlsx, open_workbook_from_rs, Data};
use polars::prelude::*;
use std::io::Cursor;

pub fn convert_excel_to_df(bytes: &[u8]) -> Result<DataFrame> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

    // Assume the first sheet is the target
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| anyhow!("No worksheets found"))?
        .map_err(|e| anyhow!("Failed to read worksheet: {}", e))?;

    let mut rows_iter = range.rows();

    // Get headers
    let headers: Vec<String> = rows_iter
        .next()
        .ok_or_else(|| anyhow!("Empty worksheet"))?
        .iter()
        .map(|c| c.to_string())
        .collect();

    let mut columns_data: Vec<Vec<AnyValue>> = vec![vec![]; headers.len()];

    // Iterate over rows and collect data
    for row in rows_iter {
        for (i, cell) in row.iter().enumerate() {
            if i < columns_data.len() {
                let value = match cell {
                    Data::Int(v) => AnyValue::Int64(*v),
                    Data::Float(v) => AnyValue::Float64(*v),
                    Data::String(v) => AnyValue::StringOwned(v.into()),
                    Data::Bool(v) => AnyValue::Boolean(*v),
                    Data::DateTime(v) => AnyValue::Float64(v.as_f64()), // Simplification
                    Data::Error(_) => AnyValue::Null,
                    Data::Empty => AnyValue::Null,
                    Data::DateTimeIso(v) => AnyValue::StringOwned(v.into()), 
                    Data::DurationIso(v) => AnyValue::StringOwned(v.into()), 
                };
                columns_data[i].push(value);
            }
        }
    }

    // Create Series
    let series_vec: Vec<Series> = headers
        .into_iter()
        .zip(columns_data.into_iter())
        .map(|(name, data)| {
            Series::from_any_values(&name, &data, false).unwrap()
        })
        .collect();

    DataFrame::new(series_vec).map_err(|e| anyhow!("Failed to create DataFrame: {}", e))
}
