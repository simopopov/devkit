# devkit

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform: macOS & Linux](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-lightgrey.svg)](#installation)

**Four focused CLI tools that replace the shell one-liners you keep forgetting.**

`devkit` is a Rust workspace monorepo providing single-binary utilities for port management, environment file comparison, git authorship analysis, and process topology visualization. Each tool is independent — install only what you need.

---

## Tools

| Tool | Binary | What it does |
|------|--------|--------------|
| [portwatch](#portwatch) | `pw` | Monitor ports and kill conflicts without the `lsof` gymnastics |
| [envdiff](#envdiff) | `envdiff` | Compare `.env` files and catch missing or mismatched keys |
| [gitwho](#gitwho) | `gitwho` | Find who actually knows a piece of code |
| [procmap](#procmap) | `procmap` | Visualize the process tree and network topology on your machine |

---

## portwatch

Port monitoring and conflict resolution. Replaces the `lsof -i :PORT | grep LISTEN | awk | kill` workflow with a single command.

### Usage

```bash
# Launch the interactive TUI dashboard
pw

# Query what process holds a specific port
pw :3000

# Kill whatever is holding port 3000 (SIGTERM, then SIGKILL if needed)
pw kill :3000

# Free an entire port range
pw free 3000-3010
```

### TUI dashboard

The TUI shows a live table of all listening ports with the following columns:

```
PORT    PID     PROCESS       USER     CPU%    MEM%    PROTO
3000    84201   node          alice    1.2     0.8     TCP
5432    391     postgres      _postgres 0.0    2.1     TCP
8080    12044   python3       alice    0.4     0.3     TCP
```

**Keyboard shortcuts**

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Up` / `Down` or `j` / `k` | Navigate |
| `x` | Kill selected process |
| `r` | Refresh |

---

## envdiff

Environment variable comparison and sync across `.env` files. Compares two or more files, highlights missing or mismatched keys, and masks sensitive values by default so you can share output safely.

### Usage

```bash
# Compare two environment files
envdiff .env .env.production

# Compare three files side by side
envdiff .env .env.staging .env.prod

# Reveal specific key values instead of masking them
envdiff --reveal DB_HOST,API_URL .env .env.prod

# CI mode: exits with code 1 if any differences are found
envdiff --ci .env .env.prod
```

### Output

Missing keys appear in **red**, keys with different values in **yellow**, and matching keys in **green**. All values are masked with `●●●●` by default — use `--reveal` to expose specific keys.

```
KEY              .env              .env.production
DATABASE_URL     ●●●●              ●●●●              [different]
API_KEY          ●●●●              (missing)
DEBUG            true              false             [different]
PORT             3000              3000              [match]
```

### Parser support

The parser handles real-world `.env` file quirks:

- Quoted values (single and double quotes)
- Multiline values
- Inline comments
- `export VAR=value` prefix
- Variable interpolation (`$VAR` and `${VAR}`)

---

## gitwho

Find the right person to ask about any piece of code. Analyzes git history to surface the developers who have worked most meaningfully on a given file or directory — weighted by recency, volume, and frequency.

### Usage

```bash
# Find experts for a module
gitwho src/auth/

# Find who maintains a specific file
gitwho README.md

# Limit analysis to the last 6 months
gitwho --since 6m src/api/

# Output as JSON for scripting
gitwho --json src/
```

### Output

```
RANK    AUTHOR              SCORE    COMMITS    LINES (+/-)      LAST COMMIT
1       alice@example.com   94.3     47         +3201 / -891     3 days ago
2       bob@example.com     61.7     29         +1840 / -620     3 weeks ago
3       carol@example.com   22.1     8          +410  / -88      4 months ago
```

### Scoring

Each author's score combines three factors:

- **Recency** — exponential decay with a half-life of approximately 6 months, so recent work counts more
- **Volume** — total lines added and removed
- **Frequency** — number of commits touching the path

This surfaces the people who both know the code well and have kept it up to date.

---

## procmap

Visual process and network topology for your local machine. Shows the full process tree annotated with listening ports and network connections.

### Usage

```bash
# Launch the interactive TUI
procmap

# Print a static ASCII process tree with port annotations
procmap --tree

# Focus on a specific port range (useful for filtering to dev servers)
procmap --ports 3000-9000
```

### TUI display

Each process entry shows: `PID`, `PPID`, parent-child relationship, process name, user, `CPU%`, `MEM%`, listening ports, and active connections.

**Color coding**

| Color | Meaning |
|-------|---------|
| Cyan | Process name |
| Yellow | Port numbers |
| Red | High CPU or memory usage |

**Keyboard shortcuts**

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Up` / `Down` or `j` / `k` | Navigate |
| `Enter` / `Space` | Expand or collapse subtree |

---

## Installation

### Homebrew (macOS and Linux) — recommended

```bash
brew tap simopopov/devkit

brew install portwatch   # installs the `pw` command
brew install envdiff
brew install gitwho
brew install procmap
```

### cargo install

```bash
cargo install --git https://github.com/simopopov/devkit portwatch
cargo install --git https://github.com/simopopov/devkit envdiff
cargo install --git https://github.com/simopopov/devkit gitwho
cargo install --git https://github.com/simopopov/devkit procmap
```

### From source

Requires [Rust](https://rustup.rs/) (stable toolchain).

```bash
git clone https://github.com/simopopov/devkit.git
cd devkit
cargo build --release
```

Compiled binaries will be in `target/release/`: `pw`, `envdiff`, `gitwho`, `procmap`.

---

## Platform support

| Platform | Status |
|----------|--------|
| macOS | Primary |
| Linux | Supported |
| Windows | Not supported |

---

## Contributing

Bug reports and pull requests are welcome at [github.com/simopopov/devkit](https://github.com/simopopov/devkit).

Before submitting a pull request, please:

1. Run `cargo test` and ensure all tests pass
2. Run `cargo clippy` and address any warnings
3. Format your code with `cargo fmt`

Each tool lives in its own workspace crate. If you are adding a feature to a single tool, you only need to work within that crate's directory.

---

## License

MIT. See [LICENSE](LICENSE) for the full text.
