use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_add_files_to_git_dry_run_leaves_index_empty() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    let repo = Repository::init(d).unwrap();
    std::fs::write(d.join("a.rs"), "fn a(){}\n").unwrap();
    std::fs::write(d.join("b.rs"), "fn b(){}\n").unwrap();
    let files = vec![d.join("a.rs"), d.join("b.rs")];
    let added = add_files_to_git(d.to_str().unwrap(), &files, true).unwrap();
    assert_eq!(added, 2);
    // index should remain empty because of dry_run
    let idx = repo.index().unwrap();
    assert_eq!(idx.len(), 0);
}
