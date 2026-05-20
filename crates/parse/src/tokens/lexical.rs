use core::{fmt::Display, range::RangeInclusive};

use crate::tokens::{CharWithContext, Token};

/// see [`JsonChar::is_whitespace`]
pub fn trim_end_whitespace(s: &str) -> &str {
    let end = s
        .char_indices()
        .map(Into::<CharWithContext>::into)
        .rev()
        .find_map(|CharWithContext(r, c)| (!c.is_whitespace()).then_some(r.end))
        .unwrap_or_default();

    &s[..end]
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct JsonChar(pub char);

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct JsonByte(pub u8);

impl JsonByte {
    pub fn is_whitespace(&self) -> bool {
        matches!(self.0, b' ' | b'\t' | b'\n' | b'\r')
    }

    pub fn as_token(&self) -> Option<Token> {
        let token = match self.0 {
            b'{' => Token::OpenCurlyBrace,
            b'}' => Token::ClosedCurlyBrace,
            b':' => Token::Colon,
            b',' => Token::Comma,
            b'[' => Token::OpenSquareBracket,
            b']' => Token::ClosedSquareBracket,
            _ => return None,
        };
        Some(token)
    }
}

impl JsonChar {
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
    pub fn direct_escape(self) -> Option<&'static str> {
        match self.0 {
            '"' => Some(r#"\""#),
            '\\' => Some(r"\\"),
            '/' => Some(r"\/"),
            '\u{0008}' => Some(r"\b"),
            '\u{000C}' => Some(r"\f"),
            '\n' => Some(r"\n"),
            '\r' => Some(r"\r"),
            '\t' => Some(r"\t"),
            _ => None,
        }
    }

    pub fn minimal_escape(self) -> Option<&'static str> {
        self.direct_escape().filter(|escaped| *escaped != r"\/")
    }

    pub fn escape(self) -> String {
        match self.direct_escape() {
            Some(escaped) => escaped.into(),
            None => format!("\\u{:04X}", u32::from(self.0)),
        }
    }

    pub fn is_hexdigit(&self) -> bool {
        self.0.is_ascii_hexdigit()
    }

    pub fn can_be_escaped_directly(&self) -> bool {
        matches!(self.0, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')
    }

    /// See [RFC 8259, Section 7](https://datatracker.ietf.org/doc/html/rfc8259#section-7)
    pub const CONTROL_RANGE: RangeInclusive<char> = '\u{0000}'..='\u{001F}';

    /// see [`Self::CONTROL_RANGE`]
    pub fn is_control(&self) -> bool {
        Self::CONTROL_RANGE.contains(&self.0)
    }

    /// See [RFC 8259, Section 2](https://datatracker.ietf.org/doc/html/rfc8259#section-2):
    ///
    ///```abnf
    /// ws = *(
    ///         %x20 /              ; Space
    ///         %x09 /              ; Horizontal tab
    ///         %x0A /              ; Line feed or New line
    ///         %x0D )              ; Carriage return
    /// ```
    pub fn is_whitespace(&self) -> bool {
        matches!(self.0, ' ' | '\t' | '\n' | '\r')
    }

    /// See [RFC 8259, Section 2](https://datatracker.ietf.org/doc/html/rfc8259#section-2):
    ///
    /// ```abnf
    /// structural-character = begin-array / begin-object / end-array /
    ///                        end-object / name-separator / value-separator
    ///
    /// begin-array     = ws %x5B ws  ; [ left square bracket
    /// begin-object    = ws %x7B ws  ; { left curly bracket
    /// end-array       = ws %x5D ws  ; ] right square bracket
    /// end-object      = ws %x7D ws  ; } right curly bracket
    /// name-separator  = ws %x3A ws  ; : colon
    /// value-separator = ws %x2C ws  ; , comma
    /// ```
    pub fn is_structural(&self) -> bool {
        matches!(self.0, '{' | '}' | '[' | ']' | ':' | ',')
    }
}

impl Display for JsonChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rep = if self.is_control() {
            self.escape()
        } else {
            self.0.to_string()
        };
        write!(f, "{rep}")
    }
}

impl From<char> for JsonChar {
    fn from(value: char) -> Self {
        Self(value)
    }
}

impl From<u8> for JsonByte {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case('"', Some(r#"\""#))]
    #[case('\\', Some(r"\\"))]
    #[case('/', Some(r"\/"))]
    #[case('\n', Some(r"\n"))]
    #[case('a', None)]
    fn direct_escape_cases(#[case] input: char, #[case] expected: Option<&str>) {
        assert_eq!(JsonChar(input).direct_escape(), expected);
    }

    #[rstest]
    #[case('"', Some(r#"\""#))]
    #[case('\\', Some(r"\\"))]
    #[case('/', None)]
    #[case('\n', Some(r"\n"))]
    #[case('a', None)]
    fn minimal_escape_cases(#[case] input: char, #[case] expected: Option<&str>) {
        assert_eq!(JsonChar(input).minimal_escape(), expected);
    }

    #[rstest]
    #[case('\u{0008}', r"\b")]
    #[case('\u{000C}', r"\f")]
    #[case('\n', r"\n")]
    #[case('\r', r"\r")]
    #[case('\t', r"\t")]
    fn escape_char_for_json_string_short_forms(#[case] input: char, #[case] expected: &str) {
        assert_eq!(JsonChar(input).to_string(), expected);
    }

    #[rstest]
    #[case('\u{0000}', "\\u0000")]
    #[case('\u{001F}', "\\u001F")]
    #[case('\u{0011}', "\\u0011")]
    fn escape_char_for_json_string_control_chars(#[case] input: char, #[case] expected: &str) {
        assert_eq!(JsonChar(input).to_string(), expected);
    }

    #[rstest]
    #[case('a')]
    #[case('Z')]
    #[case('0')]
    #[case(' ')]
    #[case('{')]
    #[case('"')]
    #[case('\\')]
    #[case('/')]
    fn escape_char_for_json_string_leaves_non_specials(#[case] input: char) {
        assert_eq!(JsonChar(input).to_string(), input.to_string());
    }

    #[rstest]
    #[case("h \t\n\r", "h")]
    #[case("\u{000B} h ", "\u{000B} h")]
    #[case("rust", "rust")]
    fn trim_end_whitespace_cases(#[case] input: &str, #[case] output: &str) {
        assert_eq!(trim_end_whitespace(input), output);
    }
}
