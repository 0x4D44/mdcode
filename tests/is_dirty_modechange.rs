use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_is_dirty_detects_mode_change() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
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
        .arg("core.filemode")
        .arg("true")
        .status()
        .unwrap();
    let f = d.join("x.sh");
    std::fs::write(&f, "echo hi\n").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg("x.sh")
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
    // Replace file with a symlink (typechange)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        std::fs::remove_file(&f).unwrap();
        let target = d.join("target.txt");
        std::fs::write(&target, "t\n").unwrap();
        symlink(&target, &f).unwrap();
    }
    assert!(is_dirty(d.to_str().unwrap()).unwrap());
}
