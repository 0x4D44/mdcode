use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_fetch_lists_multiple_commits() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // Repo A: create and push two ahead commits
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
    std::fs::write(a.join("x.txt"), "x1").unwrap();
    update_repository(a_s, false, Some("x1"), 50).unwrap();
    gh_push(a_s, "origin").unwrap();
    std::fs::write(a.join("x.txt"), "x2").unwrap();
    update_repository(a_s, false, Some("x2"), 50).unwrap();
    gh_push(a_s, "origin").unwrap();

    // Repo B: clone and fetch; should traverse listing path deterministically
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    gh_fetch(b.to_str().unwrap(), "origin").unwrap();
}
