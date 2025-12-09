use crate::common::{cli, exec_cmd};
use crate::test_json::*;
use insta::assert_snapshot;

#[rstest_reuse::template]
#[rstest::rstest]
#[case(crate::fixture_tuple!(VALID_FRACTION))]
#[case(crate::fixture_tuple!(VALID_NEGATIVE_FRACTION))]
#[case(crate::fixture_tuple!(VALID_INTEGER))]
#[case(crate::fixture_tuple!(VALID_NEGATIVE_INTEGER))]
#[case(crate::fixture_tuple!(LONG_INTEGER))]
#[case(crate::fixture_tuple!(LONG_FRACTION))]
#[case(crate::fixture_tuple!(EXPONENT_WITH_PLUS_SIGN))]
#[case(crate::fixture_tuple!(EXPONENT_WITH_MINUS_SIGN))]
#[case(crate::fixture_tuple!(NEGATIVE_FLOAT_WITH_EXPONENT))]
#[case(crate::fixture_tuple!(ARRAY_EMPTY))]
#[case(crate::fixture_tuple!(ARRAY_SINGLE))]
#[case(crate::fixture_tuple!(ARRAY_MANY))]
#[case(crate::fixture_tuple!(ARRAY_SUBARRAYS))]
#[case(crate::fixture_tuple!(ARRAY_OBJECTS_WITH_INCREASING_KEYS))]
#[case(crate::fixture_tuple!(ARRAY_MULTIPLE_EMPTY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_SINGLE_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_TWO_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_FIVE_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAYS_NESTED_FIVE_LEVELS_WITH_OBJECT))]
#[case(crate::fixture_tuple!(STANDALONE_NULL))]
#[case(crate::fixture_tuple!(STANDALONE_FALSE))]
#[case(crate::fixture_tuple!(STANDALONE_TRUE))]
#[case(crate::fixture_tuple!(STANDALONE_STRING))]
#[case(crate::fixture_tuple!(NESTED_OBJECT_SINGLE_KEY))]
#[case(crate::fixture_tuple!(OBJECT_WITH_LONG_KEYS))]
#[case(crate::fixture_tuple!(ARRAY_WITH_NESTED_OBJECTS))]
#[case(crate::fixture_tuple!(MIXED_ARRAY_WITH_LONG_STRINGS))]
#[case(crate::fixture_tuple!(STANDALONE_STRING_WS))]
#[case(crate::fixture_tuple!(DEEPLY_NESTED))]
fn format_template(#[case] (name, input): (&str, &str)) {}

#[rstest_reuse::apply(format_template)]
fn prettify(#[case] (name, input): (&str, &str)) {
    let mut cmd = cli();
    cmd.args(["format"]);

    let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));
    assert!(output.status.success());

    assert_snapshot!(name.to_string(), output.snapshot_display());
}

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    let mut cmd = cli();
    cmd.args(["format", "--uglify"]);

    let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));
    assert!(output.status.success());

    assert_snapshot!(format!("uglify_{name}"), output.snapshot_display());
}

#[test]
fn uglify_removes_whitespace_object() {
    let output = exec_cmd(
        cli().args(["format", "--uglify"]),
        Some(MULTIKEY_OBJECT_WITH_LOTS_OF_WHITESPACE.as_bytes().to_vec()),
    );
    assert!(output.status.success());

    assert!(
        output.stdout.trim() == r#"{"hello hi":null,"by":"hello"}"#
            || output.stdout.trim() == r#"{"by":"hello","hello hi":null}"#,
        "unexpected uglify result: {}",
        output.stdout
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
