use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_scan_total_files_excludes_common_dirs() {
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // Create files under excluded directories
    let excluded = [
        "target/debug/foo.o",
        "target_ci/debug/.fingerprint/x",
        "bin/a",
        "obj/b",
        "venv/lib.py",
        ".venv/lib.py",
        "env/lib.py",
        ".git/HEAD",
    ];
    for rel in &excluded {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"x").unwrap();
    }
    // And a legitimate source file
    std::fs::write(
        d.join("Cargo.toml"),
        b"[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();
    let total = scan_total_files(d.to_str().unwrap()).unwrap();
    assert_eq!(total, 1);
}

#[test]
fn test_create_gitignore_writes_expected_content() {
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    create_gitignore(d.to_str().unwrap(), false).unwrap();
    let content = std::fs::read_to_string(d.join(".gitignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains(".venv/"));
}

#[test]
fn test_add_files_to_git_counts_files() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping");
        return;
    }
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    std::fs::create_dir_all(d).unwrap();
    std::fs::write(d.join("a.rs"), "fn a(){}\n").unwrap();
    std::fs::write(d.join("b.rs"), "fn b(){}\n").unwrap();
    let repo = git2::Repository::init(d).unwrap();
    let files = vec![d.join("a.rs"), d.join("b.rs")];
    let added = add_files_to_git(d.to_str().unwrap(), &files, false).unwrap();
    assert_eq!(added, 2);
    // ensure index has entries
    let idx = repo.index().unwrap();
    assert_eq!(idx.len(), 2);
}

#[test]
fn test_create_temp_dir_makes_unique_dir() {
    let a = create_temp_dir("mdcode.test").unwrap();
    let b = create_temp_dir("mdcode.test").unwrap();
    assert!(a.exists() && b.exists() && a != b);
}
