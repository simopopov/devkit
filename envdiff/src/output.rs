use crate::diff::{DiffRow, KeyStatus};
use colored::*;
use std::collections::HashSet;

const MASK: &str = "\u{25CF}\u{25CF}\u{25CF}\u{25CF}";
const MISSING_MARKER: &str = "<missing>";

/// Render the diff table to stdout with colors.
pub fn print_table(rows: &[DiffRow], file_names: &[String], reveal_keys: &HashSet<String>) {
    if rows.is_empty() {
        println!("{}", "All environments are identical.".green());
        return;
    }

    // Compute column widths
    let key_width = rows
        .iter()
        .map(|r| r.key.len())
        .max()
        .unwrap_or(3)
        .max(3);

    let val_widths: Vec<usize> = (0..file_names.len())
        .map(|col| {
            let header_len = file_names[col].len();
            let max_val = rows
                .iter()
                .map(|r| display_value(r, col, reveal_keys).len())
                .max()
                .unwrap_or(0);
            header_len.max(max_val).max(4)
        })
        .collect();

    // Print header
    print!("  {:<width$}", "KEY", width = key_width);
    for (i, name) in file_names.iter().enumerate() {
        print!("  {:<width$}", name, width = val_widths[i]);
    }
    println!();

    // Separator
    let total_width = 2 + key_width + val_widths.iter().map(|w| w + 2).sum::<usize>();
    println!("{}", "\u{2500}".repeat(total_width).dimmed());

    // Rows
    for row in rows {
        let status_char = match row.status {
            KeyStatus::Same => " ".normal(),
            KeyStatus::Different => "~".yellow(),
            KeyStatus::Missing => "!".red(),
        };

        let key_colored = match row.status {
            KeyStatus::Same => row.key.green(),
            KeyStatus::Different => row.key.yellow(),
            KeyStatus::Missing => row.key.red(),
        };

        print!("{} {:<width$}", status_char, key_colored, width = key_width);

        for (col, _) in file_names.iter().enumerate() {
            let val_str = display_value(row, col, reveal_keys);
            let colored_val = match (&row.status, &row.values[col]) {
                (_, None) => MISSING_MARKER.red(),
                (KeyStatus::Same, _) => val_str.green(),
                (KeyStatus::Different, _) => val_str.yellow(),
                (KeyStatus::Missing, _) => val_str.normal(),
            };
            print!("  {:<width$}", colored_val, width = val_widths[col]);
        }
        println!();
    }

    // Summary
    println!();
    let missing = rows.iter().filter(|r| r.status == KeyStatus::Missing).count();
    let different = rows.iter().filter(|r| r.status == KeyStatus::Different).count();
    let same = rows.iter().filter(|r| r.status == KeyStatus::Same).count();

    if missing > 0 {
        println!("{}", format!("  {} missing", missing).red());
    }
    if different > 0 {
        println!("{}", format!("  {} different", different).yellow());
    }
    println!("  {} same", same);
}

fn display_value(row: &DiffRow, col: usize, reveal_keys: &HashSet<String>) -> String {
    match &row.values[col] {
        None => MISSING_MARKER.to_string(),
        Some(val) => {
            if reveal_keys.contains(&row.key) {
                // Truncate long values for display
                if val.len() > 50 {
                    format!("{}...", &val[..47])
                } else {
                    val.clone()
                }
            } else {
                MASK.to_string()
            }
        }
    }
}

/// Print machine-readable CI output (no colors, tab-separated).
pub fn print_ci(rows: &[DiffRow], file_names: &[String], reveal_keys: &HashSet<String>) {
    // Header
    print!("STATUS\tKEY");
    for name in file_names {
        print!("\t{}", name);
    }
    println!();

    for row in rows {
        let status = match row.status {
            KeyStatus::Same => "SAME",
            KeyStatus::Different => "DIFF",
            KeyStatus::Missing => "MISSING",
        };
        print!("{}\t{}", status, row.key);
        for (col, _) in file_names.iter().enumerate() {
            let val = display_value(row, col, reveal_keys);
            print!("\t{}", val);
        }
        println!();
    }
}
