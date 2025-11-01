use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_gh_create_via_cli_success_path() {
    // Create a dummy gh that always succeeds and accepts --version
    let tmp = tempdir().unwrap();
    let gh = tmp.path().join("gh");
    #[cfg(unix)]
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&gh).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(
            f,
            "if [ \"$1\" = \"--version\" ]; then echo gh version; exit 0; fi"
        )
        .unwrap();
        writeln!(f, "exit 0").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh, p).unwrap();
    }
    // Call directly; should succeed
    gh_create_via_cli(
        &gh,
        ".",
        "name",
        Some("desc".into()),
        RepoVisibility::Private,
    )
    .unwrap();
}
