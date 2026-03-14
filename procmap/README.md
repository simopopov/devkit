# procmap

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

Visual process and network topology for your local machine.

Part of [devkit](https://github.com/simopopov/devkit).

---

## Install

```bash
# Homebrew (recommended)
brew tap simopopov/devkit
brew install procmap

# cargo
cargo install --git https://github.com/simopopov/devkit procmap
```

---

## Usage

```bash
# Interactive TUI with expandable process tree and network connections
procmap

# Static ASCII process tree with port annotations, printed to stdout
procmap --tree

# Filter to processes using ports in a specific range
procmap --ports 3000-9000
```

---

## TUI display

Each row shows: PID, PPID, process name, user, CPU%, MEM%, listening ports,
and active connection count. Parent-child relationships are shown as an
expandable tree.

| Color  | Meaning                   |
|--------|---------------------------|
| Cyan   | Process name              |
| Yellow | Port numbers              |
| Red    | High CPU or memory usage  |

| Key                         | Action                   |
|-----------------------------|--------------------------|
| `q` / `Esc`                 | Quit                     |
| `Up` / `Down` or `j` / `k` | Navigate                 |
| `Enter` / `Space`           | Expand or collapse subtree |

---

## Features

- Interactive TUI built on ratatui with a collapsible process tree
- `--tree` mode prints a static snapshot — useful for logging or piping
- `--ports` filters the view to processes that hold connections in a given range,
  cutting noise when debugging a specific service
- Shows both listening ports and established outbound connections per process
- Color highlights make high-resource processes immediately visible

---

## License

MIT
