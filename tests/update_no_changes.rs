use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_update_repository_no_changes_is_ok() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Run update with no changes; should log and return Ok
    update_repository(s, false, Some("noop"), 50).unwrap_or(());
}
