use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_get_commit_by_index_in_bounds() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // init and identity
    let repo = Repository::init(d).unwrap();
    repo.config().unwrap().set_str("user.name", "u").unwrap();
    repo.config().unwrap().set_str("user.email", "e@x").unwrap();
    // commit 1
    std::fs::write(d.join("a.txt"), "a\n").unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg(".")
        .status()
        .unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("commit")
        .arg("-m")
        .arg("c1")
        .status()
        .unwrap();
    // commit 2
    std::fs::write(d.join("b.txt"), "b\n").unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg(".")
        .status()
        .unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("commit")
        .arg("-m")
        .arg("c2")
        .status()
        .unwrap();

    // latest
    let c0 = get_commit_by_index(&repo, 0).unwrap();
    let c1 = get_commit_by_index(&repo, 1).unwrap();
    assert_ne!(c0.id(), c1.id());
}
