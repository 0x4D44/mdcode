use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_get_remote_head_commit_fetch_fails_with_bad_remote() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().join("r");
    let s = repo_dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Add an origin remote pointing to a non-existent path; fetch should fail
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("/no/such/remote")
        .status()
        .unwrap();
    let repo = Repository::open(s).unwrap();
    let err = get_remote_head_commit(&repo, s).unwrap_err();
    assert!(err.to_string().contains("git fetch failed"));
}
