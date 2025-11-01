use mdcode::*;
use tempfile::tempdir;

#[test]
#[cfg(coverage)]
fn test_tag_release_defaults_version_under_coverage() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    new_repository(s, false, 50).unwrap();
    // No version provided and no Cargo.toml: should use default during coverage
    tag_release(s, None, None, false, "origin", false, true, true).unwrap();
}

#[test]
fn test_update_repository_defaults_commit_message_under_coverage() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("repo");
    let s = dir.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Create a change and call with commit_msg=None
    std::fs::write(dir.join("a.txt"), "x").unwrap();
    update_repository(s, false, None, 50).unwrap();
}
