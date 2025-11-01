.PHONY: coverage coverage-tarpaulin coverage-llvm coverage-detailed coverage-html coverage-lcov coverage-check coverage-llvm-gate-98 coverage-llvm-gate-97 preflight

coverage:
	@echo "Coverage run (COVERAGE_SKIP_TARPAULIN=$${COVERAGE_SKIP_TARPAULIN:-0})"
	@if [ "$${COVERAGE_SKIP_TARPAULIN:-0}" != "1" ]; then $(MAKE) coverage-tarpaulin; else echo "Skipping Tarpaulin"; fi
	$(MAKE) coverage-llvm
	$(MAKE) coverage-check

coverage-tarpaulin:
	mkdir -p target/coverage
	cargo tarpaulin --features offline_gh --out Json --output-dir target/coverage

coverage-llvm:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --summary-only --json --output-path target/coverage/llvm-summary.json

coverage-lcov:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --lcov --output-path target/coverage/lcov.info

coverage-detailed:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --json --output-path target/coverage/llvm-detailed.json

coverage-html:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --html --output-dir target/coverage/html

.PHONY: coverage-open
coverage-open: coverage-html
	./scripts/open_coverage_html.sh || true

coverage-check:
	./scripts/coverage_gate.py

# Strict gate for CI: require LLVM line coverage >= 98%
coverage-llvm-gate-98:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --fail-under-lines 98 --summary-only --json --output-path target/coverage/llvm-summary.json

coverage-llvm-gate-97:
	mkdir -p target/coverage
	cargo llvm-cov --lib --tests --features offline_gh --ignore-filename-regex 'src/main.rs' --fail-under-lines 97 --summary-only --json --output-path target/coverage/llvm-summary.json

# One-shot local quality gate: fmt, clippy (code + tests), unit/integration tests, and coverage gate
preflight:
	cargo fmt --all --check
	cargo clippy -- -D warnings
	cargo clippy --tests -- -D warnings
	cargo test
	COVERAGE_SKIP_TARPAULIN=1 COVERAGE_OPTIONAL_TARPAULIN=1 $(MAKE) coverage-llvm-gate-97
