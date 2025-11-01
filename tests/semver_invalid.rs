use mdcode::*;

#[test]
fn test_normalize_semver_tag_invalid_errors() {
    assert!(normalize_semver_tag("").is_err());
    assert!(normalize_semver_tag("v").is_err());
}
