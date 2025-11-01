use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_is_dirty_detects_index_new() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // init repo and identity
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
    // initial commit
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
    // stage a new file (INDEX_NEW)
    std::fs::write(d.join("b.txt"), "b\n").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg("b.txt")
        .status()
        .unwrap();
    assert!(is_dirty(d.to_str().unwrap()).unwrap());
}
