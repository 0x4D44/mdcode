use mdcode::*;
use tempfile::tempdir;

// Exercise gh_cli_path success path by shimming a `gh` that exits 0 on --version.
#[test]
fn test_gh_cli_path_returns_some_when_gh_succeeds() {
    // Only meaningful on Unix-like shells for this shim approach.
    #[cfg(unix)]
    {
        let temp = tempdir().unwrap();
        let bin = temp.path().join("gh");
        {
            use std::io::Write as _;
            let mut f = std::fs::File::create(&bin).unwrap();
            // Minimal `gh --version` implementation that just exits 0
            writeln!(f, "#!/bin/sh\nexit 0").unwrap();
        }
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();

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
        assert!(found.is_some());
    }
}
