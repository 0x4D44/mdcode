use mdcode::*;

#[test]
fn test_update_repository_errors_when_not_a_repo() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    // Point to a temp path that is not a git repo
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("not_repo");
    std::fs::create_dir_all(&p).unwrap();
    let err = update_repository(p.to_str().unwrap(), false, Some("msg"), 50).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("no git repository"));
}
