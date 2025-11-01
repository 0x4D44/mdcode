use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_gh_create_via_cli_visibility_public_internal() {
    let t = tempdir().unwrap();
    let gh = t.path().join("gh");
    #[cfg(unix)]
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&gh).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "exit 0").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh, p).unwrap();
    }
    gh_create_via_cli(&gh, ".", "n1", Some("d".into()), RepoVisibility::Public).unwrap();
    gh_create_via_cli(&gh, ".", "n2", Some("d".into()), RepoVisibility::Internal).unwrap();
}
