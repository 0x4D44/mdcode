use mdcode::*;
use std::io::Write as _;
use tempfile::tempdir;

#[test]
fn test_gh_create_conflicting_flags_error_via_execute_cli() {
    // No need for a real gh; error is raised before invoking CLI/API.
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let repo_str = repo_dir.to_str().unwrap().to_string();

    let cli = Cli {
        command: Commands::GhCreate {
            directory: repo_str,
            description: None,
            public: true,
            private: true, // conflicting with public
            internal: false,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    let err = execute_cli(cli).expect_err("conflicting flags should error");
    assert!(err.to_string().contains("Provide only one of"));
}

#[test]
#[serial_test::serial]
fn test_gh_create_resolves_dot_directory_name_and_invokes_cli() {
    // Create a temp PATH shim for `gh` that logs args and returns success.
    let temp = tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let log_path = temp.path().join("gh_args.txt");
    let gh_path = bin_dir.join("gh");

    // Simple POSIX shell script: handle `--version` and log any other args.
    #[cfg(unix)]
    {
        let mut f = std::fs::File::create(&gh_path).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(
            f,
            "if [ \"$1\" = \"--version\" ]; then echo gh version; exit 0; fi"
        )
        .unwrap();
        writeln!(f, "echo \"$@\" > {}", log_path.to_string_lossy()).unwrap();
        writeln!(f, "exit 0").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh_path).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh_path, p).unwrap();
    }

    // Prepend our bin_dir to PATH so `gh_cli_path()` finds it.
    let orig_path = std::env::var_os("PATH");
    let new_path = format!(
        "{}:{}",
        bin_dir.to_str().unwrap(),
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", new_path);

    // Create a working directory with a distinct name; call with directory ".".
    let work = tempdir().unwrap();
    let proj = work.path().join("project_dot_name");
    std::fs::create_dir_all(&proj).unwrap();

    // Change current_dir for this test so "." resolves to `project_dot_name`.
    let orig_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&proj).unwrap();

    let cli = Cli {
        command: Commands::GhCreate {
            directory: ".".to_string(),
            description: Some("desc".to_string()),
            public: false,
            private: false,
            internal: false,
        },
        dry_run: false,
        max_file_mb: 50,
    };
    // This should go down the CLI path and invoke our shim.
    execute_cli(cli).unwrap();

    // Restore environment
    if let Some(p) = orig_path {
        std::env::set_var("PATH", p);
    }
    std::env::set_current_dir(&orig_cwd).unwrap();

    // Verify the first non-`--version` invocation captured the repo name argument.
    let args = std::fs::read_to_string(&log_path).unwrap();
    // Expect pattern: repo create <name> --source . --remote origin --push ...
    let parts: Vec<&str> = args.split_whitespace().collect();
    assert!(parts.len() >= 4, "logged args too short: {}", args);
    assert_eq!(parts[0], "repo");
    assert_eq!(parts[1], "create");
    let name = parts[2];
    assert_eq!(name, "project_dot_name");
}

#[test]
fn test_gh_create_via_cli_error_nonzero_exit() {
    // Create a dummy gh that returns non-zero to trigger error path.
    let temp = tempdir().unwrap();
    let gh = temp.path().join("gh");
    #[cfg(unix)]
    {
        let mut f = std::fs::File::create(&gh).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "exit 2").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh, p).unwrap();
    }
    let err = gh_create_via_cli(
        &gh,
        ".",
        "name",
        Some("d".to_string()),
        RepoVisibility::Private,
    )
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("GitHub CLI 'gh repo create' failed"));
}
