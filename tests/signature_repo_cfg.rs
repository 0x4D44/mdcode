use git2::Repository;
use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_resolve_signature_uses_repo_or_global_config() {
    let tmp = tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    // Ensure env variables are not set
    std::env::remove_var("GIT_AUTHOR_NAME");
    std::env::remove_var("GIT_AUTHOR_EMAIL");
    std::env::remove_var("GIT_COMMITTER_NAME");
    std::env::remove_var("GIT_COMMITTER_EMAIL");
    // Write repo config
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Repo User").unwrap();
    cfg.set_str("user.email", "repo@example.com").unwrap();

    let (sig, src) = resolve_signature_with_source(&repo).unwrap();
    assert_eq!(sig.name(), Some("Repo User"));
    assert_eq!(sig.email(), Some("repo@example.com"));
    assert_eq!(src, "git config (repo/global)");
}
