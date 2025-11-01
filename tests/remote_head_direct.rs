use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_get_remote_head_commit_handles_direct_ref() {
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

    // Clone to B
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    let b_s = b.to_str().unwrap();
    let repo_b = Repository::open(b_s).unwrap();

    // Determine the oid of origin/master and set refs/remotes/origin/HEAD to a direct ref
    let out = Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("rev-parse")
        .arg("origin/master")
        .output()
        .unwrap();
    assert!(out.status.success());
    let oid = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Make origin/HEAD a direct ref
    Command::new("git")
        .arg("-C")
        .arg(&b)
        .arg("update-ref")
        .arg("refs/remotes/origin/HEAD")
        .arg(&oid)
        .status()
        .unwrap();

    // Now get_remote_head_commit should take the direct-target branch and succeed
    let _commit = get_remote_head_commit(&repo_b, b_s).unwrap();
}
