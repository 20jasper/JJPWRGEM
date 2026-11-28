use std::{fmt::Write as _, fs, path::PathBuf};

use jjpwrgem_parse::{SyntaxOptions, ast::Document};
use jjpwrgem_schema::{Schema, validate};
use serde_json::Value;

const DRAFT: u32 = 7;

/// Keywords the validator implements, used only to suggest allowlist additions.
const SUPPORTED_KEYWORDS: [&str; 4] = ["type", "properties", "title", "description"];

/// Allowlisted groups, one `file / group description` per line; every test in each must match `valid`.
const ALLOWLIST: &str = include_str!("json_schema_test_suite.allowlist");

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/JSON-Schema-Test-Suite/validation/tests")
}

/// Whether the schema uses only supported keywords, recursing into `properties`.
fn is_supported(schema: &Value) -> bool {
    match schema {
        Value::Bool(_) => true,
        Value::Object(map) => map.iter().all(|(keyword, value)| {
            if !SUPPORTED_KEYWORDS.contains(&keyword.as_str()) {
                return false;
            }
            match keyword.as_str() {
                "type" | "title" | "description" => value.is_string(),
                "properties" => value
                    .as_object()
                    .is_some_and(|props| props.values().all(is_supported)),
                _ => false,
            }
        }),
        _ => false,
    }
}

/// Implements the suite README's compatibility matching for `DRAFT`.
fn is_compatible(compatibility: Option<&str>) -> bool {
    let Some(compatibility) = compatibility else {
        return true;
    };
    let mut valid = true;
    for constraint in compatibility.split(',') {
        let constraint = constraint.trim();
        let (operator, number) = if let Some(rest) = constraint.strip_prefix("<=") {
            ("<=", rest)
        } else if let Some(rest) = constraint.strip_prefix('=') {
            ("=", rest)
        } else {
            ("", constraint)
        };
        let version: u32 = number.parse().expect("compatibility version is a number");
        valid = match operator {
            "<=" => DRAFT <= version,
            "=" => DRAFT == version,
            _ => DRAFT >= version,
        };
        if DRAFT <= version {
            break;
        }
    }
    valid
}

fn runs_case(case: &Value) -> bool {
    is_compatible(case["compatibility"].as_str())
        && case.get("externalSchemas").is_none()
        && case.get("tags").is_none()
        && is_supported(&case["schema"])
}

fn accepts(schema: &Value, data: &Value) -> Result<bool, String> {
    let schema_doc = Document::parse(schema.to_string(), SyntaxOptions::JSON)
        .map_err(|e| format!("schema text failed to parse: {e:?}"))?;
    let data_doc = Document::parse(data.to_string(), SyntaxOptions::JSON)
        .map_err(|e| format!("data text failed to parse: {e:?}"))?;
    let schema = Schema::parse(&schema_doc).map_err(|e| format!("schema rejected: {e:?}"))?;
    Ok(validate(&data_doc, &schema).is_ok())
}

#[test]
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "reports the suite summary"
)]
fn json_schema_test_suite_draft7() {
    let dir = suite_dir();
    if !dir.is_dir() {
        eprintln!(
            "skipping: {} is missing, run `git submodule update --init`",
            dir.display()
        );
        return;
    }

    let allowlist: Vec<&str> = ALLOWLIST.lines().filter(|l| !l.is_empty()).collect();
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();

    let mut summary = String::new();
    let mut failures = Vec::new();
    let mut suggestions = Vec::new();
    let mut seen = Vec::new();
    let (mut total_allowed, mut total) = (0, 0);

    for path in files {
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let groups: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let (mut allowed, mut file_total) = (0, 0);

        for group in groups["tests"].as_array().unwrap() {
            let entry = format!("{file} / {}", group["description"].as_str().unwrap());
            let tests = group["tests"].as_array().unwrap();
            file_total += tests.len();
            let listed = allowlist.contains(&entry.as_str());
            if listed {
                seen.push(entry.clone());
                allowed += tests.len();
            } else if !(runs_case(group)) {
                continue;
            }
            let mut group_failures = Vec::new();
            for test in tests {
                let expected = test["valid"].as_bool().unwrap();
                let mismatch = match accepts(&group["schema"], &test["data"]) {
                    Ok(actual) if actual == expected => None,
                    Ok(actual) => Some(format!("expected valid={expected}, got valid={actual}")),
                    Err(message) => Some(message),
                };
                if let Some(message) = mismatch {
                    group_failures.push(format!(
                        "{entry} / {}: {message}",
                        test["description"].as_str().unwrap()
                    ));
                }
            }
            if listed {
                failures.extend(group_failures);
            } else if group_failures.is_empty() {
                suggestions.push(entry);
            }
        }

        writeln!(summary, "{file}: allowlisted {allowed}, total {file_total}").unwrap();
        total_allowed += allowed;
        total += file_total;
    }

    for entry in &allowlist {
        if !seen.iter().any(|s| s == entry) {
            failures.push(format!("{entry}: allowlisted group not found in the suite"));
        }
    }

    writeln!(
        summary,
        "overall: allowlisted {total_allowed}, total {total}"
    )
    .unwrap();
    println!("{summary}");
    if !suggestions.is_empty() {
        println!(
            "passing with supported keywords but not allowlisted ({}):\n{}",
            suggestions.len(),
            suggestions.join("\n")
        );
    }

    assert!(
        failures.is_empty(),
        "{} allowlisted mismatches:\n{}\n\n{summary}",
        failures.len(),
        failures.join("\n")
    );
}
