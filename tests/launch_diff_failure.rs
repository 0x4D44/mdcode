use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_launch_diff_tool_without_custom_tool_errors() {
    // When MDCODE_DIFF_TOOL is unset, coverage variant returns an error.
    let a = tempdir().unwrap();
    let b = tempdir().unwrap();
    let err = launch_diff_tool(a.path(), b.path()).unwrap_err();
    assert!(
        err.to_string().contains("failed to launch diff tool")
            || err.to_string().contains("custom diff tool failed")
            || err.to_string().contains("Failed to launch both diff tools")
    );
}
