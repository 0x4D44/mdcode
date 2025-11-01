use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_get_remote_head_commit_symbolic_head_ok() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // Work repo A: create initial commit and push to origin
    let a = tmp.path().join("A");
    let a_s = a.to_str().unwrap();
    new_repository(a_s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&a)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(a_s, "origin").unwrap();

    // Clone to B so that refs/remotes/origin/HEAD points symbolically to origin/master
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    let b_s = b.to_str().unwrap();
    let repo_b = Repository::open(b_s).unwrap();

    // Function should resolve origin/HEAD -> origin/master and return a commit
    let _commit = get_remote_head_commit(&repo_b, b_s).unwrap();
}
