use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_is_dirty_unstaged_rename_trips_head_not_found() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // init repo + identity
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("init")
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
    std::fs::write(d.join("a.txt"), "a\n").unwrap();
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
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .unwrap();
    // Rename file in filesystem but do not stage
    std::fs::rename(d.join("a.txt"), d.join("b.txt")).unwrap();
    assert!(is_dirty(d.to_str().unwrap()).unwrap());
}
