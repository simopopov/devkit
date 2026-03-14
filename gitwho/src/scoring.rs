use chrono::Utc;

use crate::analyzer::AuthorStats;

/// Scored result for a single author, ready for display.
#[derive(Debug, Clone)]
pub struct ScoredAuthor {
    pub rank: usize,
    pub author: String,
    pub email: String,
    pub score: f64,
    pub commits: u64,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub last_commit: chrono::DateTime<Utc>,
}

/// Compute scores for each author and return them sorted descending by score.
///
/// Scoring formula per commit contribution:
///   score += lines_changed * recency_weight * frequency_bonus
///
/// where:
///   recency_weight = exp(-days_ago / 180.0)   (half-life ~6 months)
///   frequency_bonus = 1 + ln(commit_count)     (across all the author's commits)
pub fn score_authors(stats: &[AuthorStats]) -> Vec<ScoredAuthor> {
    let now = Utc::now();

    let mut scored: Vec<ScoredAuthor> = stats
        .iter()
        .map(|a| {
            let frequency_bonus = 1.0 + (a.commits as f64).ln();

            let raw_score: f64 = a
                .commit_records
                .iter()
                .map(|r| {
                    let days_ago = (now - r.date).num_seconds().max(0) as f64 / 86400.0;
                    let recency = (-days_ago / 180.0).exp();
                    r.lines_changed as f64 * recency * frequency_bonus
                })
                .sum();

            ScoredAuthor {
                rank: 0, // filled in after sorting
                author: a.author.clone(),
                email: a.email.clone(),
                score: raw_score,
                commits: a.commits,
                lines_added: a.lines_added,
                lines_removed: a.lines_removed,
                last_commit: a.last_commit,
            }
        })
        .collect();

    // Sort descending by score
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Assign ranks
    for (i, s) in scored.iter_mut().enumerate() {
        s.rank = i + 1;
    }

    scored
}
