use anyhow::{Context, Result};
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};

/// Convert a WMI Variant value into a displayable string.
pub fn variant_to_string(v: &Variant) -> String {
    match v {
        Variant::Null => String::new(),
        Variant::Empty => String::new(),
        Variant::String(s) => s.clone(),
        Variant::I1(n) => n.to_string(),
        Variant::I2(n) => n.to_string(),
        Variant::I4(n) => n.to_string(),
        Variant::I8(n) => n.to_string(),
        Variant::UI1(n) => n.to_string(),
        Variant::UI2(n) => n.to_string(),
        Variant::UI4(n) => n.to_string(),
        Variant::UI8(n) => n.to_string(),
        Variant::R4(n) => n.to_string(),
        Variant::R8(n) => n.to_string(),
        Variant::Bool(b) => b.to_string(),
        Variant::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| variant_to_string(v)).collect();
            items.join(", ")
        }
        Variant::Object(obj) => {
            // Nested WMI object — show properties as JSON-like
            if let Ok(props) = obj.list_properties() {
                let pairs: Vec<String> = props.iter()
                    .filter_map(|p| obj.get_property(p).ok().map(|v| format!("{}={}", p, variant_to_string(&v))))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            } else {
                "{...}".to_string()
            }
        }
        Variant::Unknown(_) => "<unknown>".to_string(),
    }
}

/// Build a WQL query string from class, columns, and optional WHERE clause.
pub fn build_wql(class: &str, columns: &[&str], filter: Option<&str>, extra_where: Option<&str>) -> String {
    let cols = if columns.is_empty() {
        "*".to_string()
    } else {
        columns.join(", ")
    };

    let mut query = format!("SELECT {} FROM {}", cols, class);

    let mut conditions = Vec::new();
    if let Some(f) = filter {
        conditions.push(f.to_string());
    }
    if let Some(w) = extra_where {
        conditions.push(w.to_string());
    }
    if !conditions.is_empty() {
        query.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }
    query
}

/// Execute a raw WQL query and return results as Vec<HashMap<String, Variant>>.
pub fn exec_wql(namespace: &str, wql: &str) -> Result<Vec<HashMap<String, Variant>>> {
    let com = COMLibrary::new().context("Failed to initialize COM library")?;
    let conn = if namespace == "root\\cimv2" || namespace == r"root\cimv2" {
        WMIConnection::new(com).context("Failed to connect to WMI (root\\cimv2)")?
    } else {
        WMIConnection::with_namespace_path(namespace, com)
            .context(format!("Failed to connect to WMI namespace: {}", namespace))?
    };

    let results: Vec<HashMap<String, Variant>> = conn
        .raw_query(wql)
        .context(format!("WQL query failed: {}", wql))?;

    Ok(results)
}

/// Execute an alias-based query and return results with ordered column names.
pub fn exec_alias(
    namespace: &str,
    class: &str,
    columns: &[&str],
    filter: Option<&str>,
    extra_where: Option<&str>,
) -> Result<(Vec<String>, Vec<HashMap<String, Variant>>)> {
    let wql = build_wql(class, columns, filter, extra_where);
    let results = exec_wql(namespace, &wql)?;

    // Determine column order
    let col_names: Vec<String> = if columns.is_empty() {
        // Infer from first result
        if let Some(first) = results.first() {
            let mut keys: Vec<String> = first.keys().cloned().collect();
            keys.sort();
            keys
        } else {
            Vec::new()
        }
    } else {
        columns.iter().map(|c| c.to_string()).collect()
    };

    Ok((col_names, results))
}
