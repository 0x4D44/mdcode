use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_tag_release_reads_version_from_cargo() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let dir = tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    std::fs::create_dir_all(repo_dir.join("src")).unwrap();
    std::fs::write(
        repo_dir.join("Cargo.toml"),
        "[package]\nname=\"demo\"\nversion=\"3.2.1\"\nedition=\"2021\"\n",
    )
    .unwrap();
    std::fs::write(repo_dir.join("src/lib.rs"), "pub fn main() {}\n").unwrap();
    new_repository(repo_dir.to_str().unwrap(), false, 50).unwrap();
    tag_release(
        repo_dir.to_str().unwrap(),
        None,
        None,
        false,
        "origin",
        false,
        true,
        true,
    )
    .unwrap();
}

#[test]
fn test_tag_release_pushes_tag_to_remote() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let dir = tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    new_repository(repo_dir.to_str().unwrap(), false, 50).unwrap();
    // set up remote
    let remote_dir = dir.path().join("tag-remote.git");
    Repository::init_bare(&remote_dir).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(remote_dir.to_str().unwrap())
        .status()
        .unwrap();
    // create and push tag
    tag_release(
        repo_dir.to_str().unwrap(),
        Some("1.0.0".to_string()),
        None,
        true,
        "origin",
        false,
        true,
        true,
    )
    .unwrap();
}

#[test]
fn test_tag_release_dirty_requires_flag() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let dir = tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    new_repository(repo_dir.to_str().unwrap(), false, 50).unwrap();
    // Create a tracked file, commit it, then modify to ensure dirty
    use std::io::Write as _;
    let tracked = repo_dir.join("tracked.txt");
    std::fs::write(&tracked, "one\n").unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("add")
        .arg("tracked.txt")
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("commit")
        .arg("-m")
        .arg("add tracked")
        .status()
        .unwrap();
    // modify to make worktree dirty
    let mut tf2 = std::fs::OpenOptions::new()
        .append(true)
        .open(&tracked)
        .unwrap();
    writeln!(tf2, "two").unwrap();
    drop(tf2);
    let err = tag_release(
        repo_dir.to_str().unwrap(),
        Some("1.2.3".to_string()),
        None,
        false,
        "origin",
        false,
        false,
        true,
    )
    .unwrap_err();
    assert!(err.to_string().contains("working tree"));
}
