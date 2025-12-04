use std::ffi::OsStr;
use std::fs;

use annotate_snippets::{Renderer, renderer::DecorStyle};
use cli::run;
use insta::assert_snapshot;

use crate::common::format_output_snapshot;

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
const FILENAME_FILTER: [&str; 7] = [
    // should expect comma or closed
    "n_array_colon_instead_of_comma.json",
    "n_array_items_separated_by_semicolon.json",
    // uh oh
    "500",
    "10000",
    // multi key is not consistent
    "y_object.json",
    "y_object_extreme_numbers",
    "y_object_long_strings",
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

#[test]
fn feature() {
    let (mut cases, total, rest) = get_tests();
    assert_eq!(rest, 309);
    assert_eq!(total, 318);

    cases.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let renderer = Renderer::plain().decor_style(DecorStyle::Ascii);
    for case in cases {
        let annotated = run(case.text.clone(), &renderer);

        assert_snapshot!(
            case.file_name,
            format_output_snapshot(case.text, &annotated)
        );
        assert!(case.expected != JsonResult::Fail || annotated.stderr.is_some());
        assert!(case.expected != JsonResult::Pass || annotated.stdout.is_some());
    }
}
