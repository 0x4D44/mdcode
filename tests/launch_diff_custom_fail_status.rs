use mdcode::*;
use tempfile::tempdir;

// When MDCODE_DIFF_TOOL exists but returns non-zero, we should get "custom diff tool failed".
#[test]
#[cfg(unix)]
fn test_launch_diff_tool_custom_fail_status() {
    std::env::set_var("MDCODE_DIFF_TOOL", "/bin/false");
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    let err = launch_diff_tool(a.path(), b.path()).unwrap_err();
    assert!(err.to_string().contains("custom diff tool failed"));
    std::env::remove_var("MDCODE_DIFF_TOOL");
}
