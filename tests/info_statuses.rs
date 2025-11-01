use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_info_repository_lists_added_modified_deleted() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // Add a file (Added)
    std::fs::write(repo.join("a.txt"), "a\n").unwrap();
    update_repository(s, false, Some("add a"), 50).unwrap();
    // Modify it (Modified)
    std::fs::write(repo.join("a.txt"), "a\nmod\n").unwrap();
    update_repository(s, false, Some("modify a"), 50).unwrap();
    // Delete it (Deleted)
    std::fs::remove_file(repo.join("a.txt")).unwrap();
    update_repository(s, false, Some("delete a"), 50).unwrap();
    // Should not error
    info_repository(s).unwrap();
}
