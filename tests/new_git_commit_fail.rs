#[cfg(coverage)]
use mdcode::*;
#[cfg(coverage)]
use tempfile::tempdir;

// Simulate `git commit` failure during new_repository to cover the error path.
#[test]
#[cfg(all(unix, coverage))]
fn test_new_repository_initial_commit_failure() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let repo_s = repo.to_str().unwrap();

    // Prepare a PATH shim that fails only for `git commit` and defers everything else to real git.
    let bin_dir = tmp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let shim = bin_dir.join("git");
    let real_git = which::which("git").unwrap();
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&shim).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        // check-git-installed path
        writeln!(f, "[ \"$1\" = \"--version\" ] && exit 0").unwrap();
        // fail commit path no matter where it appears in argv
        writeln!(
            f,
            "for a in \"$@\"; do [ \"$a\" = \"commit\" ] && exit 2; done"
        )
        .unwrap();
        // forward everything else
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
    let err = new_repository(repo_s, false, 50).unwrap_err();
    if let Some(p) = orig {
        std::env::set_var("PATH", p);
    }
    assert!(err.to_string().contains("Failed to create initial commit"));
}
