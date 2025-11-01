use git2::Repository;
use mdcode::*;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_is_in_excluded_path_true_false() {
    assert!(is_in_excluded_path(Path::new("target/foo")));
    assert!(is_in_excluded_path(Path::new("venv/bin/python")));
    assert!(!is_in_excluded_path(Path::new("src/lib.rs")));
}

#[test]
fn test_create_temp_dir_makes_unique_directory() {
    let a = create_temp_dir("tmp.a").unwrap();
    let b = create_temp_dir("tmp.b").unwrap();
    assert!(a.exists());
    assert!(b.exists());
    assert_ne!(a, b);
}

#[test]
fn test_get_last_commit_matches_head() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let s = tmp.path().to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    let repo = Repository::open(s).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap().id();
    let last = get_last_commit(&repo).unwrap().id();
    assert_eq!(head, last);
}
