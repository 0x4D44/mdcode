use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_gh_cli_path_returns_none_when_gh_fails() {
    // Prepend a `gh` shim that exits non-zero so `--version` check fails
    let temp = tempdir().unwrap();
    let bin = temp.path().join("gh");
    #[cfg(unix)]
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&bin).unwrap();
        writeln!(f, "#!/bin/sh\nexit 2").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();
    }
    let orig = std::env::var_os("PATH");
    let newp = format!(
        "{}:{}",
        temp.path().to_str().unwrap(),
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", newp);
    let found = gh_cli_path();
    if let Some(p) = orig {
        std::env::set_var("PATH", p);
    }
    assert!(found.is_none());
}
