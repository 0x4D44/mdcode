use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_resolve_signature_uses_repo_config_when_no_env() {
    let tmp = tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Clear env overrides
    let (orig_a, orig_e) = (
        std::env::var_os("GIT_AUTHOR_NAME"),
        std::env::var_os("GIT_AUTHOR_EMAIL"),
    );
    let (orig_cna, orig_cne) = (
        std::env::var_os("GIT_COMMITTER_NAME"),
        std::env::var_os("GIT_COMMITTER_EMAIL"),
    );
    std::env::remove_var("GIT_AUTHOR_NAME");
    std::env::remove_var("GIT_AUTHOR_EMAIL");
    std::env::remove_var("GIT_COMMITTER_NAME");
    std::env::remove_var("GIT_COMMITTER_EMAIL");

    // Set repo-local config
    repo.config()
        .unwrap()
        .set_str("user.name", "Repo User")
        .unwrap();
    repo.config()
        .unwrap()
        .set_str("user.email", "repo@example.com")
        .unwrap();

    let (sig, src) = resolve_signature_with_source(&repo).unwrap();
    assert_eq!(sig.name(), Some("Repo User"));
    assert_eq!(sig.email(), Some("repo@example.com"));
    assert!(src.contains("git config"));

    // Restore env
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
    if let Some(v) = orig_cna {
        std::env::set_var("GIT_COMMITTER_NAME", v);
    } else {
        std::env::remove_var("GIT_COMMITTER_NAME");
    }
    if let Some(v) = orig_cne {
        std::env::set_var("GIT_COMMITTER_EMAIL", v);
    } else {
        std::env::remove_var("GIT_COMMITTER_EMAIL");
    }
}

#[test]
fn test_remote_branch_exists_true_and_false() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // work repo
    let work = tmp.path().join("work");
    let ws = work.to_str().unwrap();
    new_repository(ws, false, 50).unwrap();
    // add origin and push initial
    repo_add_remote(ws, "origin", bare.to_str().unwrap());
    gh_push(ws, "origin").unwrap();
    // true case
    assert!(remote_branch_exists(ws, "origin", "master").unwrap());
    // false case
    assert!(!remote_branch_exists(ws, "origin", "nonexist").unwrap());
}

fn repo_add_remote(dir: &str, name: &str, url: &str) {
    std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("remote")
        .arg("add")
        .arg(name)
        .arg(url)
        .status()
        .unwrap();
}
