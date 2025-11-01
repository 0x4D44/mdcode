use mdcode::*;
use std::path::Path;

#[test]
fn test_detect_file_type_more_extensions() {
    // Headers
    assert_eq!(detect_file_type(Path::new("test.hh")), Some("C++ Header"));
    assert_eq!(detect_file_type(Path::new("test.hxx")), Some("C++ Header"));
    // Less common languages/build files
    assert_eq!(detect_file_type(Path::new("script.r")), Some("R"));
    assert_eq!(detect_file_type(Path::new("model.jl")), Some("Julia"));
    assert_eq!(
        detect_file_type(Path::new("objc.mm")),
        Some("Objective-C++")
    );
    assert_eq!(detect_file_type(Path::new("rules.cmake")), Some("CMake"));
    // API/IDL
    assert_eq!(
        detect_file_type(Path::new("schema.proto")),
        Some("Protobuf")
    );
    assert_eq!(
        detect_file_type(Path::new("query.graphql")),
        Some("GraphQL")
    );
    assert_eq!(detect_file_type(Path::new("query.gql")), Some("GraphQL"));
    assert_eq!(
        detect_file_type(Path::new("service.thrift")),
        Some("Thrift")
    );
    // Markup variants
    assert_eq!(detect_file_type(Path::new("index.htm")), Some("HTML"));
    assert_eq!(detect_file_type(Path::new("styles.sass")), Some("CSS"));
    assert_eq!(detect_file_type(Path::new("vars.less")), Some("CSS"));
    assert_eq!(
        detect_file_type(Path::new("notebook.ipynb")),
        Some("Notebook")
    );
    assert_eq!(detect_file_type(Path::new("Cargo.lock")), Some("Lockfile"));
    // Installer
    assert_eq!(
        detect_file_type(Path::new("setup.iss")),
        Some("Installer Script")
    );
    // Fonts
    assert_eq!(detect_file_type(Path::new("font.ttf")), Some("Font"));
    assert_eq!(detect_file_type(Path::new("font.otf")), Some("Font"));
    assert_eq!(detect_file_type(Path::new("font.woff")), Some("Font"));
    assert_eq!(detect_file_type(Path::new("font.woff2")), Some("Font"));
    // Audio
    assert_eq!(detect_file_type(Path::new("tone.wav")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("music.mp3")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("lossless.flac")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("track.aac")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("song.m4a")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("alt.ogg")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("voice.opus")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("sample.aiff")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("clip.aif")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("winmedia.wma")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("midi.mid")), Some("Audio"));
    assert_eq!(detect_file_type(Path::new("midi2.midi")), Some("Audio"));
}
