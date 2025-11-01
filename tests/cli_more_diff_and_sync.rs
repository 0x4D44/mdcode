use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_execute_cli_diff_numeric_and_two_index() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    // Create repo with two extra commits
    new_repository(&s, false, 50).unwrap();
    // Add a dummy remote so get_remote_head_commit won't fail when L path triggers remote lookup
    let bare = tmp.path().join("remote.git");
    git2::Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    // Push initial commit so origin has a default branch
    gh_push(&s, "origin").unwrap();
    std::fs::write(repo.join("f.txt"), "1").unwrap();
    update_repository(&s, false, Some("c1"), 50).unwrap();
    std::fs::write(repo.join("f.txt"), "2").unwrap();
    update_repository(&s, false, Some("c2"), 50).unwrap();

    // single numeric index
    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    let cli1 = Cli {
        command: Commands::Diff {
            directory: s.clone(),
            versions: vec!["1".into()],
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli1).unwrap();
    // two indices
    let cli2 = Cli {
        command: Commands::Diff {
            directory: s.clone(),
            versions: vec!["2".into(), "1".into()],
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli2).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}

#[test]
fn test_execute_cli_diff_l_current_launches_tool() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // Add remote and push initial so get_remote_head_commit works
    let bare = tmp.path().join("remote.git");
    git2::Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(&s, "origin").unwrap();
    // modify working tree so L compares different trees
    std::fs::write(repo.join("x.txt"), "x").unwrap();
    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    let cli = Cli {
        command: Commands::Diff {
            directory: s.clone(),
            versions: vec!["L".into()],
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}

#[test]
fn test_execute_cli_diff_h_vs_index_launches_tool() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    git2::Repository::init_bare(&bare).unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(&s, "origin").unwrap();
    // add one local commit and compare
    std::fs::write(repo.join("y.txt"), "y").unwrap();
    update_repository(&s, false, Some("y"), 50).unwrap();
    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    let cli = Cli {
        command: Commands::Diff {
            directory: s.clone(),
            versions: vec!["H".into(), "0".into()],
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}

#[test]
fn test_execute_cli_sync_missing_remote_branch() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // Add a bogus remote URL so branch lookup fails
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("/does/not/exist")
        .status()
        .unwrap();
    let cli = Cli {
        command: Commands::GhSync {
            directory: s.clone(),
            remote: "origin".into(),
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli).unwrap();
}
