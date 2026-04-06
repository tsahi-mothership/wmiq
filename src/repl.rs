use anyhow::Result;
use colored::Colorize;
use rustyline::DefaultEditor;

use crate::aliases::{build_alias_registry, list_aliases};
use crate::explore::explore_class;
use crate::output::{format_results, OutputFormat};
use crate::query::{exec_alias, exec_wql};

pub fn run_repl(namespace: &str, format: OutputFormat) -> Result<()> {
    let registry = build_alias_registry();
    let mut rl = DefaultEditor::new()?;

    println!("{}", "wmiq interactive mode. Type :exit to quit, :help for commands.".green());

    loop {
        let line = match rl.readline("wmiq> ") {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(line);

        if line == ":exit" || line == ":quit" {
            break;
        }

        if line == ":help" {
            println!("Commands:");
            println!("  <alias> [columns]    Query using alias (e.g. cpu name,cores)");
            println!("  :q <WQL>             Execute raw WQL query");
            println!("  :aliases             List all aliases");
            println!("  :props <ClassName>   Show properties of a WMI class");
            println!("  :exit                Exit the REPL");
            continue;
        }

        if line == ":aliases" {
            for (name, class, cols) in list_aliases() {
                println!("  {:10} {} ({})", name.cyan(), class, cols);
            }
            continue;
        }

        if let Some(wql) = line.strip_prefix(":q ") {
            let wql = wql.trim();
            match exec_wql(namespace, wql) {
                Ok(results) => {
                    let cols: Vec<String> = if let Some(first) = results.first() {
                        let mut keys: Vec<String> = first.keys().cloned().collect();
                        keys.sort();
                        keys
                    } else {
                        Vec::new()
                    };
                    match format_results(&cols, &results, format) {
                        Ok(output) => println!("{}", output),
                        Err(e) => eprintln!("{}: {}", "Format error".red(), e),
                    }
                }
                Err(e) => eprintln!("{}: {}", "Query error".red(), e),
            }
            continue;
        }

        if let Some(class_name) = line.strip_prefix(":props ") {
            let class_name = class_name.trim();
            match explore_class(namespace, class_name) {
                Ok(props) => {
                    println!("{} properties of {}:", props.len(), class_name.cyan());
                    for p in &props {
                        println!("  {}", p);
                    }
                }
                Err(e) => eprintln!("{}: {}", "Error".red(), e),
            }
            continue;
        }

        // Parse as alias query: <alias> [col1,col2,...]
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let alias_name = parts[0];

        if let Some(entry) = registry.get(alias_name) {
            let columns: Vec<&str> = if parts.len() > 1 {
                parts[1].split(',').map(|s| s.trim()).collect()
            } else {
                entry.columns.clone()
            };

            match exec_alias(namespace, entry.class, &columns, entry.filter, None) {
                Ok((col_names, results)) => {
                    match format_results(&col_names, &results, format) {
                        Ok(output) => println!("{}", output),
                        Err(e) => eprintln!("{}: {}", "Format error".red(), e),
                    }
                }
                Err(e) => eprintln!("{}: {}", "Query error".red(), e),
            }
        } else {
            eprintln!("{}: unknown command or alias '{}'. Type :help for usage.", "Error".red(), alias_name);
        }
    }

    Ok(())
}
