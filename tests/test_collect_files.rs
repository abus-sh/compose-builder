use std::{fmt::Debug, path::PathBuf};

use compose_builder::{FileInfo, Specialisation, collect_files};

// Given two arrays (where each array has no repeated elements), check if the arrays contain the
// same elements, regardless of order.
fn assert_compare_lists<T: PartialEq + Debug>(given: &[T], expected: &[T]) {
    // Arrays with a different number of elements can't be equal
    assert_eq!(given.len(), expected.len(), "mismatch in len");

    // Require all elements of a to be in b
    for elem_a in given {
        assert!(
            expected.contains(elem_a),
            "extra element in given array: {:?}",
            elem_a
        );
    }

    // Require all elements of b to be in a
    for elem_b in expected {
        assert!(
            given.contains(elem_b),
            "missing element in expected array: {:?}",
            elem_b
        );
    }
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

    assert_compare_lists(&files, &expected_files);
}

#[test]
fn test_collect_files_3() {
    let files = collect_files("tests/compose3").expect("unable to read compose3 test directory");

    assert_eq!(files.len(), 4);

    let expected_files = vec![
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose3/compose.foo.yaml"),
            tag: None,
            specialisation: None,
        },
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose3/compose.foo.dev.yaml"),
            tag: Some("dev".to_string()),
            specialisation: Some(Specialisation::Development),
        },
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose3/compose.foo.prod.yaml"),
            tag: Some("prod".to_string()),
            specialisation: Some(Specialisation::Production),
        },
        FileInfo {
            service_name: "foo".to_string(),
            path: PathBuf::from(r"tests/compose3/compose.foo.secret.yaml"),
            tag: Some("secret".to_string()),
            specialisation: Some(Specialisation::Production),
        },
    ];

    assert_compare_lists(&files, &expected_files)
}
