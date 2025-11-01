use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_gh_push_initial_and_subsequent_push() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("push-repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();

    // Create remote and set origin
    let remote_dir = temp.path().join("push-remote.git");
    Repository::init_bare(&remote_dir).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(repo_str)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(remote_dir.to_str().unwrap())
        .status()
        .unwrap();

    // Initial push should set upstream
    gh_push(repo_str, "origin").unwrap();

    // Modify and push again
    std::fs::write(repo_dir.join("file.txt"), "change").unwrap();
    update_repository(repo_str, false, Some("change"), 50).unwrap();
    gh_push(repo_str, "origin").unwrap();
}

#[test]
fn test_gh_fetch_and_sync_with_local_remote() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let remote_dir = temp.path().join("remote.git");
    Repository::init_bare(&remote_dir).unwrap();

    // Local repo
    let repo_dir = temp.path().join("fetch-repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();

    // Add origin and push
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

    // Fetch should succeed even when up-to-date
    gh_fetch(repo_str, "origin").unwrap();
    // Sync should detect remote branch and run a pull path
    gh_sync(repo_str, "origin").unwrap();
}

#[test]
fn test_gh_fetch_missing_remote_returns_error() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();

    // Add a bogus remote URL, then run fetch which should fail
    Command::new("git")
        .arg("-C")
        .arg(repo_str)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("/path/does/not/exist")
        .status()
        .unwrap();
    let err = gh_fetch(repo_str, "origin").unwrap_err();
    assert!(err.to_string().contains("git fetch failed"));
    // gh_sync should return Ok and print missing branch note when upstream missing
    gh_sync(repo_str, "origin").unwrap();
}

#[test]
fn test_resolve_signature_precedence() {
    // Ensure env author takes precedence, else fallback to repo/global config.
    let temp = tempdir().unwrap();
    let repo = Repository::init(temp.path()).unwrap();
    // Save originals
    let orig_a = std::env::var_os("GIT_AUTHOR_NAME");
    let orig_e = std::env::var_os("GIT_AUTHOR_EMAIL");
    // Only set author; committer unset should still use author values
    std::env::set_var("GIT_AUTHOR_NAME", "Author Name");
    std::env::set_var("GIT_AUTHOR_EMAIL", "author@example.com");
    let (sig, src) = resolve_signature_with_source(&repo).unwrap();
    assert_eq!(sig.name(), Some("Author Name"));
    assert_eq!(sig.email(), Some("author@example.com"));
    assert_eq!(src, "env:GIT_AUTHOR_NAME/GIT_AUTHOR_EMAIL");
    // restore
    if let Some(v) = orig_a {
        std::env::set_var("GIT_AUTHOR_NAME", v);
    } else {
        std::env::remove_var("GIT_AUTHOR_NAME");
    }
    if let Some(v) = orig_e {
        std::env::set_var("GIT_AUTHOR_EMAIL", v);
    } else {
        std::env::remove_var("GIT_AUTHOR_EMAIL");
    }
}
