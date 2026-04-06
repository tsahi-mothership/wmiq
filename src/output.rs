use crate::query::variant_to_string;
use anyhow::Result;
use std::collections::HashMap;
use tabled::{builder::Builder, settings::Style};
use wmi::Variant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    List,
}

impl OutputFormat {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "list" => Some(Self::List),
            _ => None,
        }
    }
}

/// Format query results according to the chosen output format.
pub fn format_results(
    columns: &[String],
    rows: &[HashMap<String, Variant>],
    format: OutputFormat,
) -> Result<String> {
    if rows.is_empty() {
        return Ok("No results.".to_string());
    }

    // If columns is empty, derive from first row
    let cols: Vec<String> = if columns.is_empty() {
        let mut keys: Vec<String> = rows[0].keys().cloned().collect();
        keys.sort();
        keys
    } else {
        columns.to_vec()
    };

    match format {
        OutputFormat::Table => format_table(&cols, rows),
        OutputFormat::Json => format_json(&cols, rows),
        OutputFormat::Csv => format_csv(&cols, rows),
        OutputFormat::List => format_list(&cols, rows),
    }
}

fn format_table(columns: &[String], rows: &[HashMap<String, Variant>]) -> Result<String> {
    let mut builder = Builder::default();
    builder.push_record(columns.iter().map(|c| c.as_str()));

    for row in rows {
        let values: Vec<String> = columns.iter().map(|col| {
            row.get(col).map(variant_to_string).unwrap_or_default()
        }).collect();
        builder.push_record(values);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    Ok(table.to_string())
}

fn format_json(columns: &[String], rows: &[HashMap<String, Variant>]) -> Result<String> {
    let json_rows: Vec<serde_json::Value> = rows.iter().map(|row| {
        let mut map = serde_json::Map::new();
        for col in columns {
            let val = row.get(col).map(variant_to_json).unwrap_or(serde_json::Value::Null);
            map.insert(col.clone(), val);
        }
        serde_json::Value::Object(map)
    }).collect();

    Ok(serde_json::to_string_pretty(&json_rows)?)
}

fn variant_to_json(v: &Variant) -> serde_json::Value {
    match v {
        Variant::Null | Variant::Empty => serde_json::Value::Null,
        Variant::String(s) => serde_json::Value::String(s.clone()),
        Variant::Bool(b) => serde_json::Value::Bool(*b),
        Variant::I1(n) => serde_json::json!(*n),
        Variant::I2(n) => serde_json::json!(*n),
        Variant::I4(n) => serde_json::json!(*n),
        Variant::I8(n) => serde_json::json!(*n),
        Variant::UI1(n) => serde_json::json!(*n),
        Variant::UI2(n) => serde_json::json!(*n),
        Variant::UI4(n) => serde_json::json!(*n),
        Variant::UI8(n) => serde_json::json!(*n),
        Variant::R4(n) => serde_json::json!(*n),
        Variant::R8(n) => serde_json::json!(*n),
        Variant::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(variant_to_json).collect())
        }
        Variant::Object(obj) => {
            if let Ok(props) = obj.list_properties() {
                let mut map = serde_json::Map::new();
                for p in &props {
                    if let Ok(v) = obj.get_property(p) {
                        map.insert(p.clone(), variant_to_json(&v));
                    }
                }
                serde_json::Value::Object(map)
            } else {
                serde_json::Value::Null
            }
        }
        Variant::Unknown(_) => serde_json::Value::Null,
    }
}

fn format_csv(columns: &[String], rows: &[HashMap<String, Variant>]) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(columns)?;

    for row in rows {
        let values: Vec<String> = columns.iter().map(|col| {
            row.get(col).map(variant_to_string).unwrap_or_default()
        }).collect();
        wtr.write_record(&values)?;
    }

    wtr.flush()?;
    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

fn format_list(columns: &[String], rows: &[HashMap<String, Variant>]) -> Result<String> {
    let mut output = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        for col in columns {
            let val = row.get(col).map(variant_to_string).unwrap_or_default();
            output.push_str(&format!("{}={}\n", col, val));
        }
    }
    Ok(output)
}
