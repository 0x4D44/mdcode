// extracted from lib.rs: tests_detect_and_cap
use mdcode::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_detect_file_type_audio_fonts_and_textlike() {
    assert_eq!(detect_file_type(Path::new("x.wav")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("x.MP3")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("x.flac")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("x.ipynb")), Some("Notebook"));
    assert_eq!(detect_file_type(Path::new("x.proto")), Some("Protobuf"));
    assert_eq!(detect_file_type(Path::new("x.gql")), Some("GraphQL"));
    assert_eq!(detect_file_type(Path::new("x.thrift")), Some("Thrift"));
    assert_eq!(detect_file_type(Path::new("x.r")), Some("R"));
    assert_eq!(detect_file_type(Path::new("x.jl")), Some("Julia"));
    assert_eq!(detect_file_type(Path::new("x.mm")), Some("Objective-C++"));
    assert_eq!(detect_file_type(Path::new("x.ttf")), Some("Font"));
    assert_eq!(detect_file_type(Path::new("x.woff2")), Some("Font"));
}

#[test]
fn test_detect_file_type_special_filenames() {
    assert_eq!(detect_file_type(Path::new("LICENSE")), Some("License"));
    assert_eq!(
        detect_file_type(Path::new("Dockerfile")),
        Some("Build Script")
    );
    assert_eq!(
        detect_file_type(Path::new("Makefile")),
        Some("Build Script")
    );
    assert_eq!(detect_file_type(Path::new("CMakeLists.txt")), Some("CMake"));
}

#[test]
fn test_detect_file_type_installer_scripts() {
    assert_eq!(
        detect_file_type(Path::new("setup.iss")),
        Some("Installer Script")
    );
    assert_eq!(
        detect_file_type(Path::new("SETUP.ISS")),
        Some("Installer Script")
    );
}

#[test]
fn test_detect_file_type_lockfiles() {
    assert_eq!(detect_file_type(Path::new("Cargo.lock")), Some("Lockfile"));
    assert_eq!(
        detect_file_type(Path::new("Gemfile.lock")),
        Some("Lockfile")
    );
    assert_eq!(detect_file_type(Path::new("yarn.lock")), Some("Lockfile"));
}

#[test]
fn test_scan_source_files_respects_size_cap() {
    let dir = tempdir().unwrap();
    let d = dir.path();
    // small recognized file
    let mut f_small = File::create(d.join("small.wav")).unwrap();
    f_small.write_all(&vec![0u8; 1024]).unwrap(); // 1 KB

    // large recognized file (~2 MB)
    let mut f_large = File::create(d.join("large.mp3")).unwrap();
    f_large.write_all(&vec![1u8; 2 * 1024 * 1024]).unwrap();

    // cap = 1 MB
    let (files, count) = scan_source_files(d.to_str().unwrap(), 1).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(count, 1);
    assert!(names.contains(&"small.wav".to_string()));
    assert!(!names.contains(&"large.mp3".to_string()));
}

#[test]
fn test_scan_respects_gitignore() {
    let dir = tempdir().unwrap();
    let d = dir.path();
    // Recognized file we will ignore via .gitignore
    std::fs::write(d.join("README.md"), b"Ignored doc").unwrap();
    std::fs::write(d.join(".gitignore"), b"# ignore readme\nREADME.md\n").unwrap();
    // Another recognized file that should remain
    std::fs::write(
        d.join("Cargo.toml"),
        b"[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();

    let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.contains(&"Cargo.toml".to_string()));
    assert!(!names.contains(&"README.md".to_string()));
}

#[test]
fn test_scan_includes_lockfiles() {
    let dir = tempdir().unwrap();
    let d = dir.path();
    std::fs::write(d.join("Cargo.lock"), b"[[package]]").unwrap();
    std::fs::write(d.join("Gemfile.lock"), b"GEM\n").unwrap();

    let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.contains(&"Cargo.lock".to_string()));
    assert!(names.contains(&"Gemfile.lock".to_string()));
}

#[test]
fn test_scan_ignores_target_ci() {
    let dir = tempdir().unwrap();
    let d = dir.path();
    // Simulate Rust CI build artifact under target_ci
    let fp = d.join("target_ci").join("debug").join(".fingerprint");
    std::fs::create_dir_all(&fp).unwrap();
    std::fs::write(fp.join("lib-anyhow.json"), b"{}").unwrap();
    // A legitimate source/config file in the root
    std::fs::write(
        d.join("Cargo.toml"),
        b"[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();

    let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(d).unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n == "Cargo.toml"));
    assert!(
        !names
            .iter()
            .any(|n| n.contains("target_ci") || n.contains(".fingerprint")),
        "should ignore files under target_ci"
    );
}
