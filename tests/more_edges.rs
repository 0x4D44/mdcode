use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_get_remote_head_commit_remote_show_fallback() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let remote_dir = temp.path().join("remote.git");
    Repository::init_bare(&remote_dir).unwrap();

    // local repo
    let repo_dir = temp.path().join("local");
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
        .unwrap();
    gh_push(repo_str, "origin").unwrap();

    // Remove the refs/remotes/origin/HEAD to force the fallback path
    let repo = Repository::open(repo_str).unwrap();
    // Fetch first to create remote tracking refs
    Command::new("git")
        .arg("-C")
        .arg(repo_str)
        .arg("fetch")
        .arg("origin")
        .status()
        .unwrap();
    if let Ok(mut r) = repo.find_reference("refs/remotes/origin/HEAD") {
        r.delete().unwrap();
    }
    // Now call helper
    let _ = get_remote_head_commit(&repo, repo_str).unwrap();
}

#[test]
fn test_gh_cli_path_env_shim() {
    // Create a fake `gh` on PATH that returns 0 for --version
    let temp = tempdir().unwrap();
    let bin = temp.path().join("gh");
    std::fs::write(&bin, b"#!/bin/sh\necho gh version\n").unwrap();
    // make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();
    }
    let orig_path = std::env::var_os("PATH");
    let new_path = format!(
        "{}:{}",
        temp.path().to_str().unwrap(),
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", new_path);
    let found = gh_cli_path();
    // restore PATH
    if let Some(p) = orig_path {
        std::env::set_var("PATH", p);
    }
    assert!(found.is_some());
}

#[test]
fn test_add_remote_idempotent() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    add_remote(repo_str, "origin", "/tmp/some/remote").unwrap_or(());
    // second call should not fail
    add_remote(repo_str, "origin", "/tmp/some/remote").unwrap();
}

#[test]
fn test_gh_push_errors_on_detached_head() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    // detach HEAD
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("checkout")
        .arg("--detach")
        .arg("HEAD")
        .status()
        .unwrap();
    let err = gh_push(repo_str, "origin").unwrap_err();
    assert!(
        err.to_string().contains("Failed to push") || err.to_string().contains("git push failed")
    );
}
