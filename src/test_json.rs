use crate::cli::Output;

pub const OBJECT_MISSING_COLON_WITH_COMMA: &str = r#"{"hi", "#;
pub const OBJECT_MISSING_COLON_WITH_NULL: &str = r#"{"hi" null "#;
pub const OBJECT_MISSING_COLON_WITH_CLOSED_CURLY: &str = r#"{"hi" }"#;
pub const OBJECT_MISSING_COLON_WITH_LEADING_WHITESPACE: &str = r#"  {"hi"    "#;
pub const OBJECT_MISSING_COLON: &str = r#"{"hi"    "#;
pub const OBJECT_MISSING_VALUE: &str = r#"{"hi":"#;
pub const OBJECT_MISSING_COMMA_BETWEEN_VALUES: &str = r#"{"hi": null null"#;
pub const OBJECT_MISSING_COMMA_OR_CLOSING_WITH_WHITESPACE: &str = r#"{"hi": null     "#;
pub const OBJECT_TRAILING_COMMA_WITH_CLOSED: &str = r#"{"hi": null, }"#;
pub const OBJECT_TRAILING_COMMA: &str = r#"{"hi": null, "#;
pub const OBJECT_DOUBLE_OPEN_CURLY: &str = r#"{{"#;
pub const OBJECT_OPEN_CURLY: &str = r#"{"#;
pub const CLOSED_CURLY: &str = r#"}"#;
pub const EMPTY_INPUT: &str = r#""#;
pub const UNEXPECTED_CHARACTER: &str = r#"🦀"#;
pub const UNEXPECTED_ESCAPED_CHARACTER: &str = "\u{B}";
pub const DOUBLE_QUOTE: &str = r#"""#;
pub const OBJECT_WITH_LINE_BREAK_VALUE: &str = "{\"hi\": \"line\nbreak\"}";
pub const OBJECT_WITH_ADJACENT_STRINGS: &str = r#"{"hi": "bye" "ferris": null"#;
pub const OBJECT_EMPTY_THEN_OPEN: &str = r#"{}{"#;
pub const MINUS_SIGN: &str = "-";
pub const LEADING_ZERO_MINUS_SIGN: &str = "-000";
pub const LEADING_ZERO: &str = "000";
pub const UNEXPECTED_LETTER_IN_NEGATIVE: &str = "-abcd";
pub const UNEXPECTED_LETTER_IN_NUMBER: &str = "1a";
pub const FRACTION_MISSING_INTEGER: &str = ".29";
pub const NEGATIVE_FRACTION_MISSING_INTEGER: &str = "-.29";
pub const VALID_INTEGER: &str = "298";
pub const VALID_NEGATIVE_INTEGER: &str = "-298";
pub const MISSING_FRACTION: &str = "98.";
pub const VALID_FRACTION: &str = "98.8";
pub const VALID_NEGATIVE_FRACTION: &str = "-98.123456789";
pub const LONG_INTEGER: &str = "4390430989084309809824123456780099876654433231123413847890813843897873986381727319297072310970972784365768257862";
pub const LONG_FRACTION: &str = "439043098908430980982412345678009987.6654433231123413847890813843897873986381727319297072310970972784365768257862";
pub const EXPONENT_WITH_PLUS_SIGN: &str = "429e+6";
pub const EXPONENT_WITH_MINUS_SIGN: &str = "429e-6";
pub const NEGATIVE_FLOAT_WITH_EXPONENT: &str = "-98.25e12";
pub const EXPONENT_MISSING_TRAILING_DIGITS: &str = "98e";
pub const EXPONENT_MISSING_DIGITS_AFTER_SIGN: &str = "98e+";

pub fn format_output_snapshot(input: &str, output: &Output) -> String {
    format!("case --- \n{input}\n{output:?}")
}
