use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_execute_cli_tag_exists_error_no_force() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // Set a remote so execute_cli can validate push flags when requested
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();

    // First tag creation succeeds (no push)
    let cli1 = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("1.2.3".into()),
            message: None,
            no_push: true,
            remote: "origin".into(),
            force: false,
            allow_dirty: true,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli1).unwrap();
    // Second creation without --force should error
    let cli2 = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("1.2.3".into()),
            message: None,
            no_push: true,
            remote: "origin".into(),
            force: false,
            allow_dirty: true,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    let e = execute_cli(cli2).unwrap_err();
    assert!(e.to_string().contains("already exists"));
}
