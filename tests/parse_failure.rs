use annotate_snippets::{Renderer, renderer::DecorStyle};
use insta::assert_snapshot;
use jjpwrgem::test_json::*;
use rstest::rstest;

macro_rules! fixture_tuple {
    ($const:ident) => {
        (stringify!($const), $const)
    };
}

#[rstest]
#[case(fixture_tuple!(OBJECT_MISSING_COLON_WITH_COMMA))]
#[case(fixture_tuple!(OBJECT_MISSING_COLON_WITH_LEADING_WHITESPACE))]
#[case(fixture_tuple!(OBJECT_MISSING_COLON_WITH_NULL))]
#[case(fixture_tuple!(OBJECT_MISSING_COLON_WITH_CLOSED_CURLY))]
#[case(fixture_tuple!(OBJECT_MISSING_COLON))]
#[case(fixture_tuple!(OBJECT_MISSING_VALUE))]
#[case(fixture_tuple!(OBJECT_MISSING_COMMA_BETWEEN_VALUES))]
#[case(fixture_tuple!(OBJECT_MISSING_COMMA_OR_CLOSING_WITH_WHITESPACE))]
#[case(fixture_tuple!(OBJECT_TRAILING_COMMA_WITH_CLOSED))]
#[case(fixture_tuple!(OBJECT_TRAILING_COMMA))]
#[case(fixture_tuple!(OBJECT_DOUBLE_OPEN_CURLY))]
#[case(fixture_tuple!(OBJECT_OPEN_CURLY))]
#[case(fixture_tuple!(CLOSED_CURLY))]
#[case(fixture_tuple!(EMPTY_INPUT))]
// #[case(fixture_tuple!(DOUBLE_QUOTE))]
#[case(fixture_tuple!(OBJECT_WITH_LINE_BREAK_VALUE))]
#[case(fixture_tuple!(OBJECT_WITH_ADJACENT_STRINGS))]
#[case(fixture_tuple!(OBJECT_EMPTY_THEN_OPEN))]
#[case(fixture_tuple!(UNEXPECTED_CHARACTER))]
#[case(fixture_tuple!(UNEXPECTED_ESCAPED_CHARACTER))]
#[case(fixture_tuple!(LEADING_ZERO_MINUS_SIGN_NONZERO))]
#[case(fixture_tuple!(LEADING_ZERO_MINUS_SIGN_ZERO))]
#[case(fixture_tuple!(LEADING_ZERO_NON_ZERO))]
#[case(fixture_tuple!(LEADING_ZERO_ZERO))]
#[case(fixture_tuple!(MINUS_SIGN))]
#[case(fixture_tuple!(UNEXPECTED_LETTER_IN_NEGATIVE))]
#[case(fixture_tuple!(UNEXPECTED_LETTER_IN_NUMBER))]
// #[case(fixture_tuple!(FRACTION_MISSING_INTEGER))]
#[case(fixture_tuple!(NEGATIVE_FRACTION_MISSING_INTEGER))]
#[case(fixture_tuple!(MISSING_FRACTION))]
#[case(fixture_tuple!(VALID_FRACTION))]
#[case(fixture_tuple!(VALID_NEGATIVE_FRACTION))]
#[case(fixture_tuple!(VALID_INTEGER))]
#[case(fixture_tuple!(VALID_NEGATIVE_INTEGER))]
#[case(fixture_tuple!(LONG_INTEGER))]
#[case(fixture_tuple!(LONG_FRACTION))]
#[case(fixture_tuple!(EXPONENT_WITH_PLUS_SIGN))]
#[case(fixture_tuple!(EXPONENT_WITH_MINUS_SIGN))]
#[case(fixture_tuple!(NEGATIVE_FLOAT_WITH_EXPONENT))]
#[case(fixture_tuple!(EXPONENT_MISSING_TRAILING_DIGITS))]
#[case(fixture_tuple!(EXPONENT_MISSING_DIGITS_AFTER_SIGN))]
#[case(fixture_tuple!(ARRAY_EMPTY))]
#[case(fixture_tuple!(ARRAY_SINGLE))]
#[case(fixture_tuple!(ARRAY_MANY))]
#[case(fixture_tuple!(ARRAY_SUBARRAYS))]
#[case(fixture_tuple!(ARRAY_OPEN))]
#[case(fixture_tuple!(ARRAY_OPEN_WITH_VALUE))]
#[case(fixture_tuple!(ARRAY_MISSING_VALUE))]
// #[case(fixture_tuple!(INVALID_HEX_DIGIT_IN_ESCAPE))]
// #[case(fixture_tuple!(INVALID_ESCAPED_CURLY))]
fn annotate_test_json_fixtures_snapshots(#[case] (name, json): (&str, &str)) {
    use jjpwrgem::cli::run;

    let json = json.as_bytes().to_vec();

    let renderer = Renderer::plain().decor_style(DecorStyle::Ascii);
    let annotated = run(json.clone(), &renderer);

    assert_snapshot!(
        name.to_ascii_lowercase(),
        format_output_snapshot(json, &annotated)
    );
}
