// extracted from lib.rs: tests_tag
use mdcode::*;
use std::fs::File;
use std::io::Write as IoWrite;
use tempfile::tempdir;

#[test]
fn test_normalize_semver_tag_variants() {
    let (_, t1) = normalize_semver_tag("1.2.3").unwrap();
    assert_eq!(t1, "v1.2.3");
    let (_, t2) = normalize_semver_tag("v1.2.3").unwrap();
    assert_eq!(t2, "v1.2.3");
    let (_, t3) = normalize_semver_tag("  v2.0.0  ").unwrap();
    assert_eq!(t3, "v2.0.0");
}

#[test]
fn test_read_version_from_cargo_toml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("Cargo.toml");
    let mut f = File::create(&path).unwrap();
    writeln!(
        f,
        "[package]\nname=\"x\"\nversion=\"0.9.1\"\nedition=\"2021\"\n"
    )
    .unwrap();
    let v = read_version_from_cargo_toml(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(v, Some("0.9.1".to_string()));
}

#[test]
fn test_is_dirty_ignores_untracked() {
    if !check_git_installed() {
        eprintln!("git not installed; skipping test");
        return;
    }
    let dir = tempdir().unwrap();
    let d = dir.path();
    // init repo
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("init")
        .status()
        .unwrap();
    // configure local git identity
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("user.name")
        .arg("mdcode-test")
        .status()
        .unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("user.email")
        .arg("mdcode@test.local")
        .status()
        .unwrap();

    // Ensure consistent line ending behavior on Windows to avoid false positives
    // when checking for dirty state in tests.
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("core.autocrlf")
        .arg("false")
        .status()
        .unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("config")
        .arg("core.filemode")
        .arg("false")
        .status()
        .unwrap();

    // create tracked file and commit
    let mut tf = File::create(d.join("tracked.txt")).unwrap();
    writeln!(tf, "hello").unwrap();
    drop(tf); // ensure contents are flushed before adding
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("add")
        .arg("tracked.txt")
        .status()
        .unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(d)
        .arg("commit")
        .arg("-m")
        .arg("init")
        .status()
        .unwrap();
    // create an untracked file
    let mut uf = File::create(d.join("untracked.txt")).unwrap();
    writeln!(uf, "temp").unwrap();
    // is_dirty should be false (ignoring untracked)
    assert!(!is_dirty(d.to_str().unwrap()).unwrap());
    // modify tracked file to make it dirty
    let mut tf2 = File::create(d.join("tracked.txt")).unwrap();
    writeln!(tf2, "more").unwrap();
    drop(tf2);
    assert!(is_dirty(d.to_str().unwrap()).unwrap());
}
