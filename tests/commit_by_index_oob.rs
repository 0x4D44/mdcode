use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_get_commit_by_index_out_of_bounds_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // Initialize a repo with one commit
    Repository::init(d).unwrap();
    std::fs::write(d.join("a.txt"), "hi").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg("a.txt")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("user.name")
        .arg("md")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("user.email")
        .arg("md@x")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .unwrap();
    let repo = Repository::open(d).unwrap();
    let err = get_commit_by_index(&repo, 5).unwrap_err();
    assert!(err.to_string().contains("Index out of bounds"));
}
