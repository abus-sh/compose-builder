use std::path::PathBuf;

use compose_builder::{FileInfo, Specialisation, collect_files};

#[test]
fn test_collect_files_1() {
    let files = collect_files("tests/compose1").expect("unable to read test directory");

    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0],
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose1/compose.foo.dev.yaml"),
            tag: Some("dev".to_string()),
            specialisation: Some(Specialisation::Development),
        }
    );
}

#[test]
fn test_collect_files_2() {
    let files = collect_files("tests/compose2").expect("unable to read test directory");

    assert_eq!(files.len(), 2);

    let expected_files = vec![
        FileInfo {
            service_name: "bar".to_string(),
            path: PathBuf::from(r"tests/compose2/compose.bar.prod.yaml"),
            tag: Some("prod".to_string()),
            specialisation: Some(Specialisation::Production),
        },
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose2/compose.foo.dev.yaml"),
            tag: Some("dev".to_string()),
            specialisation: Some(Specialisation::Development),
        },
    ];

    // Evil O(n^2) check to make sure all elements of expected_files are in files
    // We already checked that the lengths are equal, so this is fine
    'outer: for expected_file in &expected_files {
        for file in &files {
            if file == expected_file {
                continue 'outer;
            }
        }
        panic!("no matching file found");
    }
}
