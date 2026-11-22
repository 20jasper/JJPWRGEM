use jjpwrgem_parse::{SyntaxOptions, ast::Document, validate_str};
use rstest::rstest;

use crate::common::apply_first_patch;

const COMMENT_MESSAGE: &str = "comments are not allowed in json";

// jsonc[verify single-line-comment]
#[rstest]
#[case::leading("// c\n[1]")]
#[case::trailing_without_newline("[1]// c")]
#[case::after_comma("[1, // c\n2]")]
#[case::cr_line_ending("[1,\r// c\r2]")]
#[case::crlf_line_ending("[1,\r\n// c\r\n2]")]
#[case::slashes_in_string(r#"["// not a comment", 2]"#)]
fn jsonc_accepts_line_comments(#[case] json: &str) {
    assert_eq!(validate_str(json, SyntaxOptions::JSONC), Ok(()));
    assert!(Document::parse(json, SyntaxOptions::JSONC).is_ok());
}

#[rstest]
#[case::leading("// c\n[1]")]
#[case::trailing_without_newline("[1]// c")]
#[case::after_comma("[1, // c\n2]")]
#[case::cr_line_ending("[1,\r// c\r2]")]
#[case::crlf_line_ending("[1,\r\n// c\r\n2]")]
fn json_rejects_line_comments(#[case] json: &str) {
    let error = Document::parse(json, SyntaxOptions::JSON).unwrap_err();
    assert_eq!(error.message(), COMMENT_MESSAGE, "input: {json:?}");
}

#[test]
fn json_accepts_slashes_in_string() {
    let json = r#"["// not a comment", 2]"#;
    assert!(Document::parse(json, SyntaxOptions::JSON).is_ok());
}

// jsonc[verify line-ending]
#[rstest]
#[case::lf("[1, // c\n2]", 4..8)]
#[case::crlf("[1, // c\r\n2]", 4..8)]
#[case::cr("[1, // c\r2]", 4..8)]
fn rejected_line_comment_range_excludes_line_ending(
    #[case] json: &str,
    #[case] range: std::ops::Range<usize>,
) {
    let error = Document::parse(json, SyntaxOptions::JSON).unwrap_err();
    assert_eq!(error.message(), COMMENT_MESSAGE, "input: {json:?}");
    assert_eq!(*error.range(), range.into(), "input: {json:?}");
}

#[rstest]
#[case::after_comma("[1, // c\n2]", "[1,\n2]")]
#[case::own_line(
    "{\n  \"a\": 1,\n  // c\n  \"b\": 2\n}",
    "{\n  \"a\": 1,\n  \"b\": 2\n}"
)]
#[case::leading("// c\n[1]", "\n[1]")]
fn unexpected_comment_patch_removes_comment_and_leading_whitespace(
    #[case] json: &str,
    #[case] expected: &str,
) {
    assert_eq!(
        apply_first_patch(json, SyntaxOptions::JSON),
        expected,
        "input: {json:?}"
    );
}
