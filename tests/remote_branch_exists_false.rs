use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_remote_branch_exists_false_when_missing() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // No remote configured; should return Ok(false)
    let exists = remote_branch_exists(s, "origin", "master").unwrap();
    assert!(!exists);
}
