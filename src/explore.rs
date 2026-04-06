use anyhow::Result;

use crate::query::{exec_wql, variant_to_string};

/// List all WMI class names in the given namespace.
pub fn list_classes(namespace: &str) -> Result<Vec<String>> {
    let results = exec_wql(namespace, "SELECT * FROM meta_class")?;

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
    let results = exec_wql(parent, "SELECT Name FROM __NAMESPACE")
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
    let wql = format!("SELECT * FROM {}", class_name);
    let results = exec_wql(namespace, &wql)?;

    let mut props: Vec<String> = if let Some(first) = results.first() {
        first.keys().cloned().collect()
    } else {
        Vec::new()
    };

    props.sort();
    Ok(props)
}
