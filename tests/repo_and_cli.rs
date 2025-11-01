// extracted from lib.rs: tests
use git2::Repository;
use mdcode::*;
use std::path::Path;

// (removed duplicate import)
use std::fs;
use tempfile::tempdir;

#[test]
fn test_detect_file_type_source_code() {
    // C / C++
    assert_eq!(detect_file_type(Path::new("test.c")), Some("C"));
    assert_eq!(detect_file_type(Path::new("test.cpp")), Some("C++"));
    assert_eq!(detect_file_type(Path::new("test.cc")), Some("C++"));
    assert_eq!(detect_file_type(Path::new("test.cxx")), Some("C++"));
    assert_eq!(detect_file_type(Path::new("test.h")), Some("C/C++ Header"));
    assert_eq!(detect_file_type(Path::new("test.hpp")), Some("C++ Header"));

    // Other languages
    assert_eq!(detect_file_type(Path::new("test.java")), Some("Java"));
    assert_eq!(detect_file_type(Path::new("test.py")), Some("Python"));
    assert_eq!(detect_file_type(Path::new("test.rb")), Some("Ruby"));
    assert_eq!(detect_file_type(Path::new("test.cs")), Some("C#"));
    assert_eq!(detect_file_type(Path::new("test.go")), Some("Go"));
    assert_eq!(detect_file_type(Path::new("test.php")), Some("PHP"));
    assert_eq!(detect_file_type(Path::new("test.rs")), Some("Rust"));
    assert_eq!(detect_file_type(Path::new("test.swift")), Some("Swift"));
    assert_eq!(detect_file_type(Path::new("test.kt")), Some("Kotlin"));
    assert_eq!(detect_file_type(Path::new("test.kts")), Some("Kotlin"));
    assert_eq!(detect_file_type(Path::new("test.scala")), Some("Scala"));
    assert_eq!(detect_file_type(Path::new("test.js")), Some("JavaScript"));
    assert_eq!(detect_file_type(Path::new("test.jsx")), Some("JavaScript"));
    assert_eq!(detect_file_type(Path::new("test.ts")), Some("TypeScript"));
    assert_eq!(detect_file_type(Path::new("test.tsx")), Some("TypeScript"));
    assert_eq!(detect_file_type(Path::new("test.sh")), Some("Shell Script"));
    assert_eq!(
        detect_file_type(Path::new("test.bash")),
        Some("Shell Script")
    );
    assert_eq!(
        detect_file_type(Path::new("test.zsh")),
        Some("Shell Script")
    );
    assert_eq!(
        detect_file_type(Path::new("test.bat")),
        Some("Batch Script")
    );
    assert_eq!(detect_file_type(Path::new("test.ps1")), Some("PowerShell"));
}

#[test]
fn test_detect_file_type_markup_and_config() {
    // Markup and documentation
    assert_eq!(detect_file_type(Path::new("index.html")), Some("HTML"));
    assert_eq!(detect_file_type(Path::new("style.css")), Some("CSS"));
    assert_eq!(detect_file_type(Path::new("script.scss")), Some("CSS"));
    assert_eq!(detect_file_type(Path::new("doc.xml")), Some("XML"));
    assert_eq!(detect_file_type(Path::new("data.json")), Some("JSON"));
    assert_eq!(detect_file_type(Path::new("config.yml")), Some("YAML"));
    assert_eq!(detect_file_type(Path::new("config.yaml")), Some("YAML"));
    assert_eq!(detect_file_type(Path::new("Cargo.toml")), Some("TOML"));
    assert_eq!(
        detect_file_type(Path::new("README.md")),
        Some("Documentation")
    );
    assert_eq!(
        detect_file_type(Path::new("notes.txt")),
        Some("Documentation")
    );
    assert_eq!(
        detect_file_type(Path::new("manual.rst")),
        Some("Documentation")
    );
    assert_eq!(
        detect_file_type(Path::new("guide.adoc")),
        Some("Documentation")
    );

    // Configuration / Build
    assert_eq!(
        detect_file_type(Path::new("settings.ini")),
        Some("Configuration")
    );
    assert_eq!(
        detect_file_type(Path::new("config.cfg")),
        Some("Configuration")
    );
    assert_eq!(
        detect_file_type(Path::new("app.conf")),
        Some("Configuration")
    );
    assert_eq!(
        detect_file_type(Path::new("project.sln")),
        Some("Solution File")
    );
    assert_eq!(
        detect_file_type(Path::new("app.csproj")),
        Some("C# Project File")
    );
    assert_eq!(detect_file_type(Path::new("pom.xml")), Some("XML")); // Note: Maven's pom.xml is XML
    assert_eq!(
        detect_file_type(Path::new("build.gradle")),
        Some("Gradle Build File")
    );

    // Database
    assert_eq!(detect_file_type(Path::new("schema.sql")), Some("SQL"));
}

#[test]
fn test_detect_file_type_images_and_assets() {
    // Raster images
    assert_eq!(detect_file_type(Path::new("image.jpg")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.jpeg")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.png")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.bmp")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.gif")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.tiff")), Some("Image"));
    assert_eq!(detect_file_type(Path::new("image.webp")), Some("Image"));
    // Vector and icons
    assert_eq!(
        detect_file_type(Path::new("vector.svg")),
        Some("Vector Image")
    );
    assert_eq!(detect_file_type(Path::new("icon.ico")), Some("Icon"));
    assert_eq!(detect_file_type(Path::new("cursor.cur")), Some("Cursor"));
    // Other asset
    assert_eq!(
        detect_file_type(Path::new("dialog.dlg")),
        Some("Dialog File")
    );
}

#[test]
fn test_generate_gitignore_content() {
    let content = generate_gitignore_content(".").unwrap();
    let expected = "target/\ntarget_ci/\nbin/\nobj/\nvenv/\n.venv/\nenv/\n*.tmp\n*.log";
    assert_eq!(content, expected);
}

#[test]
fn test_new_repository_and_gitignore() {
    if !check_git_installed() {
        eprintln!("Skipping test: Git not installed");
        return;
    }
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    assert!(
        Path::new(repo_str).join(".git").exists(),
        ".git directory should exist"
    );
    assert!(
        Path::new(repo_str).join(".gitignore").exists(),
        ".gitignore file should exist"
    );
}

#[test]
fn test_update_repository() {
    if !check_git_installed() {
        eprintln!("Skipping test: Git not installed");
        return;
    }
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let file_path = repo_path.join("new_file.txt");
    fs::write(&file_path, "Hello, mdcode!").unwrap();
    // Provide a commit message to avoid hanging.
    update_repository(repo_str, false, Some("Test commit message"), 50).unwrap();
    let repo = Repository::open(repo_str).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let commits: Vec<_> = revwalk.collect();
    assert!(
        commits.len() >= 2,
        "Repository should have at least two commits"
    );
}

#[test]
fn test_info_repository() {
    if !check_git_installed() {
        eprintln!("Skipping test: Git not installed");
        return;
    }
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().unwrap();
    new_repository(repo_str, false, 50).unwrap();
    let file_path = repo_path.join("info_test.txt");
    fs::write(&file_path, "Test info output").unwrap();
    update_repository(repo_str, false, Some("Test commit message"), 50).unwrap();
    info_repository(repo_str).unwrap();
}
