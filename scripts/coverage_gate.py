#!/usr/bin/env python3
"""
Basic coverage guard that prevents regressions larger than the configured tolerance.

Reads:
  - target/coverage/tarpaulin-report.json
  - target/coverage/llvm-summary.json
  - coverage_baseline.toml
"""

from __future__ import annotations

import json
import math
import pathlib
import os
import sys

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback
    import tomli as tomllib  # type: ignore


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_TARPAULIN = REPO_ROOT / "target" / "coverage" / "tarpaulin-report.json"
DEFAULT_LLVM = REPO_ROOT / "target" / "coverage" / "llvm-summary.json"
BASELINE_FILE = REPO_ROOT / "coverage_baseline.toml"


class CoverageError(RuntimeError):
    """Raised when coverage files are missing or thresholds are violated."""


def read_tarpaulin(report_path: pathlib.Path) -> float:
    """Return Tarpaulin line coverage percentage."""
    # Allow callers (e.g., CI) to mark Tarpaulin as optional.
    if not report_path.exists():
        if os.getenv("COVERAGE_OPTIONAL_TARPAULIN") == "1":
            raise CoverageError("Tarpaulin optional: report missing and optional flag set")
        raise CoverageError(f"Tarpaulin report missing: {report_path}")
    with report_path.open("r", encoding="utf-8") as fh:
        data = json.load(fh)
    coverage = float(data["coverage"])
    return coverage


def read_llvm(report_path: pathlib.Path) -> float:
    """Return LLVM line coverage percentage."""
    if not report_path.exists():
        raise CoverageError(f"LLVM summary missing: {report_path}")
    with report_path.open("r", encoding="utf-8") as fh:
        data = json.load(fh)
    totals = data["data"][0]["totals"]
    coverage = float(totals["lines"]["percent"])
    return coverage


def read_baseline(config_path: pathlib.Path) -> tuple[float, float, float]:
    """Read baseline + tolerance from TOML file (returns tarpaulin, llvm, max_drop)."""
    if not config_path.exists():
        raise CoverageError(f"Coverage baseline config missing: {config_path}")
    with config_path.open("rb") as fh:
        config = tomllib.load(fh)
    try:
        baseline_tarpaulin = float(config["baseline"]["tarpaulin_line"])
        baseline_llvm = float(config["baseline"]["llvm_line"])
        max_drop = float(config["threshold"]["max_drop"])
    except (KeyError, TypeError, ValueError) as exc:  # pragma: no cover - defensive
        raise CoverageError(f"Invalid baseline configuration ({exc})") from exc
    return baseline_tarpaulin, baseline_llvm, max_drop


def enforce_threshold(
    tarpaulin_cov: float,
    llvm_cov: float,
    baseline_tarpaulin: float,
    baseline_llvm: float,
    max_drop: float,
) -> None:
    """Ensure coverage has not regressed beyond allowed tolerance."""
    tarpaulin_floor = max(0.0, baseline_tarpaulin - max_drop)
    llvm_floor = max(0.0, baseline_llvm - max_drop)

    failures: list[str] = []

    if tarpaulin_cov < tarpaulin_floor - 1e-6:
        delta = tarpaulin_cov - baseline_tarpaulin
        failures.append(
            f"Tarpaulin coverage {tarpaulin_cov:.2f}% (Δ {delta:+.2f}%) below floor {tarpaulin_floor:.2f}%"
        )

    if llvm_cov < llvm_floor - 1e-6:
        delta = llvm_cov - baseline_llvm
        failures.append(
            f"LLVM coverage {llvm_cov:.2f}% (Δ {delta:+.2f}%) below floor {llvm_floor:.2f}%"
        )

    if failures:
        raise CoverageError("; ".join(failures))

    print(
        "Coverage OK | "
        f"Tarpaulin: {tarpaulin_cov:.2f}% (baseline {baseline_tarpaulin:.2f}%, floor {tarpaulin_floor:.2f}%) | "
        f"LLVM: {llvm_cov:.2f}% (baseline {baseline_llvm:.2f}%, floor {llvm_floor:.2f}%)"
    )


def main() -> int:
    try:
        baseline_tarpaulin, baseline_llvm, max_drop = read_baseline(BASELINE_FILE)
        try:
            if os.getenv("COVERAGE_OPTIONAL_TARPAULIN") == "1":
                # Skip Tarpaulin: use baseline as a placeholder; threshold won’t fail on it.
                tarpaulin_cov = baseline_tarpaulin
            else:
                tarpaulin_cov = read_tarpaulin(DEFAULT_TARPAULIN)
        except CoverageError as exc:
            # If optional, treat as informational and continue with LLVM-only enforcement.
            if os.getenv("COVERAGE_OPTIONAL_TARPAULIN") == "1":
                print(f"Tarpaulin skipped: {exc}")
                tarpaulin_cov = baseline_tarpaulin
            else:
                raise
        llvm_cov = read_llvm(DEFAULT_LLVM)
        enforce_threshold(
            tarpaulin_cov, llvm_cov, baseline_tarpaulin, baseline_llvm, max_drop
        )
    except CoverageError as exc:
        print(f"Coverage regression: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
