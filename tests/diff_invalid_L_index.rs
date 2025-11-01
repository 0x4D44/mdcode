use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_l_with_index_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let err = diff_command(repo_str, &["L".into(), "0".into()], true).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("invalid"));
}
