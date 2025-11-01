use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_sync_fast_forward_success() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // Create work repo A and push initial
    let a = tmp.path().join("A");
    let a_s = a.to_str().unwrap();
    new_repository(a_s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&a)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(a_s, "origin").unwrap();

    // Clone to B (behind)
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();

    // Add commit on A and push so remote is ahead
    std::fs::write(a.join("x.txt"), "x").unwrap();
    update_repository(a_s, false, Some("x"), 50).unwrap();
    gh_push(a_s, "origin").unwrap();

    // Now sync on B should fast-forward and succeed
    gh_sync(b.to_str().unwrap(), "origin").unwrap();
}
