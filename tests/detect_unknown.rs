use mdcode::*;
use std::path::Path;

#[test]
fn test_detect_file_type_unknown_extension_returns_none() {
    assert_eq!(detect_file_type(Path::new("file.unknownext")), None);
}
