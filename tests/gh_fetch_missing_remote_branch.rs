use std::fs;

use mdcode::{add_remote, gh_fetch};

fn check_git_installed() -> bool {
    which::which("git").is_ok()
}

#[test]
fn test_gh_fetch_returns_ok_when_remote_branch_missing() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let remote_dir = tmp.path().join("remote.git");
    fs::create_dir(&dir).unwrap();
    // init local repo + one commit so HEAD points to a branch
    assert!(std::process::Command::new("git")
        .args(["init"])
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    fs::write(dir.join(".gitignore"), b"target/\n").unwrap();
    assert!(std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(std::process::Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "init",
        ])
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    // init bare remote with no branches
    assert!(std::process::Command::new("git")
        .args(["init", "--bare", remote_dir.to_str().unwrap()])
        .current_dir(tmp.path())
        .status()
        .unwrap()
        .success());
    // add remote but do not push any branch
    add_remote(
        dir.to_str().unwrap(),
        "origin",
        &format!("file://{}", remote_dir.display()),
    )
    .unwrap();

    // Should early-return Ok(()) because remote branch does not exist
    gh_fetch(dir.to_str().unwrap(), "origin").unwrap();
}
