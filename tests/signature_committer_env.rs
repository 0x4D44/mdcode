use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_resolve_signature_uses_committer_env_when_author_unset() {
    let tmp = tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Clear author env if present and set committer env
    let orig_an = std::env::var_os("GIT_AUTHOR_NAME");
    let orig_ae = std::env::var_os("GIT_AUTHOR_EMAIL");
    if orig_an.is_some() {
        std::env::remove_var("GIT_AUTHOR_NAME");
    }
    if orig_ae.is_some() {
        std::env::remove_var("GIT_AUTHOR_EMAIL");
    }
    std::env::set_var("GIT_COMMITTER_NAME", "Committer Name");
    std::env::set_var("GIT_COMMITTER_EMAIL", "committer@example.com");

    let (sig, src) = resolve_signature_with_source(&repo).unwrap();
    assert_eq!(sig.name(), Some("Committer Name"));
    assert_eq!(sig.email(), Some("committer@example.com"));
    assert_eq!(src, "env:GIT_COMMITTER_NAME/GIT_COMMITTER_EMAIL");

    // restore
    if let Some(v) = orig_an {
        std::env::set_var("GIT_AUTHOR_NAME", v);
    } else {
        std::env::remove_var("GIT_AUTHOR_NAME");
    }
    if let Some(v) = orig_ae {
        std::env::set_var("GIT_AUTHOR_EMAIL", v);
    } else {
        std::env::remove_var("GIT_AUTHOR_EMAIL");
    }
    std::env::remove_var("GIT_COMMITTER_NAME");
    std::env::remove_var("GIT_COMMITTER_EMAIL");
}
