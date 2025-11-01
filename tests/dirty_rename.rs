use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_is_dirty_detects_rename_when_staged() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let dir = tempdir().unwrap();
    let d = dir.path();

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
        .arg("mdcode")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("user.email")
        .arg("md@code.local")
        .status()
        .unwrap();

    std::fs::write(d.join("a.txt"), "a").unwrap();
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

    // Rename the tracked file (staged rename)
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("mv")
        .arg("a.txt")
        .arg("b.txt")
        .status()
        .unwrap();
    // is_dirty should consider staged rename as dirty
    assert!(is_dirty(d.to_str().unwrap()).unwrap());
}
