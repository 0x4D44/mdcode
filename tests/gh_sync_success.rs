use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_sync_happy_path_with_upstream() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    // Create bare origin
    let origin = tmp.path().join("origin.git");
    Repository::init_bare(&origin).unwrap();

    // Create repo A and push initial commit
    let a = tmp.path().join("A");
    let a_s = a.to_str().unwrap();
    new_repository(a_s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&a)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(origin.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(a_s, "origin").unwrap();

    // Create repo B by cloning origin; will run gh_sync against origin successfully
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(origin.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    let b_s = b.to_str().unwrap();
    // No changes pending; gh_sync should succeed
    gh_sync(b_s, "origin").unwrap();
}
