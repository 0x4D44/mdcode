use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

// Force the coverage-only fallback path in get_remote_head_commit() to run and fail
// at `git remote show origin` by injecting a PATH shim for `git`.
#[test]
#[serial_test::serial]
#[cfg(unix)]
fn test_remote_head_fallback_remote_show_fails() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();

    // Create a bare remote and a local repo with an initial commit pushed
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();

    let repo_dir = tmp.path().join("work");
    let repo_s = repo_dir.to_str().unwrap();
    new_repository(repo_s, false, 50).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();
    gh_push(repo_s, "origin").unwrap();

    // Write a PATH shim for `git` that makes `git remote show origin` fail
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let shim = bin.join("git");
    let real_git = which_git();
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&shim).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        // Pass through --version (cargo-llvm-cov probes it sometimes)
        writeln!(
            f,
            "if [ \"$1\" = \"--version\" ]; then exec {} \"$@\"; fi",
            real_git
        )
        .unwrap();
        // Make `git remote show origin` fail non-zero regardless of -C usage
        writeln!(
            f,
            "case \"$*\" in *\" remote show origin\"*) exit 1 ;; esac"
        )
        .unwrap();
        // Otherwise exec real git preserving args intact (including -C)
        writeln!(f, "exec {} \"$@\"", real_git).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    let mut p = std::fs::metadata(&shim).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&shim, p).unwrap();

    // Prepend shim to PATH
    let orig_path = std::env::var("PATH").unwrap();
    std::env::set_var("PATH", format!("{}:{}", bin.to_string_lossy(), orig_path));

    let repo = Repository::open(repo_s).unwrap();
    let err = get_remote_head_commit(&repo, repo_s).unwrap_err();
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("git remote show origin failed"));

    // restore PATH
    std::env::set_var("PATH", orig_path);
}

#[cfg(unix)]
fn which_git() -> String {
    let out = Command::new("which").arg("git").output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
