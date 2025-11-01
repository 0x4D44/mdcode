use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_fetch_lists_remote_commits() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // work repo A
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

    // Create ahead commit on remote via A
    std::fs::write(a.join("x.txt"), "x").unwrap();
    update_repository(a_s, false, Some("x"), 50).unwrap();
    gh_push(a_s, "origin").unwrap();

    // work repo B: should fetch and list the remote commit available
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    let out_before = Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("log")
        .arg("--oneline")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out_before.stdout).lines().count() >= 1);
    gh_fetch(b.to_str().unwrap(), "origin").unwrap();
}
