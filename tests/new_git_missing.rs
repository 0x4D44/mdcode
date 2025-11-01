use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_new_repository_errors_when_git_missing() {
    // Simulate missing git by clearing PATH
    let orig_path = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    // Expect an error before any git invocation
    let err = new_repository(s, false, 50).unwrap_err();
    if let Some(p) = orig_path {
        std::env::set_var("PATH", p);
    }
    assert!(err.to_string().to_lowercase().contains("git not installed"));
}
