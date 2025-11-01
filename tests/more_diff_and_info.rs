use mdcode::*;
use std::io::Write as _;
use tempfile::tempdir;

#[test]
fn test_diff_command_non_numeric_single_arg_error() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let err = diff_command(repo_str, &["abc".into()], true).unwrap_err();
    assert!(err.to_string().contains("invalid repo indexes specified"));
}

#[test]
fn test_diff_command_two_args_invalid_modes_error() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    // Two numeric indices that are far out-of-range should fail after bounds check
    let err = diff_command(repo_str, &["999".into(), "1000".into()], true).unwrap_err();
    eprintln!("two-arg invalid modes error: {}", err);
    assert!(err.to_string().to_lowercase().contains("invalid"));
    // H with non-numeric second arg
    let err = diff_command(repo_str, &["H".into(), "x".into()], true).unwrap_err();
    eprintln!("H non-numeric error: {}", err);
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("invalid") || msg.contains("remote 'origin' not found"));
}

#[test]
fn test_info_repository_with_twenty_plus_commits() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("big-repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let file = repo_dir.join("file.txt");
    std::fs::write(&file, "0\n").unwrap();
    update_repository(repo_str, false, Some("c0"), 50).unwrap();
    for i in 1..=21 {
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap();
        writeln!(f, "{}", i).unwrap();
        drop(f);
        update_repository(repo_str, false, Some(&format!("c{}", i)), 50).unwrap();
    }
    // Should iterate and print all commits without error; exercises display_index logic
    info_repository(repo_str).unwrap();
}

#[test]
fn test_diff_command_two_indices_path() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    let repo_str = repo_dir.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    // add two extra commits
    std::fs::write(repo_dir.join("f.txt"), "1").unwrap();
    update_repository(repo_str, false, Some("c1"), 50).unwrap();
    std::fs::write(repo_dir.join("f.txt"), "2").unwrap();
    update_repository(repo_str, false, Some("c2"), 50).unwrap();
    // Use a custom diff tool that succeeds so launch path is exercised
    std::env::set_var("MDCODE_DIFF_TOOL", "true");
    diff_command(repo_str, &["2".into(), "1".into()], false).unwrap();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}

#[test]
#[cfg(unix)]
fn test_launch_diff_tool_custom_fail() {
    use std::path::Path;
    std::env::set_var("MDCODE_DIFF_TOOL", "false");
    let _ = launch_diff_tool(Path::new("/tmp/a"), Path::new("/tmp/b")).unwrap_err();
    std::env::remove_var("MDCODE_DIFF_TOOL");
}

#[test]
#[cfg(not(windows))]
fn test_launch_diff_tool_dual_failure_returns_error() {
    // Force custom tool path to be absent and ensure built-in fallbacks fail.
    std::env::remove_var("MDCODE_DIFF_TOOL");
    use std::path::Path;
    let a = Path::new("/tmp/nonexistent-a");
    let b = Path::new("/tmp/nonexistent-b");
    let err = launch_diff_tool(a, b).unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("failed to launch") || msg.contains("custom diff tool"));
}
