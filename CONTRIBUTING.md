# Contributing to mdcode

Thanks for helping improve mdcode! This guide covers local setup, coding style, tests, and coverage gates.

## Prerequisites
- Rust toolchain (stable). Install from rust-lang.org.
- Git in PATH.
- Optional (for coverage HTML viewing): a local browser.

## Local Workflow
- Build: `cargo build`
- Run: `cargo run -- <subcommand>` (see `readme.md` for examples)
- Tests: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --all`

Recommended one-shot: `make preflight`
- Runs: fmt check → clippy (code + tests) → tests → LLVM coverage gate ≥97%.

## Coverage Policy
- CI enforces LLVM line coverage ≥98% (library-only). See `.github/workflows/ci.yml`.
- Local coverage targets: `make coverage-llvm`, `make coverage-detailed`, `make coverage-html`, `make coverage-lcov`.
- Trend baseline file: `coverage_baseline.toml`. When improving coverage significantly, update the baseline and add a short doc to `wrk_docs/` explaining the change.

Notes
- Tests must avoid network I/O. Use `--features offline_gh` and PATH shims where necessary.
- Some display-heavy paths are simplified under `#[cfg(coverage)]` to keep measured lines meaningful without altering behavior.

## Coding Style
- Formatting: rustfmt (stable). Always format before committing.
- Linting: clippy with warnings as errors (`-D warnings`) for code and tests.
- Naming: `snake_case` (functions/files), `CamelCase` (types/traits), `SCREAMING_SNAKE_CASE` (consts).
- Logging: `log::{info, warn, error, debug}`; keep messages concise and actionable.

## Commit & PR Guidelines
- Use imperative mood, concise scope, e.g., `feat(tag): push annotated tag`.
- Link issues: `Closes #123` when applicable.
- Include rationale, CLI examples, and before/after snippets or logs.
- Pre‑flight locally before opening/merging PRs: `make preflight`.

## Tests
- Unit tests inline with code (`#[cfg(test)]`), integration tests in `tests/`.
- Prefer `tempfile` for filesystem isolation.
- Avoid network and non-deterministic time dependencies.

## Questions
Open a discussion or issue on the repository with logs and steps to reproduce.

