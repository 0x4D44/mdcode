use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_execute_cli_tag_missing_remote_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // No remote configured: push should error
    let cli = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("0.1.0".into()),
            message: None,
            no_push: false,
            remote: "origin".into(),
            force: false,
            allow_dirty: true,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    let err = execute_cli(cli).unwrap_err();
    assert!(
        err.to_string().contains("remote 'origin' not found")
            || err.to_string().contains("failed to push")
    );
}

#[test]
fn test_execute_cli_tag_force_overwrite_no_push_success() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // First create tag via CLI with no_push
    let cli1 = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("1.0.0".into()),
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
    // Force overwrite should succeed (still no push)
    let cli2 = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("1.0.0".into()),
            message: None,
            no_push: true,
            remote: "origin".into(),
            force: true,
            allow_dirty: true,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli2).unwrap();
}
