use mdcode::*;
use std::path::Path;

#[test]
fn test_detect_file_type_special_names() {
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
