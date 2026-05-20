use std::sync::LazyLock;

use bytes2chars::{ErrorKind, Utf8CharIndices};

const UTF8TESTS_PATH: &str = "./tests/conformance/utf8tests/utf8tests.txt";

enum CaseExpectation {
    Valid,
    Invalid,
}

struct Utf8Case {
    id: String,
    expectation: CaseExpectation,
    bytes: Vec<u8>,
}

static CASES: LazyLock<Vec<Utf8Case>> = LazyLock::new(load_cases);

/// Parse hex bytes from a string that may contain pairs without spaces
/// (e.g. "8081 8283" or "C2 A9" or "EFBFBD").
fn parse_hex_bytes(s: &str) -> Vec<u8> {
    let hex: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let s = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(s, 16).unwrap()
        })
        .collect()
}

fn load_cases() -> Vec<Utf8Case> {
    let content = std::fs::read_to_string(UTF8TESTS_PATH).unwrap();
    let mut cases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 3 {
            continue;
        }
        let id = parts[0].trim().to_string();
        // Skip visual tests (group 36.x) — these are about rendering, not decoding
        if id.starts_with("36.") {
            continue;
        }
        let kind = parts[1].trim();
        let hex_field = parts[2].trim();
        let (bytes, expectation) = match kind {
            "valid" => (hex_field.as_bytes().to_vec(), CaseExpectation::Valid),
            "valid hex" => (parse_hex_bytes(hex_field), CaseExpectation::Valid),
            "invalid hex" => (parse_hex_bytes(hex_field), CaseExpectation::Invalid),
            _ => continue,
        };
        cases.push(Utf8Case {
            id,
            expectation,
            bytes,
        });
    }
    cases
}

fn run_conformance_cases(cases: &[&Utf8Case]) {
    assert!(
        !cases.is_empty(),
        "no test cases parsed from {UTF8TESTS_PATH}"
    );

    let mut failures = Vec::new();

    for case in cases {
        let stream = Utf8CharIndices::from(case.bytes.iter().copied());
        let results: Vec<_> = stream.collect();
        let errors: Vec<ErrorKind> = results
            .iter()
            .filter_map(|r| r.as_ref().err().map(|e| e.kind))
            .collect();

        match case.expectation {
            CaseExpectation::Valid => {
                if let Some(&err) = errors.first() {
                    failures.push(format!(
                        "VALID   {} {:02X?} -> unexpected error {:?}",
                        case.id, case.bytes, err
                    ));
                }
            }
            CaseExpectation::Invalid => {
                if errors.is_empty() {
                    failures.push(format!(
                        "INVALID {} {:02X?} -> expected an error but got none",
                        case.id, case.bytes
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} UTF-8 conformance failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// utf8[verify encoding.ascii]
// utf8[verify encoding.two-byte]
// utf8[verify encoding.three-byte]
// utf8[verify encoding.four-byte]
#[test]
fn conformance_valid() {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|c| matches!(c.expectation, CaseExpectation::Valid))
        .collect();
    run_conformance_cases(&cases);
}

// utf8[verify validate.invalid-lead]
// utf8[verify validate.max-sequence-length]
// utf8[verify validate.expected-continuation]
// utf8[verify validate.no-surrogates]
// utf8[verify validate.no-overlong]
// utf8[verify validate.max-codepoint]
// utf8[verify validate.unfinished]
#[test]
fn conformance_invalid() {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|c| matches!(c.expectation, CaseExpectation::Invalid))
        .collect();
    run_conformance_cases(&cases);
}
