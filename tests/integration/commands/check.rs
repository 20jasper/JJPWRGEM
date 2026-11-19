use rstest::rstest;

use crate::{
    common::{cli, exec_cmd},
    test_json::*,
};

#[rstest]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COLON_WITH_COMMA))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COLON_WITH_LEADING_WHITESPACE))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COLON_WITH_NULL))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COLON_WITH_CLOSED_CURLY))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COLON))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_VALUE))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COMMA_BETWEEN_VALUES))]
#[case(crate::fixture_tuple!(OBJECT_MISSING_COMMA_OR_CLOSING_WITH_WHITESPACE))]
#[case(crate::fixture_tuple!(OBJECT_TRAILING_COMMA_WITH_CLOSED))]
#[case(crate::fixture_tuple!(OBJECT_TRAILING_COMMA))]
#[case(crate::fixture_tuple!(OBJECT_DOUBLE_OPEN_CURLY))]
#[case(crate::fixture_tuple!(OBJECT_OPEN_CURLY))]
#[case(crate::fixture_tuple!(CLOSED_CURLY))]
#[case(crate::fixture_tuple!(EMPTY_INPUT))]
#[case(crate::fixture_tuple!(DOUBLE_QUOTE))]
#[case(crate::fixture_tuple!(OBJECT_WITH_LINE_BREAK_VALUE))]
#[case(crate::fixture_tuple!(OBJECT_WITH_ADJACENT_STRINGS))]
#[case(crate::fixture_tuple!(OBJECT_EMPTY_THEN_OPEN))]
#[case(crate::fixture_tuple!(UNEXPECTED_CHARACTER))]
#[case(crate::fixture_tuple!(UNEXPECTED_ESCAPED_CHARACTER))]
#[case(crate::fixture_tuple!(LEADING_ZERO_MINUS_SIGN_NONZERO))]
#[case(crate::fixture_tuple!(LEADING_ZERO_MINUS_SIGN_ZERO))]
#[case(crate::fixture_tuple!(LEADING_ZERO_NON_ZERO))]
#[case(crate::fixture_tuple!(LEADING_ZERO_ZERO))]
#[case(crate::fixture_tuple!(MINUS_SIGN))]
#[case(crate::fixture_tuple!(UNEXPECTED_LETTER_IN_NEGATIVE))]
#[case(crate::fixture_tuple!(UNEXPECTED_LETTER_IN_NUMBER))]
#[case(crate::fixture_tuple!(NEGATIVE_FRACTION_MISSING_INTEGER))]
#[case(crate::fixture_tuple!(MISSING_FRACTION))]
#[case(crate::fixture_tuple!(EXPONENT_MISSING_TRAILING_DIGITS))]
#[case(crate::fixture_tuple!(EXPONENT_MISSING_DIGITS_AFTER_SIGN))]
#[case(crate::fixture_tuple!(ARRAY_OPEN))]
#[case(crate::fixture_tuple!(ARRAY_OPEN_WITH_VALUE))]
#[case(crate::fixture_tuple!(ARRAY_MISSING_VALUE))]
#[case(crate::fixture_tuple!(INVALID_HEX_DIGIT_IN_ESCAPE))]
#[case(crate::fixture_tuple!(INVALID_ESCAPED_CURLY))]
fn annotate_test_json_failure_snapshots(#[case] (name, json): (&str, &str)) {
    crate::assert_cli_snapshot!(
        name.to_ascii_lowercase(),
        ["check"],
        Some(json.as_bytes()),
        false
    );
}

#[test]
fn check_help_snapshot() {
    crate::assert_cli_snapshot!("check_help", ["check", "--help"], None, true);
}

#[test]
fn no_stdin() {
    crate::assert_cli_snapshot!("no_stdin", ["check"], None, false);
}

#[rstest::rstest]
#[case(r#"{"coolKey"}"#, "failure")]
#[case(r#"{"hello I am valid": null} "#, "success")]
fn docs(#[case] input: &str, #[case] postfix: &str) {
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        let mut cmd = cli();
        cmd.args(["check"]);

        let output = exec_cmd(&mut cmd, Some(input.as_bytes().to_vec()));

        crate::cli_snapshot!(format!("check_{postfix}"), output.docs_display_stdin());
    });
}
