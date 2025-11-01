use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_execute_cli_dispatches_core_commands() {
    // Create a repo through the CLI entry (New), then run Update (dry-run), Info, Diff, GhPush, GhFetch, GhSync, Tag.
    let temp = tempdir().unwrap();
    let repo_path = temp.path().join("cli-dispatch");
    std::fs::create_dir_all(&repo_path).unwrap();
    let repo_str = repo_path.to_str().unwrap().to_string();

    // new
    let cli_new = Cli {
        command: Commands::New {
            directory: repo_str.clone(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli_new).unwrap();
    assert!(repo_path.join(".git").exists());

    // create change so update dry-run previews
    std::fs::write(repo_path.join("cli_dispatch.txt"), "pending change").unwrap();
    let cli_update = Cli {
        command: Commands::Update {
            directory: repo_str.clone(),
        },
        dry_run: true,
        max_file_mb: 50,
    };
    execute_cli(cli_update).unwrap();

    // info
    let cli_info = Cli {
        command: Commands::Info {
            directory: repo_str.clone(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli_info).unwrap();

    // diff (dry-run)
    let cli_diff = Cli {
        command: Commands::Diff {
            directory: repo_str.clone(),
            versions: Vec::new(),
        },
        dry_run: true,
        max_file_mb: 50,
    };
    execute_cli(cli_diff).unwrap();

    // Set up a bare remote
    let remote_dir = temp.path().join("cli-dispatch-remote.git");
    Repository::init_bare(&remote_dir).unwrap();
    let remote_url = remote_dir.to_str().unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(remote_url)
        .status()
        .expect("git remote add failed");

    // push
    let cli_push = Cli {
        command: Commands::GhPush {
            directory: repo_str.clone(),
            remote: "origin".to_string(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli_push).unwrap();

    // fetch
    let cli_fetch = Cli {
        command: Commands::GhFetch {
            directory: repo_str.clone(),
            remote: "origin".to_string(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli_fetch).unwrap();

    // sync
    let cli_sync = Cli {
        command: Commands::GhSync {
            directory: repo_str.clone(),
            remote: "origin".to_string(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli_sync).unwrap();

    // tag (dry-run/no-push) so it doesn't need network
    let cli_tag = Cli {
        command: Commands::Tag {
            directory: repo_str.clone(),
            version: Some("1.2.3".to_string()),
            message: None,
            no_push: true,
            remote: "origin".to_string(),
            force: false,
            allow_dirty: true,
        },
        dry_run: true,
        max_file_mb: 50,
    };
    execute_cli(cli_tag).unwrap();
}

#[test]
fn test_diff_command_invalid_index_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_path = temp.path().join("diff-invalid");
    let repo_str = repo_path.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let err = diff_command(repo_str, &[String::from("99")], true)
        .expect_err("invalid index should return an error");
    assert!(err.to_string().contains("invalid repo indexes specified"));
}

#[test]
fn test_diff_command_remote_head_and_local_current() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let remote_dir = temp.path().join("remote.git");
    Repository::init_bare(&remote_dir).unwrap();

    let repo_dir = temp.path().join("diff-repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();

    Command::new("git")
        .arg("-C")
        .arg(repo_str)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(remote_dir.to_str().unwrap())
        .status()
        .expect("git remote add failed");
    Command::new("git")
        .arg("-C")
        .arg(repo_str)
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("master")
        .status()
        .expect("git push failed");

    diff_command(repo_str, &["H".into(), "0".into()], true).unwrap();
    diff_command(repo_str, &["L".into()], true).unwrap();
}

#[test]
fn test_diff_command_executes_diff_tool_when_not_dry_run() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("diff-run");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    std::fs::write(repo_dir.join("tracked.txt"), "modified content").unwrap();
    update_repository(repo_str, false, Some("Modify"), 50).unwrap();
    // Use `true` so the custom tool exits successfully
    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    diff_command(repo_str, &[], false).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}
