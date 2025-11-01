use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_get_remote_head_commit_fetch_fails_without_remote() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().join("r");
    let s = repo_dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    let repo = git2::Repository::open(s).unwrap();
    // No remote 'origin' configured; fetch should fail and function should return Err
    assert!(get_remote_head_commit(&repo, s).is_err());
}
