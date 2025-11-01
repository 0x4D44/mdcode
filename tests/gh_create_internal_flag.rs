use mdcode::*;
use std::io::Write as _;
use tempfile::tempdir;

// Cover the `--internal` visibility flag path inside execute_cli GhCreate.
#[test]
#[serial_test::serial]
fn test_gh_create_internal_flag_via_execute_cli() {
    // Shim `gh` so gh_cli_path() finds it and invocations succeed.
    let temp = tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let gh_path = bin_dir.join("gh");
    #[cfg(unix)]
    {
        let mut f = std::fs::File::create(&gh_path).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        // Succeed for --version and any other invocation
        writeln!(f, "exit 0").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&gh_path).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&gh_path, p).unwrap();
    }

    // Prepend to PATH
    let orig_path = std::env::var_os("PATH");
    let new_path = format!(
        "{}:{}",
        bin_dir.to_str().unwrap(),
        std::env::var("PATH").unwrap()
    );
    std::env::set_var("PATH", new_path);

    // Create a dummy working directory; we don't need a real git repo for the GhCreate CLI path
    let t = tempdir().unwrap();
    let dir = t.path().join("project_internal");
    std::fs::create_dir_all(&dir).unwrap();
    let dir_str = dir.to_str().unwrap().to_string();

    let cli = Cli {
        command: Commands::GhCreate {
            directory: dir_str,
            description: Some("d".to_string()),
            public: false,
            private: false,
            internal: true, // the path we want to cover
        },
        dry_run: false,
        max_file_mb: 50,
    };
    execute_cli(cli).unwrap();

    // Restore PATH
    if let Some(p) = orig_path {
        std::env::set_var("PATH", p);
    }
}
