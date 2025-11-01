## Summary

Describe the change, motivation, and any context.

## Checklist
- [ ] `cargo fmt --all` (or `make preflight`)
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo clippy --tests -- -D warnings`
- [ ] `cargo test` (all tests green locally)
- [ ] `make coverage-llvm` ≥ 98% lines (library-only) or `make preflight` ≥ 97% lines (local gate)
- [ ] Updated docs/journal if behavior or coverage meaningfully changed

## Screenshots / Logs (optional)

## Breaking Changes?
- [ ] No breaking changes
- [ ] Yes (explain):

