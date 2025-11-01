use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_new_repository_errors_when_repo_already_exists() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    let err = new_repository(s, false, 50).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}
