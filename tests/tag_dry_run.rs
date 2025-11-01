use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_tag_release_dry_run_prints_and_succeeds() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let dir = tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    new_repository(repo_dir.to_str().unwrap(), false, 50).unwrap();
    // set up remote to avoid push attempt when no_push=false
    let bare = dir.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    tag_release(
        repo_dir.to_str().unwrap(),
        Some("2.3.4".into()),
        Some("msg".into()),
        true,
        "origin",
        false,
        true,
        true,
    )
    .unwrap();
}
