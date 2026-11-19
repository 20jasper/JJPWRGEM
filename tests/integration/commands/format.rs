use crate::{
    common::{cli, exec_cmd, format_template},
    test_json::*,
};

#[rstest_reuse::apply(format_template)]
fn prettify(#[case] (name, input): (&str, &str)) {
    crate::assert_cli_snapshot!(name.to_string(), ["format"], Some(input.as_bytes()), true);
}

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    crate::assert_cli_snapshot!(
        format!("uglify_{name}"),
        ["format", "--uglify"],
        Some(input.as_bytes()),
        true
    );
}

#[rstest::rstest]
#[case(22, "below_threshold")]
#[case(23, "above_threshold")]
fn preferred_width_threshold(#[case] preferred_width: usize, #[case] label: &str) {
    crate::assert_cli_snapshot!(
        format!("preferred_width_{label}"),
        ["format", "--preferred-width", &preferred_width.to_string()],
        Some(ARRAY_WITH_LONG_STRING.as_bytes()),
        true
    );
}

// Snapshots normalize newlines to LF, so we assert directly to preserve each line ending.
#[rstest::rstest]
#[case(&["--end-of-line", "lf"], "[\n  null\n]\n")]
#[case(&["--end-of-line", "crlf"], "[\r\n  null\r\n]\n")]
#[case(&["--eol", "lf"], "[\n  null\n]\n")]
#[case(&["--eol", "crlf"], "[\r\n  null\r\n]\n")]
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
    crate::assert_cli_snapshot!(
        format!("preferred_width_invalid_{label}"),
        [&["format"], args].concat(),
        None,
        false
    );
}

#[rstest::rstest]
#[case(&["--eol"], "missing")]
#[case(&["--eol=what"], "wrong")]
#[case(&["--eol=899889"], "number")]
fn eol_invalid_args(#[case] args: &[&str], #[case] label: &str) {
    crate::assert_cli_snapshot!(
        format!("eol_invalid_{label}"),
        [&["format"], args].concat(),
        None,
        false
    );
}

#[test]
fn preferred_width_conflicts_with_uglify() {
    crate::assert_cli_snapshot!(
        "preferred_width_conflict",
        ["format", "--uglify", "--preferred-width", "24"],
        None,
        false
    );
}

#[rstest::rstest]
#[case(&["--uglify", "--preferred-width", "24"], "uglify_then_width")]
#[case(&["--preferred-width", "24", "--uglify"], "width_then_uglify")]
#[case(&["--uglify", "--preferred-width", "80"], "uglify_then_default_width")]
fn preferred_width_conflict_cases(#[case] args: &[&str], #[case] label: &str) {
    crate::assert_cli_snapshot!(
        format!("preferred_width_conflict_{label}"),
        [&["format"], args].concat(),
        None,
        false
    );
}

#[rstest::rstest]
#[case(&["--mode", "uglify"], "{\"a\":[1,2]}")]
#[case(&["--mode=uglify"], "{\"a\":[1,2]}")]
#[case(&["--mode", "pretty"], "{ \"a\": [1, 2] }\n")]
#[case(&["-u"], "{\"a\":[1,2]}")]
fn mode(#[case] args: &[&str], #[case] expected: &str) {
    let mut cmd = cli();
    cmd.args(std::iter::once("format").chain(args.iter().copied()));

    let output = exec_cmd(&mut cmd, Some(br#"{ "a": [1,  2] }"#.to_vec()));
    assert!(output.status.success(), "{}", output.snapshot_display());
    assert_eq!(expected.trim_end(), output.stdout.trim_end());
}

#[rstest::rstest]
#[case(&["-u", "--mode", "pretty"], "short_uglify_with_mode_pretty")]
#[case(&["--uglify", "--mode", "uglify"], "uglify_twice")]
#[case(&["--mode", "uglify", "--preferred-width", "80"], "uglify_with_preferred_width")]
#[case(&["-u", "--preferred-width", "80"], "short_uglify_with_preferred_width")]
#[case(&["--mode", "fancy"], "invalid")]
fn mode_invalid_args(#[case] args: &[&str], #[case] label: &str) {
    crate::assert_cli_snapshot!(
        format!("mode_{label}"),
        [&["format"], args].concat(),
        None,
        false
    );
}

#[test]
fn help_subcommand() {
    crate::assert_cli_snapshot!("format_help", ["format", "--help"], None, true);
}

#[rstest::rstest]
#[case(&[], r#"{ "rust":"is a must"   } "#, "pretty")]
#[case(&["--uglify"], r#"{ "shoppingList": ["cheese", "slushy machine"]   } "#, "uglify")]
fn docs(#[case] args: &[&str], #[case] input: &str, #[case] postfix: &str) {
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        let mut cmd = cli();
        cmd.args(std::iter::once("format").chain(args.iter().copied()));

        let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));

        crate::cli_snapshot!(format!("format_{postfix}"), output.docs_display_stdin());
    });
}
