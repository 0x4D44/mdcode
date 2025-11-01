use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_diff_command_two_indices_invalid_second_errors() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let t = tempdir().unwrap();
    let d = t.path().join("r");
    let s = d.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    std::fs::write(d.join("f.txt"), "1").unwrap();
    update_repository(s, false, Some("c1"), 50).unwrap();
    // invalid second index
    let err = diff_command(s, &["0".into(), "99".into()], true).unwrap_err();
    assert!(err.to_string().contains("invalid repo indexes specified"));
}
