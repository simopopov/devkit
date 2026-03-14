mod analyzer;
mod output;
mod scoring;

use anyhow::Result;
use clap::Parser;

/// Find the right person to ask about any code.
/// Analyzes git history to surface code experts per file, directory, or module.
#[derive(Parser, Debug)]
#[command(name = "gitwho", version, about)]
struct Cli {
    /// File or directory path to analyze
    path: String,

    /// Filter by time period (e.g., "6m", "1y", "3w", "90d")
    #[arg(long)]
    since: Option<String>,

    /// Output results as JSON
    #[arg(long, default_value_t = false)]
    json: bool,
}

/// Convert shorthand durations like "6m", "1y" into git-compatible --since values.
fn normalize_since(input: &str) -> String {
    let input = input.trim();

    // Already a git-compatible value (contains space or looks like a date)
    if input.contains(' ') || input.contains('-') {
        return input.to_string();
    }

    // Try to parse trailing letter as unit
    let (num_part, unit) = input.split_at(input.len().saturating_sub(1));
    if let Ok(n) = num_part.parse::<u64>() {
        match unit {
            "d" => return format!("{} days ago", n),
            "w" => return format!("{} weeks ago", n),
            "m" => return format!("{} months ago", n),
            "y" => return format!("{} years ago", n),
            _ => {}
        }
    }

    // Fall back to passing it through as-is (let git decide)
    input.to_string()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check we're in a git repo first.
    analyzer::check_git_repo()?;

    let since_normalized = cli.since.as_deref().map(normalize_since);
    let commits = analyzer::run_git_log(&cli.path, since_normalized.as_deref())?;

    let stats = analyzer::aggregate(&commits);
    let scored = scoring::score_authors(&stats);

    if cli.json {
        output::print_json(&scored);
    } else {
        output::print_table(&scored, &cli.path);
    }

    Ok(())
}
