use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_create_gitignore_dry_run_does_not_write() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();
    create_gitignore(dir.to_str().unwrap(), true).unwrap();
    assert!(!dir.join(".gitignore").exists());
}
