#![cfg(feature = "offline_gh")]
use git2::Repository;
use mdcode::*;
use std::process::Command;
use tempfile::tempdir;

// Force the API fallback path (no `gh` on PATH) and use the test stub
// to return a local file:// clone URL, allowing offline push.
#[test]
fn test_gh_create_api_fallback_offline_pushes_to_local_bare() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }

    let temp = tempdir().unwrap();
    let bare = temp.path().join("remote.git");
    Repository::init_bare(&bare).unwrap();
    let bare_url = format!("file://{}", bare.to_str().unwrap());
    std::env::set_var("MDCODE_TEST_BARE_REMOTE", &bare_url);

    // Create a local repo with one extra commit so push has something
    let work = temp.path().join("work");
    let work_str = work.to_str().unwrap();
    new_repository(work_str, false, 50).unwrap();
    std::fs::write(work.join("x.txt"), "x").unwrap();
    update_repository(work_str, false, Some("x"), 50).unwrap();

    // Prepend a failing `gh` shim to PATH so gh_cli_path() returns None
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let gh = bin.join("gh");
    #[cfg(unix)]
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&gh).unwrap();
        writeln!(f, "#!/bin/sh\nexit 2").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh, p).unwrap();
    }
    let orig_path = std::env::var_os("PATH");
    let new_path = format!(
        "{}:{}",
        bin.to_str().unwrap(),
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", new_path);

    let cli = Cli {
        command: Commands::GhCreate {
            directory: work_str.to_string(),
            description: Some("offline".into()),
            public: false,
            private: false,
            internal: false,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    // Should add origin pointing to our local bare and push successfully
    execute_cli(cli).unwrap();

    // Verify the bare has received master
    let out = Command::new("git")
        .arg("-C")
        .arg(&bare)
        .arg("show-ref")
        .arg("refs/heads/master")
        .output()
        .unwrap();
    assert!(out.status.success());

    // restore env
    if let Some(p) = orig_path {
        std::env::set_var("PATH", p);
    }
    std::env::remove_var("MDCODE_TEST_BARE_REMOTE");
}
