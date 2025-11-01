use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_checkout_tree_to_dir_restores_files() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();

    // Create and commit content
    std::fs::write(repo_dir.join("a.txt"), "a1\n").unwrap();
    std::fs::create_dir_all(repo_dir.join("sub")).unwrap();
    std::fs::write(repo_dir.join("sub/b.txt"), "b1\n").unwrap();
    update_repository(repo_str, false, Some("add files"), 50).unwrap();

    // Obtain the commit tree
    let repo = Repository::open(repo_str).unwrap();
    let commit = get_last_commit(&repo).unwrap();
    let tree = commit.tree().unwrap();

    // Checkout into a temp target directory
    let target = tmp.path().join("out");
    std::fs::create_dir_all(&target).unwrap();
    checkout_tree_to_dir(&repo, &tree, &target).unwrap();
    assert!(target.join("a.txt").exists());
    assert!(target.join("sub/b.txt").exists());
}
