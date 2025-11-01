use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_new_repository_dry_run_does_not_create_git_dir() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.rs"), "fn main(){}\n").unwrap();
    new_repository(dir.to_str().unwrap(), true, 50).unwrap();
    assert!(!dir.join(".git").exists());
}
