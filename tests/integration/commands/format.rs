use crate::common::{cli, exec_cmd, format_template};
use crate::test_json::*;
use insta::assert_snapshot;

#[rstest_reuse::apply(format_template)]
fn prettify(#[case] (name, input): (&str, &str)) {
    let mut cmd = cli();
    cmd.args(["format"]);

    let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));

    assert_snapshot!(name.to_string(), output.snapshot_display());
    assert!(output.status.success());
}

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    let mut cmd = cli();
    cmd.args(["format", "--uglify"]);

    let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));
    assert!(output.status.success());

    assert_snapshot!(format!("uglify_{name}"), output.snapshot_display());
}

#[rstest::rstest]
#[case(22, "below_threshold")]
#[case(23, "above_threshold")]
fn preferred_width_threshold(#[case] preferred_width: usize, #[case] label: &str) {
    let mut cmd = cli();
    cmd.args(["format", "--preferred-width"]);
    let width_arg = preferred_width.to_string();
    cmd.arg(&width_arg);

    let output = exec_cmd(&mut cmd, Some(ARRAY_WITH_LONG_STRING.as_bytes().to_vec()));
    assert!(output.status.success(), "{}", output.snapshot_display());

    assert_snapshot!(
        format!("preferred_width_{label}"),
        output.snapshot_display()
    );
}

// Snapshots normalize newlines to LF, so we assert directly to preserve each line ending.
#[rstest::rstest]
#[case(&["--end-of-line", "lf"], "[\n  null\n]\n")]
#[case(&["--end-of-line", "crlf"], "[\r\n  null\r\n]\n")]
#[case(&["--end-of-line", "cr"], "[\r  null\r]\n")]
#[case(&["--eol", "lf"], "[\n  null\n]\n")]
#[case(&["--eol", "crlf"], "[\r\n  null\r\n]\n")]
#[case(&["--eol", "cr"], "[\r  null\r]\n")]
fn preferred_line_endings(#[case] args: &[&str], #[case] expected: &str) {
    let mut cmd = cli();
    cmd.args(["format", "--preferred-width", "0"]);
    cmd.args(args.iter().copied());

    let output = exec_cmd(&mut cmd, Some(b"[null]".to_vec()));
    assert!(output.status.success(), "{}", output.snapshot_display());
    assert_eq!(expected, output.stdout);
}

#[rstest::rstest]
#[case(&["--preferred-width=-1"], "negative")]
#[case(&["--preferred-width"], "missing")]
#[case(&["--preferred-width=abc"], "letters")]
fn preferred_width_invalid_args(#[case] args: &[&str], #[case] label: &str) {
    let mut cmd = cli();
    cmd.args(std::iter::once("format").chain(args.iter().copied()));

    let output = exec_cmd(&mut cmd, None);
    assert!(!output.status.success());

    assert_snapshot!(
        format!("preferred_width_invalid_{label}"),
        output.snapshot_display()
    );
}

#[rstest::rstest]
#[case(&["--eol"], "missing")]
#[case(&["--eol=what"], "wrong")]
#[case(&["--eol=899889"], "number")]
fn eol_invalid_args(#[case] args: &[&str], #[case] label: &str) {
    let mut cmd = cli();
    cmd.args(std::iter::once("format").chain(args.iter().copied()));

    let output = exec_cmd(&mut cmd, None);
    assert!(!output.status.success());

    assert_snapshot!(format!("eol_invalid_{label}"), output.snapshot_display());
}

#[test]
fn preferred_width_conflicts_with_uglify() {
    let mut cmd = cli();
    cmd.args(["format", "--uglify", "--preferred-width", "24"]);

    let output = exec_cmd(&mut cmd, None);
    assert!(!output.status.success());

    assert_snapshot!("preferred_width_conflict", output.snapshot_display());
}

#[rstest::rstest]
#[case(&["--uglify", "--preferred-width", "24"], "uglify_then_width")]
#[case(&["--preferred-width", "24", "--uglify"], "width_then_uglify")]
fn preferred_width_conflict_cases(#[case] args: &[&str], #[case] label: &str) {
    let mut cmd = cli();
    cmd.args(std::iter::once("format").chain(args.iter().copied()));

    let output = exec_cmd(&mut cmd, None);
    assert!(!output.status.success());

    assert_snapshot!(
        format!("preferred_width_conflict_{label}"),
        output.snapshot_display()
    );
}

#[test]
fn help_subcommand() {
    let mut cmd = cli();
    cmd.args(["format", "--help"]);

    let output = exec_cmd(&mut cmd, None);
    assert!(output.status.success(), "{}", output.snapshot_display());

    assert_snapshot!("format_help", output.snapshot_display());
}

#[test]
fn no_stdin() {
    let mut cmd = cli();
    cmd.args(["check"]);

    let output = exec_cmd(&mut cmd, None);
    assert!(!output.status.success(), "{}", output.snapshot_display());

    assert_snapshot!(output.snapshot_display());
}

#[rstest::rstest]
#[case(&[], r#"{ "rust":"is a must"   } "#, "pretty")]
#[case(&["--uglify"], r#"{ "shoppingList": ["cheese", "slushy machine"]   } "#, "uglify")]
fn docs(#[case] args: &[&str], #[case] input: &str, #[case] postfix: &str) {
    insta::with_settings!({
        snapshot_path => "docs/snapshots",
        prepend_module_to_snapshot => false,
    }, {
        let mut cmd = cli();
        cmd.args(std::iter::once("format").chain(args.iter().copied()));

        let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));

        assert_snapshot!(format!("format_{postfix}"), output.docs_display_stdin());
    });
}
