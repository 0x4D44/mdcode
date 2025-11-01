use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_tag_release_push_errors_when_remote_missing() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // push=true but remote doesn't exist
    let err = tag_release(
        s,
        Some("0.1.0".into()),
        None,
        true,
        "origin",
        false,
        true,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("remote 'origin' not found"));
}
