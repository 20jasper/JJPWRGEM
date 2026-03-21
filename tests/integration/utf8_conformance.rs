use bytes2chars::{ErrorKind, Utf8CharIndices};

const UTF8TESTS_PATH: &str = "./tests/conformance/utf8tests/utf8tests.txt";

struct Utf8Case {
    id: String,
    valid: bool,
    bytes: Vec<u8>,
}

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

fn get_cases() -> Vec<Utf8Case> {
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
        let (valid, bytes) = match kind {
            "valid" => (true, hex_field.as_bytes().to_vec()),
            "valid hex" => (true, parse_hex_bytes(hex_field)),
            "invalid hex" => (false, parse_hex_bytes(hex_field)),
            _ => continue,
        };
        cases.push(Utf8Case { id, valid, bytes });
    }
    cases
}

/// Predicate over error kinds for invalid cases where we can assert specifically.
/// Returns `None` for cases where any error is acceptable.
fn expected_kind(id: &str) -> Option<fn(ErrorKind) -> bool> {
    // Surrogate code points (U+D800..=U+DFFF) — codepoint varies, use wildcard
    if id.starts_with("24.") || id.starts_with("25.") {
        return Some(|k| matches!(k, ErrorKind::InvalidSurrogate(_)));
    }
    // Sequences truncated at end-of-stream
    if matches!(id, "18.1" | "18.2" | "19.0" | "19.1" | "19.5") {
        return Some(|k| k == ErrorKind::UnfinishedSequence);
    }
    None
}

#[test]
fn utf8_conformance() {
    let cases = get_cases();
    assert!(
        !cases.is_empty(),
        "no test cases parsed from {UTF8TESTS_PATH}"
    );

    let mut failures = Vec::new();

    for case in &cases {
        let stream = Utf8CharIndices::from(case.bytes.iter().copied());
        let results: Vec<_> = stream.collect();

        let errors: Vec<ErrorKind> = results
            .iter()
            .filter_map(|r| r.as_ref().err().map(|e| e.kind))
            .collect();

        if case.valid {
            if let Some(&err) = errors.first() {
                failures.push(format!(
                    "VALID   {} {:02X?} -> unexpected error {:?}",
                    case.id, case.bytes, err
                ));
            }
        } else {
            match expected_kind(&case.id) {
                Some(pred) => {
                    if !errors.iter().copied().any(pred) {
                        failures.push(format!(
                            "INVALID {} {:02X?} -> no matching error kind, got {errors:?}",
                            case.id, case.bytes
                        ));
                    }
                }
                None => {
                    if errors.is_empty() {
                        failures.push(format!(
                            "INVALID {} {:02X?} -> expected an error but got none",
                            case.id, case.bytes
                        ));
                    }
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} UTF-8 conformance failure(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
