use anyhow::{Context, Result, bail};
use chrono::{DateTime, FixedOffset, Utc};
use std::collections::HashMap;
use std::process::Command;

/// A single parsed commit with its per-file stats.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: DateTime<Utc>,
    pub file_stats: Vec<FileStat>,
}

#[derive(Debug, Clone)]
pub struct FileStat {
    pub path: String,
    pub added: u64,
    pub removed: u64,
}

/// Aggregated per-author statistics across all matching commits.
#[derive(Debug, Clone)]
pub struct AuthorStats {
    pub author: String,
    pub email: String,
    pub commits: u64,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub last_commit: DateTime<Utc>,
    /// Raw commit-level records kept for scoring.
    pub commit_records: Vec<CommitRecord>,
}

#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub date: DateTime<Utc>,
    pub lines_changed: u64,
}

/// Check whether the current directory (or an ancestor) is inside a git repo.
pub fn check_git_repo() -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to run git. Is git installed and on PATH?")?;

    if !output.status.success() {
        bail!(
            "Not inside a git repository. Run gitwho from within a git project."
        );
    }
    Ok(())
}

/// Run `git log` for the given path with an optional --since filter and parse the output.
pub fn run_git_log(path: &str, since: Option<&str>) -> Result<Vec<CommitInfo>> {
    let format = "commit %H%nauthor %aN%nemail %aE%ndate %aI";

    let mut cmd = Command::new("git");
    cmd.args([
        "log",
        "--numstat",
        &format!("--format={}", format),
    ]);

    if let Some(since_val) = since {
        cmd.arg(format!("--since={}", since_val));
    }

    cmd.arg("--").arg(path);

    let output = cmd
        .output()
        .context("Failed to execute git log")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git log failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_log(&stdout)
}

/// Parse the raw text output of git log into structured CommitInfo records.
fn parse_git_log(raw: &str) -> Result<Vec<CommitInfo>> {
    let mut commits: Vec<CommitInfo> = Vec::new();
    let mut current: Option<CommitInfoBuilder> = None;

    for line in raw.lines() {
        let line = line.trim_end();

        if let Some(hash) = line.strip_prefix("commit ") {
            // Flush previous commit
            if let Some(builder) = current.take() {
                if let Some(c) = builder.build() {
                    commits.push(c);
                }
            }
            current = Some(CommitInfoBuilder::new(hash.trim().to_string()));
            continue;
        }

        if let Some(ref mut builder) = current {
            if let Some(author) = line.strip_prefix("author ") {
                builder.author = Some(author.to_string());
            } else if let Some(email) = line.strip_prefix("email ") {
                builder.email = Some(email.to_string());
            } else if let Some(date_str) = line.strip_prefix("date ") {
                let dt = DateTime::parse_from_rfc3339(date_str.trim())
                    .map(|d: DateTime<FixedOffset>| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                builder.date = Some(dt);
            } else if !line.is_empty() {
                // numstat line: ADDED\tREMOVED\tFILE
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 3 {
                    // Binary files show "-" for added/removed; skip those.
                    let added = parts[0].parse::<u64>().unwrap_or(0);
                    let removed = parts[1].parse::<u64>().unwrap_or(0);
                    builder.file_stats.push(FileStat {
                        path: parts[2].to_string(),
                        added,
                        removed,
                    });
                }
            }
        }
    }

    // Flush last commit
    if let Some(builder) = current {
        if let Some(c) = builder.build() {
            commits.push(c);
        }
    }

    Ok(commits)
}

struct CommitInfoBuilder {
    hash: String,
    author: Option<String>,
    email: Option<String>,
    date: Option<DateTime<Utc>>,
    file_stats: Vec<FileStat>,
}

impl CommitInfoBuilder {
    fn new(hash: String) -> Self {
        Self {
            hash,
            author: None,
            email: None,
            date: None,
            file_stats: Vec::new(),
        }
    }

    fn build(self) -> Option<CommitInfo> {
        Some(CommitInfo {
            hash: self.hash,
            author: self.author?,
            email: self.email.unwrap_or_default(),
            date: self.date?,
            file_stats: self.file_stats,
        })
    }
}

/// Aggregate parsed commits into per-author stats.
pub fn aggregate(commits: &[CommitInfo]) -> Vec<AuthorStats> {
    let mut map: HashMap<String, AuthorStats> = HashMap::new();

    for commit in commits {
        let total_lines: u64 = commit
            .file_stats
            .iter()
            .map(|f| f.added + f.removed)
            .sum();
        let total_added: u64 = commit.file_stats.iter().map(|f| f.added).sum();
        let total_removed: u64 = commit.file_stats.iter().map(|f| f.removed).sum();

        let entry = map
            .entry(commit.author.clone())
            .or_insert_with(|| AuthorStats {
                author: commit.author.clone(),
                email: commit.email.clone(),
                commits: 0,
                lines_added: 0,
                lines_removed: 0,
                last_commit: commit.date,
                commit_records: Vec::new(),
            });

        entry.commits += 1;
        entry.lines_added += total_added;
        entry.lines_removed += total_removed;

        if commit.date > entry.last_commit {
            entry.last_commit = commit.date;
        }

        entry.commit_records.push(CommitRecord {
            date: commit.date,
            lines_changed: total_lines,
        });
    }

    map.into_values().collect()
}
