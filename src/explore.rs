use anyhow::{Context, Result};
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};

use crate::query::variant_to_string;

/// List all WMI class names in the given namespace.
pub fn list_classes(namespace: &str) -> Result<Vec<String>> {
    let com = COMLibrary::new().context("Failed to initialize COM")?;
    let conn = if namespace == "root\\cimv2" || namespace == r"root\cimv2" {
        WMIConnection::new(com)?
    } else {
        WMIConnection::with_namespace_path(namespace, com)?
    };

    let results: Vec<HashMap<String, Variant>> = conn
        .raw_query("SELECT * FROM meta_class")
        .context("Failed to query meta_class")?;

    let mut classes: Vec<String> = results
        .iter()
        .filter_map(|row| {
            row.get("__CLASS").map(variant_to_string)
        })
        .filter(|name| !name.is_empty())
        .collect();

    classes.sort();
    Ok(classes)
}

/// List all WMI namespaces under "root".
pub fn list_namespaces() -> Result<Vec<String>> {
    let mut all = vec!["root".to_string()];
    collect_namespaces("root", &mut all)?;
    all.sort();
    Ok(all)
}

fn collect_namespaces(parent: &str, out: &mut Vec<String>) -> Result<()> {
    let com = COMLibrary::new().context("Failed to initialize COM")?;
    let conn = WMIConnection::with_namespace_path(parent, com)
        .context(format!("Failed to connect to namespace: {}", parent))?;

    let results: Vec<HashMap<String, Variant>> = conn
        .raw_query("SELECT Name FROM __NAMESPACE")
        .unwrap_or_default();

    for row in &results {
        if let Some(name) = row.get("Name") {
            let name_str = variant_to_string(name);
            if !name_str.is_empty() {
                let full = format!("{}\\{}", parent, name_str);
                out.push(full.clone());
                // Recursively collect sub-namespaces; ignore errors for inaccessible ones
                let _ = collect_namespaces(&full, out);
            }
        }
    }

    Ok(())
}

/// Explore a specific WMI class — list its properties by querying one instance.
pub fn explore_class(namespace: &str, class_name: &str) -> Result<Vec<String>> {
    let com = COMLibrary::new().context("Failed to initialize COM")?;
    let conn = if namespace == "root\\cimv2" || namespace == r"root\cimv2" {
        WMIConnection::new(com)?
    } else {
        WMIConnection::with_namespace_path(namespace, com)?
    };

    // Query a single instance to discover properties
    let wql = format!("SELECT * FROM {}", class_name);
    let results: Vec<HashMap<String, Variant>> = conn
        .raw_query(&wql)
        .context(format!("Failed to query class: {}", class_name))?;

    let mut props: Vec<String> = if let Some(first) = results.first() {
        first.keys().cloned().collect()
    } else {
        Vec::new()
    };

    props.sort();
    Ok(props)
}
