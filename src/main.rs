mod aliases;
mod explore;
mod output;
mod query;
mod repl;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use aliases::{build_alias_registry, list_aliases};
use explore::{explore_class, list_classes, list_namespaces};
use output::{format_results, OutputFormat};
use query::{exec_alias, exec_wql};

#[derive(Parser)]
#[command(
    name = "wmiq",
    about = "Modern WMI CLI replacement for WMIC",
    version,
    after_help = "Examples:\n  wmiq cpu\n  wmiq os -o json\n  wmiq -q \"SELECT Name FROM Win32_Process\"\n  wmiq explore Win32_Processor\n  wmiq -i"
)]
struct Cli {
    /// Alias name or subcommand (e.g. cpu, os, disk)
    #[arg(value_name = "ALIAS")]
    alias: Option<String>,

    /// Column list (comma-separated) to override default columns
    #[arg(value_name = "COLUMNS")]
    columns: Option<String>,

    /// Raw WQL query
    #[arg(short = 'q', long = "query")]
    wql: Option<String>,

    /// Output format: table, json, csv, list
    #[arg(short = 'o', long = "output", default_value = "table")]
    format: String,

    /// WHERE clause filter
    #[arg(short = 'w', long = "where")]
    where_clause: Option<String>,

    /// WMI namespace (default: root\cimv2)
    #[arg(short = 'n', long = "namespace", default_value = r"root\cimv2")]
    namespace: String,

    /// Enter interactive REPL mode
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Explore a WMI class — list its properties
    Explore {
        /// WMI class name (e.g. Win32_Processor)
        class_name: String,
    },
    /// List all WMI classes in a namespace
    Classes,
    /// List all WMI namespaces
    Namespaces,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let format = OutputFormat::from_str_opt(&cli.format)
        .unwrap_or_else(|| {
            eprintln!("{}: unknown format '{}', defaulting to table", "Warning".yellow(), cli.format);
            OutputFormat::Table
        });

    let namespace = &cli.namespace;

    // Interactive REPL mode
    if cli.interactive {
        return repl::run_repl(namespace, format);
    }

    // Subcommands
    if let Some(cmd) = &cli.command {
        return match cmd {
            Commands::Explore { class_name } => {
                let props = explore_class(namespace, class_name)?;
                if props.is_empty() {
                    println!("No properties found (class may have no instances or does not exist).");
                } else {
                    println!("{} properties of {}:", props.len().to_string().green(), class_name.cyan());
                    for p in &props {
                        println!("  {}", p);
                    }
                }
                Ok(())
            }
            Commands::Classes => {
                let classes = list_classes(namespace)?;
                println!("{} classes in {}:", classes.len().to_string().green(), namespace.cyan());
                for c in &classes {
                    println!("  {}", c);
                }
                Ok(())
            }
            Commands::Namespaces => {
                let ns = list_namespaces()?;
                println!("{} namespaces:", ns.len().to_string().green());
                for n in &ns {
                    println!("  {}", n);
                }
                Ok(())
            }
        };
    }

    // Raw WQL query
    if let Some(wql) = &cli.wql {
        let results = exec_wql(namespace, wql)?;
        let cols: Vec<String> = if let Some(first) = results.first() {
            let mut keys: Vec<String> = first.keys().cloned().collect();
            keys.sort();
            keys
        } else {
            Vec::new()
        };
        let output = format_results(&cols, &results, format)?;
        println!("{}", output);
        return if results.is_empty() {
            std::process::exit(1);
        } else {
            Ok(())
        };
    }

    // Alias query
    if let Some(alias_name) = &cli.alias {
        // Check if it matches a known alias
        let registry = build_alias_registry();

        if let Some(entry) = registry.get(alias_name.as_str()) {
            let columns: Vec<&str> = if let Some(cols) = &cli.columns {
                cols.split(',').map(|s| s.trim()).collect()
            } else {
                entry.columns.clone()
            };

            let (col_names, results) = exec_alias(
                namespace,
                entry.class,
                &columns,
                entry.filter,
                cli.where_clause.as_deref(),
            )?;

            let output = format_results(&col_names, &results, format)?;
            println!("{}", output);

            if results.is_empty() {
                std::process::exit(1);
            }
        } else {
            // Maybe user typed a WMI class name directly
            eprintln!("{}: unknown alias '{}'. Use --help to see usage, or wmiq -i for interactive mode.", "Error".red(), alias_name);
            eprintln!("\nAvailable aliases:");
            for (name, class, cols) in list_aliases() {
                eprintln!("  {:10} {} ({})", name.cyan(), class, cols);
            }
            std::process::exit(2);
        }
    } else {
        // No alias, no query, no subcommand — show help-like summary
        println!("{}", "wmiq — Modern WMI CLI replacement for WMIC".green().bold());
        println!();
        println!("Usage: wmiq <alias> [columns]");
        println!("       wmiq -q <WQL>");
        println!("       wmiq -i              (interactive REPL)");
        println!("       wmiq explore <class>");
        println!("       wmiq classes");
        println!("       wmiq namespaces");
        println!();
        println!("Aliases:");
        for (name, class, cols) in list_aliases() {
            println!("  {:10} {} ({})", name.cyan(), class, cols);
        }
        println!();
        println!("Run wmiq --help for full options.");
    }

    Ok(())
}
