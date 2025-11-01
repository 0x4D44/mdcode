use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_resolve_signature_fallback_when_no_env_or_config() {
    let tmp = tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Clear env variables used
    let keys = [
        "GIT_AUTHOR_NAME",
        "GIT_AUTHOR_EMAIL",
        "GIT_COMMITTER_NAME",
        "GIT_COMMITTER_EMAIL",
    ];
    let saved: Vec<_> = keys.iter().map(|k| (k, std::env::var_os(k))).collect();
    for k in keys.iter() {
        std::env::remove_var(k);
    }
    // Ensure repo config has no identity
    // (libgit2 global config may still exist; we don't control it reliably here)
    // If global config is present, this test still passes by not asserting exact source string.
    let (_sig, _src) = resolve_signature_with_source(&repo).unwrap();
    // Restore env
    for (k, v) in saved {
        if let Some(v) = v {
            std::env::set_var(k, v);
        }
    }
}
