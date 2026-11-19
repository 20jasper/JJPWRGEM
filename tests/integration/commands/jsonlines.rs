use rstest::rstest;

use crate::common::{cli, exec_cmd};

#[rstest]
#[case("blank_line", "{\"a\":1}\n\n{\"b\":2}")]
#[case("invalid_json", "{\"a\":1}\n{bad}")]
fn check_invalid(#[case] name: &str, #[case] input: &str) {
    crate::assert_cli_snapshot!(
        name,
        ["check", "--parser", "jsonlines"],
        Some(input.as_bytes()),
        false
    );
}

#[rstest]
#[case("format_single", "{ \"a\" : 1 }")]
#[case("format_multi", "{ \"a\" : 1 }\n{ \"b\" : 2 }")]
#[case("format_trailing_newline", "{ \"a\" : 1 }\n")]
fn format_valid(#[case] name: &str, #[case] input: &str) {
    crate::assert_cli_snapshot!(
        name,
        ["format", "--parser", "jsonlines"],
        Some(input.as_bytes()),
        true
    );
}

#[rstest]
#[case(&["--end-of-line", "lf"], "{\"a\":1}\n{\"b\":2}\n")]
#[case(&["--end-of-line", "crlf"], "{\"a\":1}\r\n{\"b\":2}\n")]
#[case(&["--eol", "crlf"], "{\"a\":1}\r\n{\"b\":2}\n")]
fn format_line_endings(#[case] args: &[&str], #[case] expected: &str) {
    let output = exec_cmd(
        cli()
            .args(["format", "--parser", "jsonlines"])
            .args(args.iter().copied()),
        Some(b"{ \"a\" : 1 }\n{ \"b\" : 2 }".to_vec()),
    );
    assert!(output.status.success(), "{}", output.snapshot_display());
    assert_eq!(expected, output.stdout);
}

#[test]
fn json_lines_conflicts_with_uglify() {
    crate::assert_cli_snapshot!(
        "json_lines_conflicts_with_uglify",
        ["format", "--parser", "jsonlines", "--uglify"],
        Some(b"{}"),
        false
    );
}

#[test]
fn json_lines_conflicts_with_preferred_width() {
    crate::assert_cli_snapshot!(
        "json_lines_conflicts_with_preferred_width",
        [
            "format",
            "--parser",
            "jsonlines",
            "--preferred-width",
            "100"
        ],
        Some(b"{}"),
        false
    );
}

#[test]
fn json_lines_with_uglify_and_preferred_width_reports_uglify_first() {
    crate::assert_cli_snapshot!(
        "json_lines_with_uglify_and_preferred_width_reports_uglify_first",
        [
            "format",
            "--uglify",
            "--preferred-width",
            "24",
            "--parser",
            "jsonlines",
        ],
        Some(b"{}"),
        false
    );
}

#[test]
fn json_lines_conflicts_with_default_preferred_width() {
    crate::assert_cli_snapshot!(
        "json_lines_conflicts_with_default_preferred_width",
        ["format", "--parser", "jsonlines", "--preferred-width", "80"],
        Some(b"{}"),
        false
    );
}
