//! check compatibility with prettier, a JavaScript formatter

use std::path::Path;

use proptest::{prelude::*, prop_assert, prop_assert_eq, prop_assume, prop_oneof, proptest};

use crate::{
    common::{cli, exec_cmd, format_template},
    test_json::*,
};

const PRINT_WIDTH: usize = 80;

fn is_known_prettier_difference_case(name: &str) -> bool {
    matches!(
        name,
        "ARRAY_WITH_NESTED_OBJECTS"
            | "MIXED_ARRAY_WITH_LONG_STRINGS"
            | "PRETTIER_PASS1"
            | "ARRAY_BLANK_LINES_AND_EXTRA_WHITESPACE"
            | "VALID_TRAILING_ZEROS_FRACTION"
            | "ARRAY_THREE_NEWLINES_BETWEEN_ITEMS"
            | "OBJECT_BLANK_LINES_BETWEEN_KEYS"
            | "OBJECT_LEADING_NEWLINE"
            | "EXPONENT_LEADING_ZEROS"
            | "ARRAY_NUMERIC_MATRIX_SHORT"
    )
}

fn prettier_cmd() -> std::process::Command {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("node_modules/.bin/prettier");
    assert!(
        bin.exists(),
        "prettier not found at {bin:?} — run `pnpm install`"
    );
    let mut cmd = std::process::Command::new(bin);
    cmd.args([
        "--parser",
        "json",
        "--print-width",
        &PRINT_WIDTH.to_string(),
        "--stdin-filepath",
        "input.json",
    ]);
    // Ensure node is discoverable when managed by mise (shims may not be in PATH during tests)
    let current_path = std::env::var("PATH").unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();
    let mise_shims = format!("{home}/.local/share/mise/shims");
    if !current_path.contains(&mise_shims) {
        cmd.env("PATH", format!("{mise_shims}:{current_path}"));
    }
    cmd
}

#[rstest_reuse::apply(format_template)]
fn matches_prettier(#[case] (name, input): (&str, &str)) {
    let mut prettier = prettier_cmd();

    let prettier_out = exec_cmd(&mut prettier, Some(input.as_bytes().to_vec()));
    assert!(
        prettier_out.status.success(),
        "prettier failed on {name}:\n{}",
        prettier_out.stderr
    );

    let mut jjp = cli();
    jjp.args(["format", "--preferred-width", &PRINT_WIDTH.to_string()]);
    let jjp_out = exec_cmd(&mut jjp, Some(input.as_bytes().to_vec()));
    assert!(
        jjp_out.status.success(),
        "jjp failed on {name}:\n{}",
        jjp_out.stderr
    );

    if is_known_prettier_difference_case(name) {
        assert_ne!(
            prettier_out.stdout, jjp_out.stdout,
            "{name} is in is_known_prettier_difference_case but jjp now matches prettier — remove it from the list"
        );
    } else {
        assert_eq!(
            prettier_out.stdout, jjp_out.stdout,
            "output mismatch for {name}"
        );
    }
}

// ── Proptest strategies ────────────────────────────────────────────────────────

/// Safe whitespace: at most one `\n` per gap (no blank lines)
fn arb_safe_ws() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![Just(" "), Just("\t"), Just("\n")], 0..=3_usize).prop_map(
        |chars| {
            let s = chars.concat();
            let mut out = String::new();
            let mut saw_nl = false;
            for c in s.chars() {
                if c == '\n' {
                    if !saw_nl {
                        out.push(c);
                        saw_nl = true;
                    }
                } else {
                    out.push(c);
                    saw_nl = false;
                }
            }
            out
        },
    )
}

/// Whitespace that may include blank lines (for the `#[ignore]` blank-line test)
fn arb_ws_with_blanks() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![Just(" "), Just("\t"), Just("\n")], 0..=4_usize)
        .prop_map(|chars| chars.concat())
}

/// Safe scalars: null/true/false/small integers only (no floats/exponents)
fn arb_safe_scalar() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("null".to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
        (-99i32..=99i32).prop_map(|n| n.to_string()),
    ]
}

/// Zero-exponent scalars — both prettier and jjp strip these to the mantissa
fn arb_zero_exponent_scalar() -> impl Strategy<Value = String> {
    prop_oneof![
        (1i32..=99i32).prop_map(|m| format!("{m}e0")),
        (1i32..=99i32).prop_map(|m| format!("{m}e00")),
        (1i32..=99i32).prop_map(|m| format!("{m}e+00")),
        (1i32..=99i32).prop_map(|m| format!("{m}e-00")),
    ]
}

/// Float/exponent scalars (for the `#[ignore]` number normalization test)
fn arb_float_scalar() -> impl Strategy<Value = String> {
    prop_oneof![
        (1i32..=99i32, 1u32..=9u32).prop_map(|(m, d)| format!("{m}.{d}")),
        (1i32..=99i32, 1i32..=9i32).prop_map(|(m, e)| format!("{m}e{e}")),
        (1i32..=99i32, 1i32..=9i32).prop_map(|(m, e)| format!("{m}e+{e}")),
        (1i32..=99i32, 1i32..=9i32).prop_map(|(m, e)| format!("{m}E{e}")),
        // additional exponent forms where jjp and prettier normalization differs
        (1i32..=99i32, 1i32..=9i32).prop_map(|(m, e)| format!("{m}E+{e}")),
        (1i32..=99i32, 1u32..=3u32, 1i32..=9i32)
            .prop_map(|(m, zeros, e)| format!("{m}e{}{e}", "0".repeat(zeros as usize))),
        (1i32..=99i32, 1u32..=9u32, 1u32..=4u32)
            .prop_map(|(m, d, zeros)| format!("{m}.{d}{}", "0".repeat(zeros as usize))),
    ]
}

/// Simple ASCII string value (no special escaping needed)
fn arb_safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,30}".prop_map(|s| format!("\"{s}\""))
}

/// Array of 0–4 safe short scalars. Kept small so total < 80 chars — both formatters inline.
fn arb_safe_array() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_safe_scalar(), 0..=4_usize).prop_flat_map(|items| {
        let gaps_len = items.len().saturating_sub(1).max(1);
        (
            Just(items),
            arb_safe_ws(),
            arb_safe_ws(),
            prop::collection::vec(arb_safe_ws(), gaps_len..=gaps_len),
        )
            .prop_map(|(items, open_ws, close_ws, gaps)| {
                if items.is_empty() {
                    return format!("[{open_ws}]");
                }
                let mut s = format!("[{open_ws}");
                for (i, item) in items.iter().enumerate() {
                    s.push_str(item);
                    if i + 1 < items.len() {
                        s.push(',');
                        s.push_str(&gaps[i]);
                    }
                }
                s.push_str(&close_ws);
                s.push(']');
                s
            })
    })
}

/// Array of 0–4 items mixing scalars and short strings. Total < 80 chars — both formatters inline.
fn arb_safe_array_with_strings() -> impl Strategy<Value = String> {
    let arb_item = prop_oneof![
        arb_safe_scalar(),
        "[a-zA-Z0-9]{0,10}".prop_map(|s| format!("\"{s}\"")),
    ];
    prop::collection::vec(arb_item, 0..=4_usize).prop_flat_map(|items| {
        let gaps_len = items.len().saturating_sub(1).max(1);
        (
            Just(items),
            arb_safe_ws(),
            arb_safe_ws(),
            prop::collection::vec(arb_safe_ws(), gaps_len..=gaps_len),
        )
            .prop_map(|(items, open_ws, close_ws, gaps)| {
                if items.is_empty() {
                    return format!("[{open_ws}]");
                }
                let mut s = format!("[{open_ws}");
                for (i, item) in items.iter().enumerate() {
                    s.push_str(item);
                    if i + 1 < items.len() {
                        s.push(',');
                        s.push_str(&gaps[i]);
                    }
                }
                s.push_str(&close_ws);
                s.push(']');
                s
            })
    })
}

/// Array of 1–5 small objects. Prettier expands each object; jjp collapses short ones.
fn arb_array_of_objects() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_object(), 1..=5usize).prop_map(|items| {
        let mut s = "[".to_string();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(item);
        }
        s.push(']');
        s
    })
}

/// Object of 0–5 entries with safe whitespace
fn arb_object() -> impl Strategy<Value = String> {
    prop::collection::vec(
        ("[a-z]{1,8}".prop_map(|k| k), arb_safe_scalar()),
        0..=5usize,
    )
    .prop_flat_map(|entries| {
        let n = entries.len();
        let gaps_len = n.saturating_sub(1).max(1);
        let colon_ws_len = n.max(1);
        (
            Just(entries),
            arb_safe_ws(),
            arb_safe_ws(),
            prop::collection::vec(arb_safe_ws(), gaps_len..=gaps_len),
            prop::collection::vec(arb_safe_ws(), colon_ws_len..=colon_ws_len),
        )
            .prop_map(|(entries, open_ws, close_ws, gaps, colon_ws)| {
                if entries.is_empty() {
                    return format!("{{{open_ws}}}");
                }
                let mut s = format!("{{{open_ws}");
                for (i, (key, val)) in entries.iter().enumerate() {
                    s.push('"');
                    s.push_str(key);
                    s.push('"');
                    s.push(':');
                    s.push_str(&colon_ws[i]);
                    s.push_str(val);
                    if i + 1 < entries.len() {
                        s.push(',');
                        s.push_str(&gaps[i]);
                    }
                }
                s.push_str(&close_ws);
                s.push('}');
                s
            })
    })
}

/// Whitespace with spaces/tabs only — no newlines, so prettier doesn't treat the array as
/// "already expanded" (which would cause outer arrays to expand even when they fit on one line)
fn arb_ws_no_newlines() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![Just(" "), Just("\t")], 0..=3_usize)
        .prop_map(|chars| chars.concat())
}

/// Inner array with no-newline whitespace, used inside nested arrays
fn arb_inner_array() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_safe_scalar(), 0..=4_usize).prop_flat_map(|items| {
        let gaps_len = items.len().saturating_sub(1).max(1);
        (
            Just(items),
            arb_ws_no_newlines(),
            arb_ws_no_newlines(),
            prop::collection::vec(arb_ws_no_newlines(), gaps_len..=gaps_len),
        )
            .prop_map(|(items, open_ws, close_ws, gaps)| {
                if items.is_empty() {
                    return format!("[{open_ws}]");
                }
                let mut s = format!("[{open_ws}");
                for (i, item) in items.iter().enumerate() {
                    s.push_str(item);
                    if i + 1 < items.len() {
                        s.push(',');
                        s.push_str(&gaps[i]);
                    }
                }
                s.push_str(&close_ws);
                s.push(']');
                s
            })
    })
}

/// Array of 0–3 inner arrays (no newlines in inner arrays).
/// When the outer array must expand, each inner array stays inline in both formatters.
fn arb_nested_array() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_inner_array(), 0..=3_usize).prop_flat_map(|items| {
        let gaps_len = items.len().saturating_sub(1).max(1);
        (
            Just(items),
            arb_safe_ws(),
            arb_safe_ws(),
            prop::collection::vec(arb_safe_ws(), gaps_len..=gaps_len),
        )
            .prop_map(|(items, open_ws, close_ws, gaps)| {
                if items.is_empty() {
                    return format!("[{open_ws}]");
                }
                let mut s = format!("[{open_ws}");
                for (i, item) in items.iter().enumerate() {
                    s.push_str(item);
                    if i + 1 < items.len() {
                        s.push(',');
                        s.push_str(&gaps[i]);
                    }
                }
                s.push_str(&close_ws);
                s.push(']');
                s
            })
    })
}

/// Array of 20–40 short integers whose inline form exceeds print width, triggering
/// prettier's numeric-array fill mode.
fn arb_fill_mode_array() -> impl Strategy<Value = String> {
    prop::collection::vec((-99i32..=99i32).prop_map(|n| n.to_string()), 20..=40usize)
        .prop_filter("array must exceed print width", |items| {
            2 + items.iter().map(String::len).sum::<usize>() + items.len().saturating_sub(1) * 2
                > PRINT_WIDTH
        })
        .prop_map(|items| {
            let mut s = "[".to_string();
            for (i, item) in items.iter().enumerate() {
                s.push_str(item);
                if i + 1 < items.len() {
                    s.push_str(", ");
                }
            }
            s.push(']');
            s
        })
}

/// Any safe value: scalar or long ASCII string (1–150 chars)
fn arb_any_safe_value() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_safe_scalar(),
        "[a-zA-Z0-9 ]{1,150}".prop_map(|s| format!("\"{s}\"")),
    ]
}

/// Large object: 1–150 entries, keys 1–150 chars, any safe value
fn arb_large_object() -> impl Strategy<Value = String> {
    prop::collection::vec(("[a-z]{1,150}", arb_any_safe_value()), 1..=150usize).prop_map(
        |entries| {
            let mut s = "{".to_string();
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push('"');
                s.push_str(key);
                s.push('"');
                s.push(':');
                s.push_str(val);
            }
            s.push('}');
            s
        },
    )
}

/// Object with array values: 1–20 entries, each value is an array of 0–10 safe values
fn arb_object_with_array_values() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (
            "[a-z]{1,50}",
            prop::collection::vec(arb_any_safe_value(), 0..=10usize).prop_map(|items| {
                let inner = items.join(",");
                format!("[{inner}]")
            }),
        ),
        1..=20usize,
    )
    .prop_map(|entries| {
        let mut s = "{".to_string();
        for (i, (key, val)) in entries.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('"');
            s.push_str(key);
            s.push('"');
            s.push(':');
            s.push_str(val);
        }
        s.push('}');
        s
    })
}

fn run_prop_comparison(input: &str) -> Result<(), proptest::test_runner::TestCaseError> {
    let prettier_out = exec_cmd(&mut prettier_cmd(), Some(input.as_bytes().to_vec()));
    prop_assume!(prettier_out.status.success());

    let mut jjp = cli();
    jjp.args(["format", "--preferred-width", &PRINT_WIDTH.to_string()]);
    let jjp_out = exec_cmd(&mut jjp, Some(input.as_bytes().to_vec()));
    prop_assert!(jjp_out.status.success(), "jjp failed:\n{}", jjp_out.stderr);
    prop_assert_eq!(&prettier_out.stdout, &jjp_out.stdout, "input: {:?}", input);
    Ok(())
}

// ── Passing property tests ─────────────────────────────────────────────────────

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(16))]

    #[test]
    fn prop_scalar_matches_prettier(
        pre in arb_safe_ws(), val in arb_safe_scalar(), post in arb_safe_ws()
    ) {
        let input = format!("{pre}{val}{post}");
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_string_scalar_matches_prettier(
        pre in arb_safe_ws(), val in arb_safe_string(), post in arb_safe_ws()
    ) {
        let input = format!("{pre}{val}{post}");
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_array_matches_prettier(input in arb_safe_array()) {
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_array_with_strings_matches_prettier(input in arb_safe_array_with_strings()) {
        run_prop_comparison(&input)?;
    }
}

fn run_prop_expanded_object_comparison(
    input: &str,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let prettier_out = exec_cmd(&mut prettier_cmd(), Some(input.as_bytes().to_vec()));
    prop_assume!(prettier_out.status.success());
    // Skip cases where prettier collapses (known difference)
    prop_assume!(prettier_out.stdout.matches('\n').count() > 2);

    let mut jjp = cli();
    jjp.args(["format", "--preferred-width", &PRINT_WIDTH.to_string()]);
    let jjp_out = exec_cmd(&mut jjp, Some(input.as_bytes().to_vec()));
    prop_assert!(jjp_out.status.success(), "jjp failed:\n{}", jjp_out.stderr);
    prop_assert_eq!(&prettier_out.stdout, &jjp_out.stdout, "input: {:?}", input);
    Ok(())
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]

    #[test]
    #[ignore = "slow: calls prettier+jjp for up to 150-entry objects; run with `just test-prettier-large`"]
    fn prop_large_object_expanded_matches_prettier(input in arb_large_object()) {
        run_prop_expanded_object_comparison(&input)?;
    }

    #[test]
    #[ignore = "known: prettier preserves expanded object input in some array-valued object cases, jjp ignores input layout"]
    fn prop_object_with_array_values_matches_prettier(input in arb_object_with_array_values()) {
        run_prop_expanded_object_comparison(&input)?;
    }
}

proptest! {
    #[test]
    #[ignore = "known: prettier expands outer arrays containing multiple sub-arrays, jjp inlines when they fit"]
    fn prop_nested_array_matches_prettier(input in arb_nested_array()) {
        run_prop_comparison(&input)?;
    }

    #[test]
    #[ignore = "known: prettier preserves expanded object layouts in arrays, jjp ignores input layout and inlines fitting objects"]
    fn prop_array_of_objects_matches_prettier(input in arb_array_of_objects()) {
        run_prop_comparison(&input)?;
    }
}

// ── Known-difference tests (ignored until fixed) ───────────────────────────────

proptest! {
    #[test]
    #[ignore = "known: prettier preserves pre-existing line breaks before the first object key, jjp ignores input layout"]
    fn prop_object_matches_prettier(input in arb_object()) {
        run_prop_comparison(&input)?;
    }

    #[test]
    #[ignore = "known: blank lines between object keys — jjp normalizes to one blank line, prettier strips all"]
    fn prop_object_blank_lines_matches_prettier(
        input in prop::collection::vec(("[a-z]{1,8}".prop_map(|k| k), arb_safe_scalar()), 1..=5usize)
            .prop_flat_map(|entries| {
                let n = entries.len();
                let gaps_len = n.saturating_sub(1).max(1);
                (
                    Just(entries),
                    prop::collection::vec(arb_ws_with_blanks(), gaps_len..=gaps_len),
                )
                    .prop_map(|(entries, gaps)| {
                        let mut s = "{".to_string();
                        for (i, (key, val)) in entries.iter().enumerate() {
                            s.push('"');
                            s.push_str(key);
                            s.push('"');
                            s.push(':');
                            s.push_str(val);
                            if i + 1 < entries.len() {
                                s.push(',');
                                s.push_str(&gaps[i]);
                            }
                        }
                        s.push('}');
                        s
                    })
            })
    ) {
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_array_blank_lines_matches_prettier(
        input in prop::collection::vec(arb_safe_scalar(), 0..=4_usize)
            .prop_flat_map(|items| {
                let gaps_len = items.len().saturating_sub(1).max(1);
                (
                    Just(items),
                    prop::collection::vec(arb_ws_with_blanks(), gaps_len..=gaps_len),
                )
                    .prop_map(|(items, gaps)| {
                        if items.is_empty() { return "[]".to_string(); }
                        let mut s = "[".to_string();
                        for (i, item) in items.iter().enumerate() {
                            s.push_str(item);
                            if i + 1 < items.len() {
                                s.push(',');
                                s.push_str(&gaps[i]);
                            }
                        }
                        s.push(']');
                        s
                    })
            })
    ) {
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_zero_exponent_matches_prettier(
        pre in arb_safe_ws(), val in arb_zero_exponent_scalar(), post in arb_safe_ws()
    ) {
        let input = format!("{pre}{val}{post}");
        run_prop_comparison(&input)?;
    }

    #[test]
    #[ignore = "known: prettier normalizes floats/exponents differently from jjp"]
    fn prop_float_matches_prettier(
        pre in arb_safe_ws(), val in arb_float_scalar(), post in arb_safe_ws()
    ) {
        let input = format!("{pre}{val}{post}");
        run_prop_comparison(&input)?;
    }

    #[test]
    fn prop_array_fill_mode_matches_prettier(input in arb_fill_mode_array()) {
        run_prop_comparison(&input)?;
    }
}
