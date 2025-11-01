use mdcode::*;
use tempfile::tempdir;

#[test]
fn test_create_gitignore_writes_file() {
    let tmp = tempdir().unwrap();
    let d = tmp.path();
    create_gitignore(d.to_str().unwrap(), false).unwrap();
    let path = d.join(".gitignore");
    assert!(path.exists());
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("target/"));
}
