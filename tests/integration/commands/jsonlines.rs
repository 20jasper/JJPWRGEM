use insta::assert_snapshot;
use rstest::rstest;

use crate::common::{cli, exec_cmd};

#[rstest]
#[case("blank_line", "{\"a\":1}\n\n{\"b\":2}")]
#[case("invalid_json", "{\"a\":1}\n{bad}")]
fn check_invalid(#[case] name: &str, #[case] input: &str) {
    let output = exec_cmd(
        cli().args(["check", "--parser", "jsonlines"]),
        Some(input.as_bytes().to_vec()),
    );
    insta::with_settings!({snapshot_path => "../snapshots"}, {
        assert_snapshot!(name, output.snapshot_display());
    });
    assert!(!output.status.success());
}

#[rstest]
#[case("format_single", "{ \"a\" : 1 }")]
#[case("format_multi", "{ \"a\" : 1 }\n{ \"b\" : 2 }")]
#[case("format_trailing_newline", "{ \"a\" : 1 }\n")]
fn format_valid(#[case] name: &str, #[case] input: &str) {
    let output = exec_cmd(
        cli().args(["format", "--parser", "jsonlines"]),
        Some(input.as_bytes().to_vec()),
    );
    insta::with_settings!({snapshot_path => "../snapshots"}, {
        assert_snapshot!(name, output.snapshot_display());
    });
    assert!(output.status.success());
}

#[test]
fn json_lines_conflicts_with_uglify() {
    let output = exec_cmd(
        cli().args(["format", "--parser", "jsonlines", "--uglify"]),
        Some(b"{}".to_vec()),
    );
    insta::with_settings!({snapshot_path => "../snapshots"}, {
        assert_snapshot!(output.snapshot_display());
    });
    assert!(!output.status.success());
}

#[test]
fn json_lines_conflicts_with_preferred_width() {
    let output = exec_cmd(
        cli().args([
            "format",
            "--parser",
            "jsonlines",
            "--preferred-width",
            "100",
        ]),
        Some(b"{}".to_vec()),
    );
    insta::with_settings!({snapshot_path => "../snapshots"}, {
        assert_snapshot!(output.snapshot_display());
    });
    assert!(!output.status.success());
}
