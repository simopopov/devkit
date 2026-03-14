# portwatch

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

Port monitoring and conflict resolution for macOS and Linux. Binary: `pw`.

Replaces the `lsof -i :PORT | grep LISTEN | awk | kill` workflow with a single command.

Part of [devkit](https://github.com/simopopov/devkit).

---

## Install

```bash
# Homebrew (recommended)
brew tap simopopov/devkit
brew install portwatch

# cargo
cargo install --git https://github.com/simopopov/devkit portwatch
```

---

## Usage

```bash
# Interactive TUI dashboard — all listening ports, live
pw

# Query a specific port
pw :3000

# Kill whatever holds port 3000 (SIGTERM, then SIGKILL if needed)
pw kill :3000

# Free every process in a port range
pw free 3000-3010
```

---

## TUI dashboard

Launches with `pw`. Shows a live table of all listening ports:

```
PORT    PID     PROCESS       USER       CPU%    MEM%    PROTO
3000    84201   node          alice      1.2     0.8     TCP
5432    391     postgres      _postgres  0.0     2.1     TCP
8080    12044   python3       alice      0.4     0.3     TCP
```

| Key              | Action                |
|------------------|-----------------------|
| `q` / `Esc`      | Quit                  |
| `Up` / `Down` or `j` / `k` | Navigate   |
| `x`              | Kill selected process |
| `r`              | Refresh               |

---

## Features

- Interactive TUI with sortable columns (PORT, PID, PROCESS, USER, CPU%, MEM%, PROTO)
- Point query: `pw :PORT` prints a single-line answer without launching the TUI
- Graceful kill sequence: SIGTERM first, SIGKILL only if the process does not exit
- Range free: kills all processes in a port range in one command
- Works on macOS and Linux; no root required for ports you own

---

## License

MIT
