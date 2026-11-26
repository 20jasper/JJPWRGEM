use rstest::rstest;

use crate::common::assert_cli_snapshot;

const TRAILING_COMMA: &[u8] = b"{\"a\":\n  \"\xc3\xa9\" ,}";

#[test]
fn check_success() {
    assert_cli_snapshot!(
        "json_check_success",
        ["check", "--reporter", "json"],
        Some(br#"{"a": 1}"#),
        true
    );
}

#[test]
fn check_failure_with_context_and_fixes() {
    assert_cli_snapshot!(
        "json_check_failure",
        ["check", "--reporter", "json"],
        Some(TRAILING_COMMA),
        false
    );
}

#[test]
fn format_success() {
    assert_cli_snapshot!(
        "json_format_success",
        ["format", "--reporter", "json"],
        Some(br#"{"a":   [1,2]}"#),
        true
    );
}

#[test]
fn format_failure() {
    assert_cli_snapshot!(
        "json_format_failure",
        ["format", "--reporter", "json"],
        Some(br#"{"coolKey"}"#),
        false
    );
}

#[test]
fn stdin_required() {
    assert_cli_snapshot!(
        "json_no_stdin",
        ["check", "--reporter", "json"],
        None,
        false
    );
}

#[test]
fn invalid_utf8() {
    assert_cli_snapshot!(
        "json_invalid_utf8",
        ["check", "--reporter", "json"],
        Some(b"{\"a\": \"\xff\"}"),
        false
    );
}

#[rstest]
#[case("check", br#"{"a": 1}"#, true)]
#[case("check", TRAILING_COMMA, false)]
#[case("format", br#"{"a":   [1,2]}"#, true)]
#[case("format", br#"{"coolKey"}"#, false)]
fn shorthand_matches_long_flag(#[case] command: &str, #[case] input: &[u8], #[case] ok: bool) {
    let run = |flag: &[&str]| {
        let args = [&[command], flag].concat();
        let output = crate::common::exec_cmd(crate::common::cli().args(args), Some(input.to_vec()));
        assert_eq!(output.status.success(), ok, "{}", output.snapshot_display());
        (output.stdout, output.stderr)
    };
    assert_eq!(run(&["--json"]), run(&["--reporter", "json"]));
}
