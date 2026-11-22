use jjpwrgem_parse::{
    SyntaxOptions, ast::Document, diagnostics::Patch, format::uglify_document, validate_str,
};
use rstest::rstest;

const COMMENT_MESSAGE: &str = "comments are not allowed in json";
const UNTERMINATED_MESSAGE: &str = "block comment is never closed";

// jsonc[verify multi-line-comment]
// jsonc[verify json-syntax-with-comments]
// jsonc[verify comments-where-whitespace-allowed]
#[rstest]
#[case::leading("/* c */[1]")]
#[case::trailing("[1]/* c */")]
#[case::between_values("[1, /* c */ 2]")]
#[case::before_comma("[1 /* c */, 2]")]
#[case::between_key_and_colon(r#"{"a" /* c */ : 1}"#)]
#[case::after_colon(r#"{"a": /* c */ 1}"#)]
#[case::inside_empty_array("[/* c */]")]
#[case::multiline("[1, /* line one\nline two\r\nline three */ 2]")]
#[case::star_inside("[/* * ** */ 1]")]
#[case::slash_inside("[/* / // */ 1]")]
#[case::line_comment_opener_inside("[/* // */ 1]")]
#[case::empty("[/**/1]")]
#[case::stars_only("[/***/1]")]
#[case::between_tokens("[1/**/,2]")]
#[case::slashes_in_string(r#"["/* not a comment */", 2]"#)]
fn jsonc_accepts_block_comments(#[case] json: &str) {
    assert_eq!(validate_str(json, SyntaxOptions::JSONC), Ok(()));
    assert!(Document::parse(json, SyntaxOptions::JSONC).is_ok());
}

// jsonc[verify comments-do-not-affect-data]
// jsonc[verify comments-ignored]
#[rstest]
#[case::array("[1, /* c */ 2]", "[1, 2]")]
#[case::object(r#"{"a" /* c */ : /* d */ 1}"#, r#"{"a": 1}"#)]
#[case::multiline("[1, /* a\nb */ 2]", "[1, 2]")]
#[case::string_untouched(r#"["/* x */"]"#, r#"["/* x */"]"#)]
fn comments_do_not_affect_data(#[case] with_comments: &str, #[case] without: &str) {
    let with = Document::parse(with_comments, SyntaxOptions::JSONC).unwrap();
    let without = Document::parse(without, SyntaxOptions::JSONC).unwrap();
    assert_eq!(uglify_document(&with), uglify_document(&without));
}

#[rstest]
#[case::leading("/* c */[1]")]
#[case::between_values("[1, /* c */ 2]")]
#[case::multiline("[1, /* a\nb */ 2]")]
#[case::empty("[/**/1]")]
fn json_rejects_block_comments(#[case] json: &str) {
    let error = Document::parse(json, SyntaxOptions::JSON).unwrap_err();
    assert_eq!(error.message(), COMMENT_MESSAGE, "input: {json:?}");
}

#[rstest]
#[case::json(SyntaxOptions::JSON)]
#[case::jsonc(SyntaxOptions::JSONC)]
fn slash_star_inside_string_is_untouched(#[case] options: SyntaxOptions) {
    let json = r#"["/* a", "b */", "/**/"]"#;
    assert!(Document::parse(json, options).is_ok());
}

// jsonc[verify no-nested-comments]
#[test]
fn nested_comment_ends_at_first_close() {
    assert!(Document::parse("[/* /* */ 1]", SyntaxOptions::JSONC).is_ok());
    let error = Document::parse("[/* /* */ */ 1]", SyntaxOptions::JSONC).unwrap_err();
    assert_eq!(
        error.message(),
        "unexpected character `*`. expected start of a json value"
    );
    assert_eq!(*error.range(), (10..11).into());
}

#[rstest]
#[case::unclosed("[1, /* c")]
#[case::just_opener("/*")]
#[case::almost_closed("[1] /* c *")]
#[case::opener_overlaps_close("[1] /*/")]
fn jsonc_rejects_unterminated_comment(#[case] json: &str) {
    let error = Document::parse(json, SyntaxOptions::JSONC).unwrap_err();
    assert_eq!(error.message(), UNTERMINATED_MESSAGE, "input: {json:?}");
    assert_eq!(
        *error.range(),
        (json.find("/*").unwrap()..json.len()).into()
    );
}

#[rstest]
#[case::unclosed("[1, /* c")]
#[case::just_opener("/*")]
fn json_rejects_unterminated_comment(#[case] json: &str) {
    let error = Document::parse(json, SyntaxOptions::JSON).unwrap_err();
    assert_eq!(error.message(), COMMENT_MESSAGE, "input: {json:?}");
    assert_eq!(
        *error.range(),
        (json.find("/*").unwrap()..json.len()).into()
    );
    assert_eq!(Vec::<Patch<'_>>::from(&error), []);
}

// jsonc[verify no-hash-comments]
#[rstest]
#[case::leading("# c\n[1]")]
#[case::trailing("[1] # c")]
fn jsonc_rejects_hash_comments(#[case] json: &str) {
    assert!(Document::parse(json, SyntaxOptions::JSONC).is_err());
}

fn apply_first_patch(json: &str, options: SyntaxOptions) -> String {
    let error = Document::parse(json, options).unwrap_err();
    let patches = Vec::<Patch<'_>>::from(&error);
    let [patch] = patches.as_slice() else {
        panic!("expected exactly one patch, got {patches:?}");
    };
    let mut fixed = json.to_string();
    fixed.replace_range(patch.span, &patch.replacement);
    fixed
}

#[rstest]
#[case::after_comma("[1, /* c */ 2]", "[1, 2]")]
#[case::own_line(
    "{\n  \"a\": 1,\n  /* c\n  d */\n  \"b\": 2\n}",
    "{\n  \"a\": 1,\n  \"b\": 2\n}"
)]
#[case::adjacent_to_punctuation("[1,/**/2]", "[1,2]")]
#[case::joins_numbers("[1/*x*/2]", "[1 2]")]
#[case::joins_literals("[true/**/false]", "[true false]")]
#[case::joins_minus("[1/**/-2]", "[1 -2]")]
#[case::whitespace_before_and_alnum_after("[1 /*x*/2]", "[1 2]")]
fn unexpected_block_comment_patch(#[case] json: &str, #[case] expected: &str) {
    assert_eq!(
        apply_first_patch(json, SyntaxOptions::JSON),
        expected,
        "input: {json:?}"
    );
}

#[test]
fn unterminated_comment_patch_closes_comment() {
    assert_eq!(
        apply_first_patch("[1, /* c", SyntaxOptions::JSONC),
        "[1, /* c*/"
    );
}
