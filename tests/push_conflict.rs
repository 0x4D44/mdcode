use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_push_merge_conflict_error_path() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // First worktree A
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

    // Second worktree B clones from bare
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    // Configure Git identity to avoid commit failures
    Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("config")
        .arg("user.name")
        .arg("mdcode")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("config")
        .arg("user.email")
        .arg("md@code.local")
        .status()
        .unwrap();

    // Diverge: commit in A and push
    std::fs::write(a.join("x.txt"), "one").unwrap();
    update_repository(a_s, false, Some("one"), 50).unwrap();
    gh_push(a_s, "origin").unwrap();

    // Commit conflicting change in B and try to push via our function, which pulls first and should error
    std::fs::write(b.join("x.txt"), "two").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("add")
        .arg("x.txt")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("commit")
        .arg("-m")
        .arg("two")
        .status()
        .unwrap();
    let err = gh_push(b.to_str().unwrap(), "origin").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Merge failed") || msg.contains("Failed to push changes"));
}
