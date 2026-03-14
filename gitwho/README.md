# gitwho

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

Find the right person to ask about any file or directory in a git repository.

Part of [devkit](https://github.com/simopopov/devkit).

---

## Install

```bash
# Homebrew (recommended)
brew tap simopopov/devkit
brew install gitwho

# cargo
cargo install --git https://github.com/simopopov/devkit gitwho
```

---

## Usage

```bash
# Who knows the auth module?
gitwho src/auth/

# Who maintains a specific file?
gitwho README.md

# Limit analysis to the last 6 months
gitwho --since 6m src/api/

# JSON output for scripting
gitwho --json src/
```

---

## Output

Authors are ranked by score — a composite of how recently, how much, and how often
they have touched the target path.

```
RANK    AUTHOR               SCORE    COMMITS    LINES (+/-)       LAST COMMIT
1       alice@example.com    94.3     47         +3201 / -891      3 days ago
2       bob@example.com      61.7     29         +1840 / -620      3 weeks ago
3       carol@example.com    22.1     8          +410  / -88       4 months ago
```

---

## Scoring

Each author's score combines three factors:

- **Recency** — exponential decay with a half-life of approximately 6 months,
  so recent work weighs more than old commits
- **Volume** — total lines added and removed across all matching commits
- **Frequency** — number of commits that touch the target path

This surfaces people who both know the code deeply and have kept it up to date,
rather than just whoever wrote the original version years ago.

---

## Features

- Works on any file or directory path within a git repository
- `--since` accepts human-readable durations: `30d`, `6m`, `1y`
- `--json` outputs structured data for use in scripts or CI tooling
- Reads local git history only — no network calls, no external services

---

## License

MIT
