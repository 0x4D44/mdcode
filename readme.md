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

## Features
- **Repository Initialization:** Quickly create new local Git repositories with an initial commit.
- **Automated Updates:** Stage and commit changes with a single command.
- **Repository Info:** Display the latest commits and file change summaries.
 - **Diffing:** Compare different versions of your code using external diff tools, including GitHub HEAD and local comparisons.
- **GitHub Integration:** Automatically create a GitHub repository, add it as a remote, and push your local branch.
- **Release Tagging:** Create annotated git tags using the version from Cargo.toml or a provided version.
- **Exclusion of Unwanted Files:** Automatically ignores directories like `target`, `bin`, `obj`, `venv`, `.venv`, and `env` so that build artifacts and virtual environments do not clutter your repository.
- **Multi-Language Support:** Recognizes various file types (Rust, Python, C/C++, Java, etc.) to accurately identify source files.

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

## Commands

- `new <dir>` — Initialize a new repo with initial commit.
- `update <dir>` — Stage changes and commit.
- `info <dir>` — Show recent commits and file changes.
- `diff <dir> [m] [n]` — Diff commits or vs. working tree.
- `gh_create <dir> [--description <text>]` — Create a GitHub repo and push. Prefers GitHub CLI; falls back to API with `GITHUB_TOKEN`/`GH_TOKEN`.
- `gh_push <dir> [--remote <name>]` — Push the current branch.
- `gh_fetch <dir> [--remote <name>]` — Fetch and list remote-only commits.
- `gh_sync <dir> [--remote <name>]` — Pull to sync with remote.
- `tag <dir> [--version <semver>] [--message <msg>] [--remote <name>] [--force] [--allow-dirty] [--no-push]` — Create an annotated tag on HEAD and push it by default.

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
