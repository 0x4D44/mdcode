use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_scan_total_files_excludes_common_dirs() {
    let t = tempdir().unwrap();
    let d = t.path();
    // included files
    std::fs::create_dir_all(d.join("src")).unwrap();
    std::fs::write(d.join("src/lib.rs"), "mod x;\n").unwrap();
    std::fs::write(d.join("README.md"), "x\n").unwrap();
    // excluded directories and files
    for excl in ["target", "bin", "obj", "venv", ".venv", "env", ".git"] {
        std::fs::create_dir_all(d.join(excl)).unwrap();
        std::fs::write(d.join(excl).join("ignored.txt"), "x\n").unwrap();
    }
    let n = scan_total_files(d.to_str().unwrap()).unwrap();
    // Only the two included files should be counted
    assert_eq!(n, 2);
}
