use crate::test_json::*;
use insta::assert_snapshot;
use jjpwrgem_parse::format::{prettify_str, uglify_str};

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
#[case(crate::fixture_tuple!(STANDALONE_NULL))]
#[case(crate::fixture_tuple!(STANDALONE_FALSE))]
#[case(crate::fixture_tuple!(STANDALONE_TRUE))]
#[case(crate::fixture_tuple!(STANDALONE_STRING))]
#[case(crate::fixture_tuple!(NESTED_OBJECT_SINGLE_KEY))]
#[case(crate::fixture_tuple!(STANDALONE_STRING_WS))]
fn format_template(#[case] (name, input): (&str, &str)) {}

#[rstest_reuse::apply(format_template)]
fn prettify(#[case] (name, input): (&str, &str)) {
    assert_snapshot!(name.to_string(), prettify_str(input).unwrap());
}

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    assert_snapshot!(format!("uglify_{name}"), uglify_str(input).unwrap());
}

#[test]
fn uglify_removes_whitespace_object() {
    let input = MULTIKEY_OBJECT_WITH_LOTS_OF_WHITESPACE;
    let res = uglify_str(input).unwrap();
    // we aren't guaranteed a key order
    assert!(
        res == r#"{"hello hi":null,"by":"hello"}"# || res == r#"{"by":"hello","hello hi":null}"#
    );
}
