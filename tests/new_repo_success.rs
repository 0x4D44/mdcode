use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_new_repository_creates_initial_commit() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // .git exists and there is a HEAD commit
    assert!(dir.join(".git").exists());
    let repo = Repository::open(&dir).unwrap();
    assert!(repo.head().is_ok());
}
