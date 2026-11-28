use std::{fs, path::PathBuf};

use jjpwrgem_parse::{SyntaxOptions, ast::Document};
use jjpwrgem_schema::{Schema, validate};
use serde_json::Value;

/// Allowlisted draft 07 `(file, group description)` pairs; every test in each must match `valid`.
const ALLOWLIST: &[(&str, &str)] = &[
    (
        "additionalProperties.json",
        "additionalProperties are allowed by default",
    ),
    ("boolean_schema.json", "boolean schema 'false'"),
    ("boolean_schema.json", "boolean schema 'true'"),
    // r[verify json-schema-v7.array.items-schema]
    ("items.json", "a schema given for items"),
    (
        "items.json",
        "additional items are allowed by default (2019)",
    ),
    // r[verify json-schema-v7.array.items-tuple]
    ("items.json", "an array of schemas for items"),
    ("items.json", "array-form items with null instance elements"),
    ("items.json", "items as array with boolean schemas"),
    ("items.json", "items with boolean schema (false)"),
    ("items.json", "items with boolean schema (true)"),
    ("items.json", "items with null instance elements"),
    ("items.json", "nested items"),
    ("properties.json", "object properties validation"),
    (
        "properties.json",
        "properties whose names are Javascript object property names",
    ),
    ("properties.json", "properties with boolean schema"),
    ("properties.json", "properties with escaped characters"),
    (
        "properties.json",
        "properties with null valued instance properties",
    ),
    ("ref.json", "property named $ref that is not a reference"),
    ("required.json", "required default validation"),
    ("type.json", "array type matches arrays"),
    ("type.json", "boolean type matches booleans"),
    ("type.json", "integer type matches integers"),
    ("type.json", "integer type matches integers (6)"),
    ("type.json", "null type matches only the null object"),
    ("type.json", "number type matches numbers"),
    ("type.json", "object type matches objects"),
    ("type.json", "string type matches strings"),
];

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/JSON-Schema-Test-Suite/validation/tests")
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

    let mut failures = Vec::new();
    let mut seen = Vec::new();
    let (mut allowed, mut total) = (0, 0);

    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let groups: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

        for group in groups["tests"].as_array().unwrap() {
            let description = group["description"].as_str().unwrap();
            let tests = group["tests"].as_array().unwrap();
            total += tests.len();
            let Some(listed) = ALLOWLIST
                .iter()
                .find(|&&(f, d)| f == file && d == description)
            else {
                continue;
            };
            seen.push(listed);
            allowed += tests.len();
            for test in tests {
                let expected = test["valid"].as_bool().unwrap();
                let mismatch = match accepts(&group["schema"], &test["data"]) {
                    Ok(actual) if actual == expected => None,
                    Ok(actual) => Some(format!("expected valid={expected}, got valid={actual}")),
                    Err(message) => Some(message),
                };
                if let Some(message) = mismatch {
                    failures.push(format!(
                        "{file} / {description} / {}: {message}",
                        test["description"].as_str().unwrap()
                    ));
                }
            }
        }
    }

    for listed in ALLOWLIST {
        if !seen.contains(&listed) {
            let (file, description) = listed;
            failures.push(format!(
                "{file} / {description}: allowlisted group not found in the suite"
            ));
        }
    }

    let summary = format!("allowlisted {allowed} of {total} tests");
    println!("{summary}");
    assert!(
        failures.is_empty(),
        "{} allowlisted mismatches:\n{}\n\n{summary}",
        failures.len(),
        failures.join("\n")
    );
}
