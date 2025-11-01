use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_various_invalid_modes() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo = temp.path().join("r");
    let repo_s = repo.to_str().unwrap();
    new_repository(repo_s, false, 50).unwrap();

    // Three-arg input falls back to current-after branch; should still run without panic
    diff_command(repo_s, &["0".into(), "1".into(), "2".into()], true).unwrap();

    // H with very large index should error out
    assert!(diff_command(repo_s, &["H".into(), "9999".into()], true).is_err());

    // L with index provided is invalid (mode mismatch)
    assert!(diff_command(repo_s, &["L".into(), "0".into()], true).is_err());

    // Non-numeric two-arg pair should error
    assert!(diff_command(repo_s, &["x".into(), "y".into()], true).is_err());
}
