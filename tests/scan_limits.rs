use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_scan_source_files_respects_max_file_mb() {
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    // Create a small and a large file (large has a recognized extension but exceeds threshold)
    std::fs::write(d.join("small.rs"), b"fn main(){}\n").unwrap();
    // Make a ~2.5MB file with a recognized extension
    let big_path = d.join("big.rs");
    let big = vec![0u8; 2_600_000];
    std::fs::write(&big_path, &big).unwrap();
    let (files, total) = scan_source_files(d.to_str().unwrap(), 1).unwrap(); // 1 MB threshold
    assert_eq!(total, 1, "only small.rs should be included below size cap");
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.contains(&"small.rs".to_string()));
    assert!(!names.contains(&"big.bin".to_string()));
}
