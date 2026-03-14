# envdiff

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

Compare `.env` files across environments and catch missing or mismatched keys.

Part of [devkit](https://github.com/simopopov/devkit).

---

## Install

```bash
# Homebrew (recommended)
brew tap simopopov/devkit
brew install envdiff

# cargo
cargo install --git https://github.com/simopopov/devkit envdiff
```

---

## Usage

```bash
# Compare two environment files
envdiff .env .env.production

# Compare three or more files side by side
envdiff .env .env.staging .env.prod

# Reveal specific key values instead of masking them
envdiff --reveal DB_HOST,API_URL .env .env.prod

# CI mode: exits with code 1 if any differences are found
envdiff --ci .env .env.prod
```

---

## Output

Missing keys appear in red, differing values in yellow, matching keys in green.
All values are masked with `●●●●` by default — use `--reveal KEY,...` to expose specific keys.

```
KEY              .env        .env.production
DATABASE_URL     ●●●●        ●●●●              [different]
API_KEY          ●●●●        (missing)
DEBUG            true        false             [different]
PORT             3000        3000              [match]
```

---

## Features

- Compares two or more files in a single run
- Safe by default: values are masked so output is shareable in logs and CI
- `--reveal` exposes only the keys you name, leaving the rest masked
- `--ci` flag makes the process exit 1 on any diff, suitable for pre-deploy checks
- Robust parser handles real-world `.env` quirks:
  - Single and double quoted values
  - Multiline values
  - Inline comments
  - `export VAR=value` prefix
  - Variable interpolation (`$VAR` and `${VAR}`)

---

## License

MIT
