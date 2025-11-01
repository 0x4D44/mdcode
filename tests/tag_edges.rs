use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_tag_release_overwrite_error_and_force_success() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let dir_s = dir.to_str().unwrap();
    new_repository(dir_s, false, 50).unwrap();

    // set up remote for push in force case
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();

    // Create tag v1.0.0
    tag_release(
        dir_s,
        Some("1.0.0".into()),
        None,
        false,
        "origin",
        false,
        true,
        false,
    )
    .unwrap();
    // Attempt to create same tag without --force: expect error
    let err = tag_release(
        dir_s,
        Some("1.0.0".into()),
        None,
        false,
        "origin",
        false,
        true,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("already exists"));

    // Force update should succeed (still no push)
    tag_release(
        dir_s,
        Some("1.0.0".into()),
        None,
        false,
        "origin",
        true,
        true,
        false,
    )
    .unwrap();
}
