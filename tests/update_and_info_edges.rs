use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_update_repository_dry_run_preview() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    use std::io::Write as _;
    let mut gitignore = std::fs::OpenOptions::new()
        .append(true)
        .open(repo_path.join(".gitignore"))
        .unwrap();
    writeln!(gitignore, "# dry-run change").unwrap();
    drop(gitignore);
    update_repository(repo_str, true, None, 50).unwrap();
}

#[test]
fn test_info_repository_missing_repo_errors() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("not-a-repo");
    let err = info_repository(missing.to_str().unwrap()).unwrap_err();
    assert!(err.to_string().contains("No git repository"));
}

#[test]
fn test_info_repository_empty_repo_errors() {
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    // initialize empty repo but no commits
    std::process::Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("init")
        .status()
        .unwrap();
    let err = info_repository(repo_dir.to_str().unwrap()).unwrap_err();
    assert!(err.to_string().contains("Empty repository"));
}
