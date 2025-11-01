use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_single_invalid_parse_reports_error() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let t = tempdir().unwrap();
    let repo = t.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Single non-numeric arg should error with invalid indexes
    let err = diff_command(s, &["x".into()], true).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("invalid"));
}
