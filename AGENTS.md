# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: CLI entrypoint and core helpers (Clap, git2, octocrab, logging).
- `Cargo.toml` / `Cargo.lock`: crate metadata and dependencies.
- `readme.md`: usage, install, and command reference.
- `target/`: build artifacts (ignored). Add `tests/` for integration tests when needed.

## Build, Test, and Development Commands
- Build: `cargo build` (use `--release` for optimized binary).
- Run: `cargo run -- <subcommand>`
  - Examples: `cargo run -- new .`, `cargo run -- update .`,
    `cargo run -- diff . 0 1`, `cargo run -- gh_create . --description "Repo"`,
    `cargo run -- gh_push .`, `cargo run -- gh_fetch .`, `cargo run -- gh_sync .`,
    `cargo run -- tag . --version 1.2.3 --message "Cut 1.2.3"`.
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --all`

## Coding Style & Naming Conventions
- Formatting: rustfmt (4 spaces, stable defaults). Run `cargo fmt` before commits.
- Linting: clippy must pass with no warnings; treat warnings as errors in PRs.
- Naming: `snake_case` (functions/files), `CamelCase` (types/traits), `SCREAMING_SNAKE_CASE` (consts).
- Logging: use `log::{info, warn, error, debug}`; keep messages concise and actionable.

## Testing Guidelines
- Framework: built-in Rust harness via `cargo test`.
- Location: unit tests inline (`#[cfg(test)]`); integration tests under `tests/`.
- Conventions: name tests `test_*`; prefer `tempfile` for FS isolation and avoid network.
- Scope: cover subcommands (`new`, `update`, `info`, `diff`, `gh_*`, `tag`) and edge cases (missing remotes, ambiguous HEAD).

## Commit & Pull Request Guidelines
- Commits: imperative mood, concise scope (e.g., `feat(tag): push annotated tag`).
- Link issues with `Closes #123` when applicable.
- PRs: include rationale, CLI examples (before/after), and logs or screenshots.
- Pre‑flight: `cargo fmt` • `cargo clippy -- -D warnings` • `cargo test`.

## Security & Configuration Tips
- GitHub: set `GITHUB_TOKEN` for `gh_*` commands.
- Diagnostics: set `RUST_LOG=info` (or `debug`) when troubleshooting.
- Secrets: never commit tokens; confirm `.gitignore` covers `target/`, venvs, and local tooling.
