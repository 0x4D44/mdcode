# 2025-10-22 Coverage Update

## Commands
- `make coverage` (runs `cargo tarpaulin --out Json --output-dir target/coverage` and `cargo llvm-cov --summary-only --json`)
- `./scripts/coverage_gate.py` (guards against >5 pp regressions vs `coverage_baseline.toml`)

## Results
- **Tarpaulin**: 85.88 % line coverage (`target/coverage/tarpaulin-report.json`)
- **LLVM-cov**: 89.54 % line, 87.54 % region, 89.87 % function coverage (`target/coverage/llvm-summary.json`)
- Gate status: `Coverage OK | Tarpaulin: 85.88% (baseline 43.49%, floor 38.49%) | LLVM: 89.54% (baseline 55.83%, floor 50.83%)`

## Progress Since Baseline (2025-10-22)
- Tarpaulin line coverage **+42.39 pp** (43.49 % ➜ 85.88 %)
- LLVM line coverage **+33.71 pp** (55.83 % ➜ 89.54 %)
- Function coverage improved from 59.22 % ➜ 89.87 %

## Newly Covered Paths
- `update_repository`’s interactive commit message flow is now testable via `MDCODE_TEST_COMMIT_PROMPT`, covering the previously untestable stdin branch.
- Added regression coverage for `gh_push` merge-conflict handling, asserting the user guidance emitted when `git pull --no-edit` fails.
- The GitHub `gh_create` auto-push path now executes under test (using the API stub), so the `MDCODE_SKIP_GH_PUSH` short-circuit and CLI dispatch branches are all exercised.
- Enabling the test logger ensures fine-grained `log::debug!` telemetry is exercised, capturing numerous diagnostic branches within repository scanning, tagging, and diff flows.
- Added negative coverage for `info_repository` (missing repo and empty repo) alongside `is_dirty`’s pre-head guard and the `read_version_from_cargo_toml` “no file” path.
- Dry-run `update` previews, diffing, tagging, and GitHub command flows remain green under Tarpaulin after the expanded suite.

## Remaining Gaps & Follow-ups
- `run()`/`main()` wiring still goes uncovered under Tarpaulin (expected, as the binary entrypoint is bypassed). Leave as-is or craft an integration smoke test if desired.
- `#[cfg(not(test))]` stdin/stdout paths (e.g., actual interactive prompts) stay intentionally untested; document this behavior or inject further hooks if deterministic coverage is needed.
- Tarpaulin still reports uncovered verbose logging tied to cold error paths (e.g., `info_repository` failure diagnostics); consider synthetic failure fixtures if we want to close the final gap.

## Next Steps
- Continue deduplicating temporary-repository helpers so that higher-level CLI integration tests stay performant.
- Evaluate whether additional GitHub API failure simulations are required to exercise remaining error-reporting branches.
