use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_launch_diff_tool_custom_fails_with_not_found() {
    // Set a non-existent program to hit the Err(e) branch
    std::env::set_var("MDCODE_DIFF_TOOL", "definitely-not-a-real-binary-xyz");
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    let err = launch_diff_tool(a.path(), b.path()).unwrap_err();
    assert!(err.to_string().contains("custom diff tool failed"));
    std::env::remove_var("MDCODE_DIFF_TOOL");
}
