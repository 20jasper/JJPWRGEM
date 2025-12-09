use crate::common::cli;
use crate::common::exec_cmd;
use insta::assert_snapshot;
use std::ffi::OsStr;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonResult {
    Fail,
    Pass,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Case {
    text: Vec<u8>,
    file_name: String,
    expected: JsonResult,
}

const CONFORMANCE_PATH: &str = "./tests/conformance/JSONTestSuite/test_parsing";
const FILENAME_FILTER: [&str; 5] = [
    // should expect comma or closed
    "n_array_colon_instead_of_comma.json",
    "n_array_items_separated_by_semicolon.json",
    // uh oh
    "500",
    "10000",
    "n_structure_open_array_object",
];

fn get_tests() -> (Vec<Case>, usize, usize) {
    let entries = fs::read_dir(CONFORMANCE_PATH).unwrap();
    let mut total = 0;
    let mut cases = Vec::new();

    let json_files = entries.filter_map(|entry| {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            return None;
        }

        let path = entry.path();
        if path.extension() != Some(OsStr::new("json")) {
            return None;
        }
        Some(path)
    });

    for path in json_files {
        total += 1;

        let file_name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_ascii_lowercase();
        if FILENAME_FILTER
            .iter()
            .any(|x| file_name.contains(&x.to_ascii_lowercase()))
        {
            continue;
        }

        let expected = match file_name.chars().next().unwrap() {
            'i' => JsonResult::Indeterminate,
            'y' => JsonResult::Pass,
            'n' => JsonResult::Fail,
            _ => continue,
        };
        cases.push(Case {
            text: std::fs::read(&path).expect("should be able to read file"),
            file_name,
            expected,
        });
    }

    let rest = cases.len();
    (cases, total, rest)
}

mod common {}

#[test]
fn feature() {
    let (mut cases, total, rest) = get_tests();
    assert_eq!(rest, 313);
    assert_eq!(total, 318);

    cases.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    for case in cases {
        let output = exec_cmd(cli().arg("format"), Some(case.text));

        assert_snapshot!(case.file_name.clone(), output.snapshot_display());

        assert!(
            case.expected != JsonResult::Fail || !output.status.success(),
            "expected failure: {}",
            &case.file_name,
        );
        assert!(
            case.expected != JsonResult::Pass || output.status.success(),
            "expected success: {}",
            &case.file_name,
        );
    }
}
