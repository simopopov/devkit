mod diff;
mod output;
mod parser;

use anyhow::{bail, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "envdiff",
    about = "Compare .env files across environments",
    version
)]
struct Cli {
    /// .env files to compare (at least 2)
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Reveal values for specific keys (comma-separated)
    #[arg(long, value_delimiter = ',')]
    reveal: Vec<String>,

    /// CI mode: machine-readable output, exit 1 if differences found
    #[arg(long)]
    ci: bool,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.files.len() < 2 {
        bail!("At least 2 .env files are required for comparison");
    }

    // Parse all files
    let mut envs = Vec::with_capacity(cli.files.len());
    for path in &cli.files {
        let map = parser::parse_env_file(path)?;
        envs.push(map);
    }

    // Build file name labels
    let file_names: Vec<String> = cli
        .files
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect();

    // Compute diff
    let rows = diff::diff_envs(&envs);
    let has_diff = diff::has_differences(&rows);

    // Build reveal set
    let reveal_keys: HashSet<String> = cli.reveal.into_iter().collect();

    // Output
    if cli.ci {
        // Disable colors in CI mode
        colored::control::set_override(false);
        output::print_ci(&rows, &file_names, &reveal_keys);
        if has_diff {
            process::exit(1);
        }
    } else {
        output::print_table(&rows, &file_names, &reveal_keys);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(2);
    }
}
