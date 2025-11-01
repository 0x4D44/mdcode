use mdcode::*;
use std::io::Write as _;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_is_dirty_eol_only_not_dirty() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // init repo and set deterministic config
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
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("core.autocrlf")
        .arg("false")
        .status()
        .unwrap();

    let file = d.join("file.txt");
    std::fs::write(&file, "line1\nline2\n").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg("file.txt")
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

    // Rewrite file with CRLF (EOL-only change)
    let mut f = std::fs::File::create(&file).unwrap();
    write!(f, "line1\r\nline2\r\n").unwrap();
    drop(f);

    // is_dirty should consider it not dirty after normalization
    assert!(!is_dirty(d.to_str().unwrap()).unwrap());
}
