use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_two_args_mixed_numeric_non_numeric_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let t = tempdir().unwrap();
    let repo = t.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // First numeric ok to parse, second invalid should hit error branch
    assert!(diff_command(s, &["2".into(), "x".into()], true).is_err());
}
