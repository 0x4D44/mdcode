# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: CLI implementation and core helpers (Clap, git2, octocrab, logging).
- `Cargo.toml` / `Cargo.lock`: crate metadata and dependencies.
- `readme.md`: usage, features, and install notes.
- `target/`: build artifacts (ignored by Git). Create `tests/` for integration tests when needed.

## Build, Test, and Development Commands
- Build: `cargo build` (use `--release` for optimized binary).
- Run locally: `cargo run -- <subcommand>`
  - Examples: `cargo run -- new .`, `cargo run -- update .`,
    `cargo run -- diff . 0 1`, `cargo run -- gh_create . --description "Repo"`.
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --all`

## Coding Style & Naming Conventions
- Formatting: Rustfmt (4 spaces, stable defaults). Run `cargo fmt` before commits.
- Linting: Clippy must pass with no warnings in CI/PRs.
- Naming: `snake_case` for files/functions, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Logs: prefer `log::{info, warn, error, debug}` with concise messages.

## Testing Guidelines
- Framework: built-in Rust test harness via `cargo test`.
- Locations: unit tests inline (`#[cfg(test)] mod tests`) and integration tests under `tests/`.
- Conventions: functions named `test_*`; use `tempfile` for filesystem isolation.
- Scope: cover CLI subcommands (new, update, info, diff, gh_*). Validate edge cases (missing remotes, HEAD without target).

## Commit & Pull Request Guidelines
- Messages: imperative mood, concise summary; include scope when helpful (e.g., `fix: handle remote HEAD w/o target`).
- Link issues: `Closes #123` in the body when applicable.
- PRs: include description, rationale, CLI examples (before/after), and any screenshots/log snippets.
- Pre-flight: run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before opening/merging.

## Security & Configuration Tips
- GitHub API: set `GITHUB_TOKEN` in your environment for `gh_*` commands.
- Logging: set `RUST_LOG=info` (or `debug`) to troubleshoot.
- Secrets: do not commit tokens; verify `.gitignore` excludes `target/`, virtual envs, and local tooling folders.
