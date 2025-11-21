use crate::error::Error;
use crate::tokens::CONTROL_RANGE;
use crate::{ErrorKind, Result};
use core::{iter::Peekable, str::CharIndices};

pub fn build_str_while<'a>(
    start: usize,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> Result<&'a str> {
    let mut escape = false;
    while let Some((_, c)) =
        chars.next_if(|(_, c)| (*c != '"' && !CONTROL_RANGE.contains(c)) || escape)
    {
        escape = c == '\\' && !escape;
    }

    if let Some((end, c)) = chars.next() {
        if !CONTROL_RANGE.contains(&c) {
            Ok(&input[start..end])
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedControlCharacterInString(escape_char_for_json_string(c)),
                end..end + c.len_utf8(),
                input,
            ))
        }
    } else {
        Err(Error::from_unterminated(ErrorKind::ExpectedQuote, input))
    }
}

///```abnf
/// char = unescaped /
///       escape (
///           ...
///           %x75 4HEXDIG )  ; uXXXX                U+XXXX
/// ```
/// see [RFC 8249 section 7](https://datatracker.ietf.org/doc/html/rfc8259#section-7)
pub fn escape_char_for_json(c: char) -> String {
    format!("\\u{:04X}", u32::from(c))
}

///```abnf
/// char = unescaped /
///       escape (
///           %x22 /          ; "    quotation mark  U+0022
///           %x5C /          ; \    reverse solidus U+005C
///           %x2F /          ; /    solidus         U+002F
///           %x62 /          ; b    backspace       U+0008
///           %x66 /          ; f    form feed       U+000C
///           %x6E /          ; n    line feed       U+000A
///           %x72 /          ; r    carriage return U+000D
///           %x74 /          ; t    tab             U+0009
///           %x75 4HEXDIG )  ; uXXXX                U+XXXX
///       escape = %x5C              ; \
///
/// quotation-mark = %x22      ; "
///
/// unescaped = %x20-21 / %x23-5B / %x5D-10FFFF
/// ```
/// see [RFC 8249 section 7](https://datatracker.ietf.org/doc/html/rfc8259#section-7)
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
        c if CONTROL_RANGE.contains(&c) => escape_char_for_json(c),
        c => c.to_string(),
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
    fn short_form_escapes(#[case] input: char, #[case] expected: &str) {
        assert_eq!(escape_char_for_json_string(input), expected);
    }

    #[rstest]
    #[case('\u{0000}', "\\u0000")]
    #[case('\u{001F}', "\\u001F")]
    #[case('\u{0011}', "\\u0011")]
    fn unicode_escapes_other_control_chars(#[case] input: char, #[case] expected: &str) {
        assert_eq!(escape_char_for_json_string(input), expected);
    }

    #[rstest]
    #[case('a')]
    #[case('Z')]
    #[case('0')]
    #[case(' ')]
    #[case('{')]
    fn leaves_non_special_characters_unescaped(#[case] input: char) {
        assert_eq!(escape_char_for_json_string(input), input.to_string());
    }
}
