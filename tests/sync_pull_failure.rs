use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[serial_test::serial]
#[cfg(unix)]
fn test_gh_sync_pull_failure_path() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    // Seed remote with one commit via A
    let a = tmp.path().join("A");
    let a_s = a.to_str().unwrap();
    new_repository(a_s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&a)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(a_s, "origin").unwrap();

    // Clone to B (up-to-date), ensure remote branch exists
    let b = tmp.path().join("B");
    Command::new("git")
        .arg("clone")
        .arg(bare.to_str().unwrap())
        .arg(&b)
        .status()
        .unwrap();
    let b_s = b.to_str().unwrap();

    // PATH shim that makes `git pull` fail
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
        // Fail only on pull; detect regardless of -C position
        writeln!(f, "case \"$*\" in *\" pull \"*) exit 1 ;; esac").unwrap();
        writeln!(f, "exec {} \"$@\"", real_git).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    let mut p = std::fs::metadata(&shim).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&shim, p).unwrap();

    let orig_path = std::env::var("PATH").unwrap();
    std::env::set_var("PATH", format!("{}:{}", bin.to_string_lossy(), orig_path));

    let err = gh_sync(b_s, "origin").unwrap_err();
    assert!(err.to_string().contains("git pull failed"));

    std::env::set_var("PATH", orig_path);
}

#[cfg(unix)]
fn which_git() -> String {
    let out = Command::new("which").arg("git").output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
