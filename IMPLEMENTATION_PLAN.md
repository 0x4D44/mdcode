# Implementation Plan

## Stage 1 – Stabilise Critical Workflows
- Fix dotfile staging: adjust `new` to regenerate the staging set after writing `.gitignore`, explicitly add the file, and update `update` to stage tracked-but-unclassified paths (e.g. fallback to `git add -u`). Extend unit tests to cover `.gitignore` creation and modification.
- Preserve GitHub visibility in API fallback: plumb the `RepoVisibility` selection through to `gh_create_api`, update the payload to set the correct visibility/private flag, and add an integration-style test (mock or feature-gated) to confirm the request body.
- Normalise diff temp prefixes: replace path separators and drive-colon characters before calling `create_temp_dir`, and add a regression test covering Windows-style paths.

## Stage 2 – Cross-Platform UX & Accuracy
- Expand diff tooling support: honour a configurable diff command (env var/CLI flag) and implement platform-aware fallbacks (`code --diff`, `git diff --stat`, etc.). Document the precedence and add tests for the selection logic.
- Keep CLI version in sync: replace the static Clap version string with `env!("CARGO_PKG_VERSION")`, and add a unit test that asserts the reported version matches `pkg_version`.

## Stage 3 – Behaviour Alignment & Docs
- Resolve `tag --allow-dirty`: either enforce a cleanliness check with an override or remove the flag; update CLI help, README, and tests accordingly.
- Harden `gh_push` branch resolution: detect detached HEAD and surface a clear error or require an explicit branch argument; cover with tests that simulate detached/exotic branches.
- Housekeeping from observations: improve `update --dry-run` preview behaviour (optional if prioritised later) and fix the unclosed code block in `readme.md`, ensuring documentation renders correctly.
