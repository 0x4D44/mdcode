use mdcode::*;
use std::io::Write as _;
use tempfile::tempdir;

#[test]
fn test_read_version_from_cargo_toml_missing_returns_none() {
    let dir = tempdir().unwrap();
    let v = read_version_from_cargo_toml(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(v, None);
}

#[test]
fn test_read_version_from_cargo_toml_package_without_version_returns_none() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), b"[package]\nname='x'\n").unwrap();
    let v = read_version_from_cargo_toml(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(v, None);
}

#[test]
fn test_read_version_from_cargo_toml_malformed_errors() {
    let dir = tempdir().unwrap();
    // Write invalid TOML that cannot parse
    let mut f = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
    writeln!(f, "[package\nname='x' version='0.1.0'").unwrap(); // missing closing bracket & quotes mismatch
    let err = read_version_from_cargo_toml(dir.path().to_str().unwrap()).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("parse"));
}
