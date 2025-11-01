# Coverage Improvement Plan

Goal: raise mdcode’s automated test coverage to **≥80 % line coverage** (LLVM) and **≥70 % line coverage** (Tarpaulin) while keeping test runtimes <5 min. The plan is split into sequential stages; each stage defines scope, success criteria, and tooling.

---

## Stage 1 — Establish Baseline & Infrastructure (Week 1)
- **Inventory**: Document current coverage gaps from `tarpaulin-report.json` and the latest `cargo llvm-cov --summary-only` output; snapshot these metrics in `reports/coverage/YYYY-MM-DD.md`.
- **Automation**: Add a `coverage` Makefile target (or Cargo alias) that runs both `cargo tarpaulin --out Json` and `cargo llvm-cov --summary-only`. Ensure artefacts land in `target/coverage/`.
- **Continuous Integration**: Update CI workflow to upload coverage artefacts and fail if either line metric regresses >5 % from baseline.
- **Success criteria**: Repeatable local command (`make coverage`) and CI job producing identical metrics to manual runs.

## Stage 2 — Cover Core CLI Command Paths (Week 2)
- **Run-loop tests**: Add unit tests that invoke `run()` with synthetic `Cli` inputs to exercise:
  - `Commands::New`, `Commands::Update`, `Commands::Info`, `Commands::Diff`
  - GitHub commands using stubbed environment variables and temporary repositories
- **Repo scaffolding helpers**: Use `tempfile` + in-memory Git fixtures to simulate repos, verifying diff and tag logic.
- **Assertions**: Validate both success paths and expected error handling (e.g., invalid visibility flags, missing remotes).
- **Success criteria**: Tarpaulin line coverage ≥55 %, LLVM line coverage ≥65 %.

## Stage 3 — Expand GitHub & Diff Coverage (Weeks 3–4)
- **API fallback**: Introduce mocks for `octocrab` interactions (feature-gated or via dependency injection) to cover `gh_create_api`.
- **CLI fallback**: Use `assert_cmd` to simulate `gh` CLI presence/absence around `gh_create_via_cli`.
- **Diff tooling**: Build integration tests for `diff_command`, `launch_diff_tool`, and `launch_custom_diff_tool` using dummy files and environment overrides.
- **Branch coverage**: Extend tests to cover both dry-run and non-dry-run states plus large-file thresholds.
- **Success criteria**: Tarpaulin ≥65 %, LLVM ≥75 %; no uncovered branches in diff/gh modules.

## Stage 4 — Regression & Edge Coverage (Week 5)
- **Edge cases**: Add tests for repo detection failures, malformed semver tags, dirty workspace scenarios, and network failure handling.
- **Fuzz-style inputs**: Use property tests (`proptest`) for path sanitisation and `.gitignore` generation to catch edge cases.
- **Performance guardrails**: Monitor runtime of new suites; parallelise where possible (`cargo test -- --test-threads=1` when necessary).
- **Success criteria**: Achieve target coverage (Tarpaulin ≥70 %, LLVM ≥80 %) with stable test duration.

## Stage 5 — Maintain & Monitor (Ongoing)
- **Documentation**: Update `readme.md` with coverage commands and thresholds.
- **CI enforcement**: Gate merges on coverage thresholds via status checks.
- **Review cadence**: Re-run coverage after major feature additions; ensure new code includes targeted tests before review.
- **Toolchain updates**: Keep `cargo-tarpaulin` and `cargo-llvm-cov` pinned in `Cargo.toml` `[dev-dependencies]`, updating quarterly.

---

### Deliverables Checklist
- [ ] `reports/coverage/YYYY-MM-DD.md` baseline report
- [ ] Combined coverage command (`make coverage` or Cargo alias)
- [ ] New/updated tests covering CLI, diff, GitHub flows, edge cases
- [ ] CI workflow enforcing thresholds
- [ ] Documentation updates for developers

