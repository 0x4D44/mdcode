#[cfg(coverage)]
use mdcode::*;
#[cfg(coverage)]
use tempfile::tempdir;

// Simulate `git tag` failure to cover the error path in tag_release (coverage variant).
#[test]
#[cfg(all(unix, coverage))]
fn test_tag_release_git_tag_failure() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();

    // Shim git to fail `tag` while delegating to real git otherwise.
    let bin_dir = tmp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let shim = bin_dir.join("git");
    let real_git = which::which("git").unwrap();
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&shim).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "[ \"$1\" = \"--version\" ] && exit 0").unwrap();
        writeln!(
            f,
            "for a in \"$@\"; do [ \"$a\" = \"tag\" ] && exit 2; done"
        )
        .unwrap();
        writeln!(f, "exec {} \"$@\"", real_git.to_string_lossy()).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    let mut p = std::fs::metadata(&shim).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&shim, p).unwrap();
    let orig = std::env::var_os("PATH");
    std::env::set_var(
        "PATH",
        format!(
            "{}:{}",
            bin_dir.to_str().unwrap(),
            std::env::var("PATH").unwrap()
        ),
    );
    let err = tag_release(
        s,
        Some("1.0.0".into()),
        None,
        false,
        "origin",
        false,
        true,
        false,
    )
    .unwrap_err();
    if let Some(p) = orig {
        std::env::set_var("PATH", p);
    }
    assert!(err.to_string().contains("failed to create tag via git"));
}
