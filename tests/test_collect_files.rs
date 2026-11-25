use std::{fmt::Debug, path::PathBuf};

use compose_builder::{FileInfo, Specialisation, collect_files};

// Given two arrays (where each array has no repeated elements), check if the arrays contain the
// same elements, regardless of order.
fn compare_lists<T: PartialEq + Debug>(a: &[T], b: &[T]) -> bool {
    // Arrays with a different number of elements can't be equal
    if a.len() != b.len() {
        return false;
    }

    // If all elements of a are in b, then the arrays are equal
    a.iter().all(|e| b.contains(e))
}

#[test]
fn test_collect_files_1() {
    let files = collect_files("tests/compose1").expect("unable to read compose2 test directory");

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
    let files = collect_files("tests/compose2").expect("unable to read compose2 test directory");

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

    assert!(
        compare_lists(&files, &expected_files),
        "not all files collected"
    );
}
