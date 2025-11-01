# mdcode

**mdcode** is a simple code management tool built on top of Git, written in Rust. It provides a clean command-line interface for creating new repositories, updating commits, viewing repository history, performing diffs, and integrating seamlessly with GitHub. With built-in support for excluding build artifacts and virtual environments, mdcode helps keep your repository clean and focused on your source code.

## Table of Contents
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Local Workflow](#local-workflow)
  - [GitHub Integration](#github-integration)
- [Commands](#commands)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)
 - [Coverage](#coverage)
 - [Changelog](#changelog)

## Features
- **Repository Initialization:** Quickly create new local Git repositories with an initial commit.
- **Automated Updates:** Stage and commit changes with a single command.
- **Repository Info:** Display the latest commits and file change summaries.
 - **Diffing:** Compare different versions of your code using external diff tools, including GitHub HEAD and local comparisons.
- **GitHub Integration:** Automatically create a GitHub repository, add it as a remote, and push your local branch.
- **Release Tagging:** Create annotated git tags using the version from Cargo.toml or a provided version.
- **Exclusion of Unwanted Files:** Automatically ignores directories like `target`, `bin`, `obj`, `venv`, `.venv`, and `env` so that build artifacts and virtual environments do not clutter your repository.
- **Multi-Language Support:** Recognizes various file types (Rust, Python, C/C++, Java, etc.) to accurately identify source files.
 - **Asset Support + Size Cap:** Recognizes audio (`wav`, `mp3`, `flac`, `aac`, `m4a`, `ogg`, `opus`, `aiff`, `aif`, `wma`, `mid`, `midi`) and fonts (`ttf`, `otf`, `woff`, `woff2`). Large files are skipped by default if larger than 50 MB; adjust with `--max-file-mb`.

## Installation

### Prerequisites
- **Rust:** Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Git:** Ensure Git is installed. Download from [git-scm.com](https://git-scm.com/downloads).
- **GitHub CLI (optional, recommended):** Install [GitHub CLI](https://cli.github.com/). `gh_create` prefers `gh` (uses OS keychain/Windows Credential Manager). Run `gh auth login` once.
- **GitHub Personal Access Token (fallback):** If `gh` is not available, set a token with `repo` scope.  
  - On Windows:
    ```batch
    set GITHUB_TOKEN=your_token_here
    ```
  - On Unix/Linux:
    ```bash
    export GITHUB_TOKEN=your_token_here
    ```

### Build from Source
Clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/mdcode.git
cd mdcode
cargo build --release
```

## Commands

- `new <dir>` — Initialize a new repo with initial commit.
- `update <dir>` — Stage changes and commit.
- `info <dir>` — Show recent commits and file changes.
- `diff <dir> [m] [n]` — Diff commits or vs. working tree.
- `gh_create <dir> [--description <text>] [--public|--private|--internal]` — Create a GitHub repo and push. Prefers GitHub CLI; falls back to API with `GITHUB_TOKEN`/`GH_TOKEN`. If no visibility is provided, defaults to `--private`.
- `gh_push <dir> [--remote <name>]` — Push the current branch (fails fast if HEAD is detached).
- `gh_fetch <dir> [--remote <name>]` — Fetch and list remote-only commits.
- `gh_sync <dir> [--remote <name>]` — Pull to sync with remote.
- `tag <dir> [--version <semver>] [--message <msg>] [--remote <name>] [--force] [--allow-dirty] [--no-push]` — Create an annotated tag on HEAD (requires clean tree unless `--allow-dirty`) and push it by default.

### Tag examples

```bash
# Tag using version from Cargo.toml and push
mdcode tag .

# Tag with explicit version and message (pushes by default)
mdcode tag . --version 1.2.3 --message "Cut 1.2.3"

# Overwrite existing tag and push to origin
mdcode tag . --force

# Create tag but do not push
mdcode tag . --no-push
```

## Configuration

- `--max-file-mb <N>`: Set a per-run maximum size (in MB) for files that `new`/`update` will auto-stage. Default: `50`.
  - Files exceeding the cap are skipped with a notice: `Ignoring '<path>' as larger than <N> MB - use '--max-file-mb'`.
- `MDCODE_DIFF_TOOL` / `DIFF_TOOL`: Set to a command (e.g. `code --diff`) to override the diff viewer used by `mdcode diff`. The before/after paths are appended to the command.
- `mdcode update --dry-run`: Shows a preview list of files that would be committed without touching the repository.

## Coverage

This repo uses LLVM source-based coverage via `cargo llvm-cov` and includes an optional Tarpaulin run.

- Quick summary JSON (library-only, tests included):
  - `make coverage-llvm` → writes `target/coverage/llvm-summary.json`.
- Detailed JSON (for pinpointing lines/regions):
  - `make coverage-detailed` → writes `target/coverage/llvm-detailed.json`.
- HTML report for browsing:
  - `make coverage-html` → generates `target/coverage/html/`.
  - `make coverage-open` → builds the HTML if needed and opens it (best effort).
  - `make preflight` → fmt + clippy (code+tests) + tests + coverage gate (LLVM ≥97%).
 - LCOV export:
   - `make coverage-lcov` → writes `target/coverage/lcov.info` (useful for external tools or local diffing).
- Full "trend" run (Tarpaulin + LLVM + gate):
  - `make coverage`
  - You can skip Tarpaulin (useful in CI/local for speed):
    - `COVERAGE_SKIP_TARPAULIN=1 COVERAGE_OPTIONAL_TARPAULIN=1 make coverage`
- Strict gate (CI):
  - `make coverage-llvm-gate-98` → fails if LLVM line coverage < 98%.

Notes
- Coverage measures library code only; the thin binary wrapper in `src/main.rs` is ignored.
- Tests run offline and avoid network/API calls by using the `offline_gh` feature and PATH shims.

### Coverage Enforcement
- CI runs `make coverage` (LLVM summary + optional Tarpaulin), then enforces LLVM lines ≥98% via `make coverage-llvm-gate-98`.
- Local baseline guard (`scripts/coverage_gate.py`) compares current results against `coverage_baseline.toml` and rejects large regressions.
- Developer preflight: `make preflight` (fmt + clippy + tests + LLVM ≥97%) for fast iteration.
- Artifacts uploaded in CI: `target/coverage/llvm-summary.json`, `target/coverage/html/`, and `target/coverage/lcov.info`.
- To browse locally: `make coverage-html && make coverage-open`.

### Badges (optional)
- After pushing to GitHub, add a CI badge in README (replace OWNER/REPO):
  - `![CI](https://github.com/OWNER/REPO/actions/workflows/ci.yml/badge.svg)`
- If using a coverage service (e.g., Codecov), add its badge once configured. This repo already exports `lcov.info` for ingestion.

## Changelog
- See release notes in `wrk_docs/2025.10.31 - Release Notes - v2.0.2 — Coverage ≥98%.md`.
