use colored::Colorize;
use serde::Serialize;

use crate::scoring::ScoredAuthor;

#[derive(Serialize)]
struct JsonAuthor {
    rank: usize,
    author: String,
    email: String,
    score: f64,
    commits: u64,
    lines_added: u64,
    lines_removed: u64,
    last_commit: String,
}

/// Print results as a JSON array.
pub fn print_json(authors: &[ScoredAuthor]) {
    let records: Vec<JsonAuthor> = authors
        .iter()
        .map(|a| JsonAuthor {
            rank: a.rank,
            author: a.author.clone(),
            email: a.email.clone(),
            score: (a.score * 100.0).round() / 100.0,
            commits: a.commits,
            lines_added: a.lines_added,
            lines_removed: a.lines_removed,
            last_commit: a.last_commit.format("%Y-%m-%d").to_string(),
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".to_string())
    );
}

/// Print results as a colored table to stdout.
pub fn print_table(authors: &[ScoredAuthor], path: &str) {
    if authors.is_empty() {
        println!("No git history found for '{}'.", path);
        return;
    }

    println!();
    println!(
        "  {} {}",
        "Code experts for".bold(),
        path.bold().underline()
    );
    println!();

    // Header
    println!(
        "  {:<5} {:<25} {:>8} {:>8} {:>12} {:>12}",
        "RANK", "AUTHOR", "SCORE", "COMMITS", "LINES(+/-)", "LAST COMMIT"
    );
    println!("  {}", "-".repeat(74));

    for a in authors {
        let score_str = format!("{:.1}", a.score);
        let lines_str = format!("+{}/-{}", a.lines_added, a.lines_removed);
        let date_str = a.last_commit.format("%Y-%m-%d").to_string();

        let row = format!(
            "  {:<5} {:<25} {:>8} {:>8} {:>12} {:>12}",
            a.rank, a.author, score_str, a.commits, lines_str, date_str
        );

        if a.rank == 1 {
            println!("{}", row.bright_green().bold());
        } else {
            println!("{}", row);
        }
    }

    println!();
}
