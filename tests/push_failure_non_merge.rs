use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[serial_test::serial]
#[cfg(unix)]
fn test_gh_push_plain_push_failure_path() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    let repo = tmp.path().join("repo");
    let s = repo.to_str().unwrap();
    new_repository(s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    // Initial push to set branch
    gh_push(s, "origin").unwrap();

    // PATH shim that causes `git push` to fail non-zero
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let shim = bin.join("git");
    let real_git = which_git();
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&shim).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(
            f,
            "if [ \"$1\" = \"--version\" ]; then exec {} \"$@\"; fi",
            real_git
        )
        .unwrap();
        // Fail only on push; preserve args even with -C present
        writeln!(f, "case \"$*\" in *\" push \"*) exit 1 ;; esac").unwrap();
        writeln!(f, "exec {} \"$@\"", real_git).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    let mut p = std::fs::metadata(&shim).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&shim, p).unwrap();

    let orig_path = std::env::var("PATH").unwrap();
    std::env::set_var("PATH", format!("{}:{}", bin.to_string_lossy(), orig_path));

    let err = gh_push(s, "origin").unwrap_err();
    assert!(err.to_string().contains("Failed to push changes."));

    std::env::set_var("PATH", orig_path);
}

#[cfg(unix)]
fn which_git() -> String {
    let out = Command::new("which").arg("git").output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
