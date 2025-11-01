use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_add_remote_idempotent() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    let repo = Repository::init(d).unwrap();
    // First add should create the remote
    add_remote(
        d.to_str().unwrap(),
        "origin",
        "https://example.invalid/remote.git",
    )
    .unwrap();
    assert!(repo.find_remote("origin").is_ok());
    // Second add should be no-op and still succeed
    add_remote(
        d.to_str().unwrap(),
        "origin",
        "https://example.invalid/remote.git",
    )
    .unwrap();
}
