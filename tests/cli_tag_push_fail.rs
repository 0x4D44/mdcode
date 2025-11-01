use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[cfg(unix)]
fn test_execute_cli_tag_push_failure_with_shim() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let repo = tmp.path().join("r");
    let s = repo.to_str().unwrap().to_string();
    new_repository(&s, false, 50).unwrap();
    // Set up origin
    let bare = tmp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(bare.to_str().unwrap())
        .status()
        .unwrap();

    // PATH shim that fails on any argument equal to 'push'
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let shim = bin.join("git");
    let real = which_git();
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&shim).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(
            f,
            "if [ \"$1\" = \"--version\" ]; then exec {} \"$@\"; fi",
            real
        )
        .unwrap();
        writeln!(
            f,
            "for a in \"$@\"; do if [ \"$a\" = \"push\" ]; then exit 1; fi; done"
        )
        .unwrap();
        writeln!(f, "exec {} \"$@\"", real).unwrap();
    }
    use std::os::unix::fs::PermissionsExt;
    let mut p = std::fs::metadata(&shim).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&shim, p).unwrap();
    let orig_path = std::env::var("PATH").unwrap();
    std::env::set_var("PATH", format!("{}:{}", bin.to_string_lossy(), orig_path));

    let cli = Cli {
        command: Commands::Tag {
            directory: s.clone(),
            version: Some("3.4.5".into()),
            message: None,
            no_push: false,
            remote: "origin".into(),
            force: false,
            allow_dirty: true,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    let err = execute_cli(cli).unwrap_err();
    assert!(err.to_string().contains("failed to push tag"));
    std::env::set_var("PATH", orig_path);
}

#[cfg(unix)]
fn which_git() -> String {
    let out = Command::new("which").arg("git").output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
