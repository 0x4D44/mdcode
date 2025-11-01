use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_add_files_to_git_real_adds_to_index() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    let repo = Repository::init(d).unwrap();
    std::fs::write(d.join("x.rs"), "fn x(){}\n").unwrap();
    std::fs::write(d.join("y.rs"), "fn y(){}\n").unwrap();
    let files = vec![d.join("x.rs"), d.join("y.rs")];
    let added = add_files_to_git(d.to_str().unwrap(), &files, false).unwrap();
    assert_eq!(added, 2);
    let idx = repo.index().unwrap();
    assert!(idx.get_path(std::path::Path::new("x.rs"), 0).is_some());
    assert!(idx.get_path(std::path::Path::new("y.rs"), 0).is_some());
}
