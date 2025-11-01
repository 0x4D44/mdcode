use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_is_dirty_no_commits_returns_false() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    // Initialize an empty repo with no commits
    std::process::Command::new("git")
        .arg("-C")
        .arg(tmp.path())
        .arg("init")
        .status()
        .unwrap();
    // is_dirty should treat no-commit repos as not dirty
    let d = tmp.path().to_str().unwrap();
    assert!(!is_dirty(d).unwrap());
}
