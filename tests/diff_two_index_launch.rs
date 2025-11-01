use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_two_indices_launches_tool() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    // add two commits
    std::fs::write(repo.join("f.txt"), "1\n").unwrap();
    update_repository(s, false, Some("c1"), 50).unwrap();
    std::fs::write(repo.join("f.txt"), "2\n").unwrap();
    update_repository(s, false, Some("c2"), 50).unwrap();

    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    diff_command(s, &["2".into(), "1".into()], false).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}
