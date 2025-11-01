#[cfg(coverage)]
use git2::Repository;
#[cfg(coverage)]
use mdcode::*;
#[cfg(coverage)]
use tempfile::tempdir;

#[test]
#[cfg(coverage)]
fn test_resolve_signature_mdcode_fallback_with_clean_env() {
    let tmp = tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Clear env overrides
    for k in [
        "GIT_AUTHOR_NAME",
        "GIT_AUTHOR_EMAIL",
        "GIT_COMMITTER_NAME",
        "GIT_COMMITTER_EMAIL",
    ] {
        std::env::remove_var(k);
    }
    // Force coverage variant to ignore global/system git configs
    std::env::set_var("MDCODE_IGNORE_GLOBAL_GIT", "1");
    // Ensure repo-local config has no identity
    // Call and assert fallback
    let (sig, src) = resolve_signature_with_source(&repo).unwrap();
    assert_eq!(sig.name(), Some("mdcode"));
    assert_eq!(sig.email(), Some("mdcode@example.com"));
    assert_eq!(src, "mdcode fallback");
}
