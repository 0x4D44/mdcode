use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_update_repository_commits_changes_under_coverage() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Modify a file and commit via update_repository
    std::fs::write(repo.join("x.txt"), "x\n").unwrap();
    update_repository(s, false, Some("update"), 50).unwrap();
    // Run again with no changes to exercise early exit path
    update_repository(s, false, Some("noop"), 50).unwrap();
}
