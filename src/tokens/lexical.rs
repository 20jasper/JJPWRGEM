use core::ops::RangeInclusive;

pub const CONTROL_RANGE: RangeInclusive<char> = '\u{0000}'..='\u{001F}';

pub fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

pub fn trim_end_whitespace(s: &str) -> &str {
    let end = s
        .char_indices()
        .rev()
        .find(|(_, c)| !is_whitespace(*c))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or_default();

    &s[..end]
}

pub fn escape_char_for_json(c: char) -> String {
    format!("\\u{:04X}", u32::from(c))
}

pub fn escape_char_for_json_string(c: char) -> String {
    match c {
        '\u{0008}' => r"\b".into(),
        '\u{000C}' => r"\f".into(),
        '\n' => r"\n".into(),
        '\r' => r"\r".into(),
        '\t' => r"\t".into(),
        '"' => r#"\""#.into(),
        '\\' => r"\\".into(),
        '/' => r"\/".into(),
        ch if CONTROL_RANGE.contains(&ch) => escape_char_for_json(ch),
        ch => ch.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case('\u{0008}', r"\b")]
    #[case('\u{000C}', r"\f")]
    #[case('\n', r"\n")]
    #[case('\r', r"\r")]
    #[case('\t', r"\t")]
    #[case('"', r#"\""#)]
    #[case('\\', r"\\")]
    #[case('/', r"\/")]
    fn escape_char_for_json_string_short_forms(#[case] input: char, #[case] expected: &str) {
        assert_eq!(escape_char_for_json_string(input), expected);
    }

    #[rstest]
    #[case('\u{0000}', "\\u0000")]
    #[case('\u{001F}', "\\u001F")]
    #[case('\u{0011}', "\\u0011")]
    fn escape_char_for_json_string_control_chars(#[case] input: char, #[case] expected: &str) {
        assert_eq!(escape_char_for_json_string(input), expected);
    }

    #[rstest]
    #[case('a')]
    #[case('Z')]
    #[case('0')]
    #[case(' ')]
    #[case('{')]
    fn escape_char_for_json_string_leaves_non_specials(#[case] input: char) {
        assert_eq!(escape_char_for_json_string(input), input.to_string());
    }

    #[rstest]
    #[case("h \t\n\r", "h")]
    #[case("\u{000B} h ", "\u{000B} h")]
    #[case("rust", "rust")]
    fn trim_end_whitespace_cases(#[case] input: &str, #[case] output: &str) {
        assert_eq!(trim_end_whitespace(input), output);
    }
}
