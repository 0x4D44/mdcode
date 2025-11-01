use std::path::Path;

pub fn detect_file_type(file_path: &Path) -> Option<&'static str> {
    // Recognize special filenames without extensions.
    if let Some(file_name) = file_path.file_name()?.to_str() {
        if file_name.eq_ignore_ascii_case("LICENSE") {
            return Some("License");
        }
        if file_name.eq_ignore_ascii_case("Dockerfile") {
            return Some("Build Script");
        }
        if file_name.eq_ignore_ascii_case("Makefile") {
            return Some("Build Script");
        }
        if file_name.eq_ignore_ascii_case("CMakeLists.txt") {
            return Some("CMake");
        }
    }

    let extension = file_path.extension()?.to_str()?.to_lowercase();
    match extension.as_str() {
        // Source Code
        "c" => Some("C"),
        "cpp" | "cc" | "cxx" => Some("C++"),
        "h" => Some("C/C++ Header"),
        "hpp" | "hh" | "hxx" => Some("C++ Header"),
        "java" => Some("Java"),
        "py" => Some("Python"),
        "rb" => Some("Ruby"),
        "cs" => Some("C#"),
        "go" => Some("Go"),
        "php" => Some("PHP"),
        "rs" => Some("Rust"),
        "swift" => Some("Swift"),
        "kt" | "kts" => Some("Kotlin"),
        "scala" => Some("Scala"),
        "js" | "jsx" => Some("JavaScript"),
        "ts" | "tsx" => Some("TypeScript"),
        "sh" | "bash" | "zsh" => Some("Shell Script"),
        "bat" => Some("Batch Script"),
        "ps1" => Some("PowerShell"),
        // Additional languages / build systems
        "r" => Some("R"),
        "jl" => Some("Julia"),
        "mm" => Some("Objective-C++"),
        "cmake" => Some("CMake"),
        // APIs / IDL
        "proto" => Some("Protobuf"),
        "graphql" | "gql" => Some("GraphQL"),
        "thrift" => Some("Thrift"),
        // Markup / Documentation
        "html" | "htm" => Some("HTML"),
        "css" | "scss" | "sass" | "less" => Some("CSS"),
        "xml" => Some("XML"),
        "json" => Some("JSON"),
        "yml" | "yaml" => Some("YAML"),
        "toml" => Some("TOML"),
        "lock" => Some("Lockfile"),
        "md" | "txt" | "rst" | "adoc" => Some("Documentation"),
        "ipynb" => Some("Notebook"),
        // Configuration / Build
        "ini" | "cfg" | "conf" => Some("Configuration"),
        "sln" => Some("Solution File"),
        "csproj" => Some("C# Project File"),
        "pom" => Some("Maven Project File"),
        "gradle" => Some("Gradle Build File"),
        // Installer scripts
        "iss" => Some("Installer Script"),
        // Database
        "sql" => Some("SQL"),
        // Images & Assets
        "jpg" | "jpeg" => Some("Image"),
        "png" => Some("Image"),
        "bmp" => Some("Image"),
        "gif" => Some("Image"),
        "tiff" => Some("Image"),
        "webp" => Some("Image"),
        "svg" => Some("Vector Image"),
        "ico" => Some("Icon"),
        "cur" => Some("Cursor"),
        "dlg" => Some("Dialog File"),
        // Audio
        "wav" | "mp3" | "flac" | "aac" | "m4a" | "ogg" | "opus" | "aiff" | "aif" | "wma"
        | "mid" | "midi" => Some("Audio"),
        // Fonts
        "ttf" | "otf" | "woff" | "woff2" => Some("Font"),
        _ => None,
    }
}
