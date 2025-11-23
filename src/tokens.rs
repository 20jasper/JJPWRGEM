pub mod lexical;

pub use lexical::{CONTROL_RANGE, trim_end_whitespace};

use self::lexical::{escape_char_for_json_string, is_whitespace};
use crate::tokens::number::parse_num;
use crate::{Error, ErrorKind, Result};
use core::fmt::Display;
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenCurlyBrace,
    ClosedCurlyBrace,
    Colon,
    Comma,
    String(String),
    Number(String),
    Null,
    Boolean(bool),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Token::OpenCurlyBrace => "{",
            Token::ClosedCurlyBrace => "}",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::String(x) => &format!("{x:?}"),
            Token::Number(x) => &x.to_string(),
            Token::Boolean(x) => &format!("{x:?}"),
            Token::Null => NULL,
        };
        write!(f, "`{val}`")
    }
}

const NO_SIGNIFICANT_CHARACTERS: &str = "no significant characters";
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenOption(pub(crate) Option<Token>);

impl Display for TokenOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match &self.0 {
            Some(x) => x.to_string(),
            None => NO_SIGNIFICANT_CHARACTERS.to_owned(),
        };
        write!(f, "{val}")
    }
}

impl From<Option<Token>> for TokenOption {
    fn from(value: Option<Token>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenWithContext {
    pub token: Token,
    pub range: Range<usize>,
}

pub const NULL: &str = "null";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";

pub fn str_to_tokens(s: &str) -> Result<Vec<TokenWithContext>> {
    let mut chars = s.char_indices().peekable();

    let mut res = vec![];

    while let Some(&(i, c)) = chars.peek() {
        if is_whitespace(c) {
            chars.next();
            continue;
        }
        let token = match c {
            '{' => {
                chars.next();
                Token::OpenCurlyBrace
            }
            '}' => {
                chars.next();
                Token::ClosedCurlyBrace
            }
            ':' => {
                chars.next();
                Token::Colon
            }
            ',' => {
                chars.next();
                Token::Comma
            }
            '"' => {
                chars.next();
                // TODO to parse and handle it's own start
                Token::String(parse_str(i + 1, s, &mut chars)?.into())
            }
            '0'..='9' | '-' => {
                res.push(parse_num(s, &mut chars)?);
                continue;
            }
            'n' | 't' | 'f' => {
                let expected = match c {
                    'n' => NULL,
                    't' => TRUE,
                    'f' => FALSE,
                    _ => unreachable!("{c} is not able to be reached"),
                };
                let actual = chars.by_ref().take(expected.len()).map(|(_, c)| c);

                if actual.eq(expected.chars()) {
                    match c {
                        'n' => Token::Null,
                        't' => true.into(),
                        'f' => false.into(),
                        _ => unreachable!("{c} is not able to be reached"),
                    }
                } else {
                    return Err(Error::new(
                        ErrorKind::UnexpectedCharacter(c.into()),
                        i..(i + c.len_utf8()),
                        s,
                    ));
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter(c.into()),
                    i..(i + c.len_utf8()),
                    s,
                ));
            }
        };
        let start = i;
        let end = *chars.peek().map(|(i, _)| i).unwrap_or(&s.len());
        res.push(TokenWithContext {
            token,
            range: start..end,
        });
    }

    Ok(res)
}

mod number {
    use core::{iter::Peekable, ops::Range, str::CharIndices};

    use crate::{
        Error, ErrorKind, Result,
        tokens::{Token, TokenWithContext, lexical::is_whitespace},
    };

    #[derive(Debug, PartialEq, Eq, Clone)]
    enum NumberState {
        MinusOrInteger,
        Leading(Range<usize>), // TODO char with context type??
        IntegerOrDecimalOrExponentOrEnd {
            leading: Option<char>,
            ctx: Range<usize>,
        },
        Fraction(Range<usize>),
        FractionOrExponentOrEnd(Range<usize>),
        ExponentOrEnd(Range<usize>),
        End(TokenWithContext),
    }

    impl NumberState {
        fn process(
            self,
            chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
            input: &str,
        ) -> Result<Self> {
            let res = match self {
                NumberState::MinusOrInteger => match chars.next() {
                    Some((i, c @ '-')) => NumberState::Leading(i..i + c.len_utf8()),
                    Some((i, leading @ '0'..='9')) => {
                        NumberState::IntegerOrDecimalOrExponentOrEnd {
                            leading: Some(leading),
                            ctx: i..i + leading.len_utf8(),
                        }
                    }
                    _ => todo!("err, number must start with `-` or digit"),
                },
                NumberState::Leading(range) => match chars.next() {
                    Some((i, digit @ '0'..='9')) => NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(digit),
                        ctx: range.start..i + digit.len_utf8(),
                    },
                    c @ (Some(_) | None) => {
                        return Err(Error::new(
                            ErrorKind::ExpectedDigitFollowingMinus(
                                range.clone(),
                                c.map(|(_, c)| c.into()),
                            ),
                            range.clone().start
                                ..c.map(|(i, c)| i + c.len_utf8()).unwrap_or(input.len()),
                            input,
                        ));
                    }
                },
                NumberState::IntegerOrDecimalOrExponentOrEnd { leading, ctx } => match chars.next()
                {
                    Some((_, '0')) if matches!(leading, Some('0')) => {
                        todo!("err, leading 0s are not allowed big nerd")
                    }
                    Some((i, c @ '0'..='9')) => NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: None,
                        ctx: ctx.start..i + c.len_utf8(),
                    },
                    Some((_, '.')) => {
                        todo!("handle frac")
                    }
                    Some((_, 'e' | 'E')) => {
                        todo!("handle exp")
                    }
                    Some((_, ws)) if is_whitespace(ws) => NumberState::End(TokenWithContext {
                        token: Token::Number(input[ctx.clone()].into()),
                        range: ctx,
                    }),
                    None => NumberState::End(TokenWithContext {
                        token: Token::Number(input[ctx.clone()].into()),
                        range: ctx,
                    }),
                    _ => todo!("invalid"),
                },
                NumberState::Fraction(range) => todo!(),
                NumberState::FractionOrExponentOrEnd(range) => todo!(),
                NumberState::ExponentOrEnd(range) => todo!(),
                NumberState::End(_) => self,
            };

            Ok(res)
        }
    }

    /// ```abnf
    /// number        = [ minus ] int [ frac ] [ exp ]
    /// decimal-point = %x2E              ; .
    /// digit1-9      = %x31-39           ; 1-9
    /// e             = %x65 / %x45       ; e E
    /// exp           = e [ minus / plus ] 1*DIGIT
    /// frac          = decimal-point 1*DIGIT
    /// int           = zero / ( digit1-9 *DIGIT )
    /// minus         = %x2D              ; -
    /// plus          = %x2B              ; +
    /// zero          = %x30              ; 0
    /// ```
    /// See [RFC 8259 Section 6](https://datatracker.ietf.org/doc/html/rfc8259#section-6)
    pub fn parse_num<'a>(
        input: &'a str,
        chars: &mut Peekable<CharIndices<'a>>,
    ) -> Result<TokenWithContext> {
        let mut state = NumberState::MinusOrInteger;

        loop {
            state = state.process(chars, input)?;
            if let NumberState::End(tok) = state {
                break Ok(tok);
            }
        }

        // Does the RFC require whitespace after nums?
        // can start with - optionally
        // can't start with extra leading 0

        // exponent E or e
    }
}

fn parse_str<'a>(
    start: usize,
    input: &'a str,
    chars: &mut core::iter::Peekable<core::str::CharIndices<'a>>,
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
                ErrorKind::UnexpectedControlCharacterInString(c.into()),
                end..end + c.len_utf8(),
                input,
            ))
        }
    } else {
        Err(Error::from_unterminated(ErrorKind::ExpectedQuote, input))
    }
}

impl From<bool> for Token {
    fn from(value: bool) -> Self {
        Token::Boolean(value)
    }
}
// TODO macro for every num type
impl From<usize> for Token {
    fn from(value: usize) -> Self {
        Token::Number(value.to_string())
    }
}
impl From<i32> for Token {
    fn from(value: i32) -> Self {
        Token::Number(value.to_string())
    }
}
impl From<f64> for Token {
    fn from(value: f64) -> Self {
        Token::Number(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_single_key_object() {
        assert_eq!(
            str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust".into()),
                    range: 1..7
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 7..8
                },
                TokenWithContext {
                    token: Token::String("is a must".into()),
                    range: 9..20
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: 20..21
                }
            ]
        )
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", Token::Null)]
    #[case("true", Token::Boolean(true))]
    #[case("false", Token::Boolean(false))]
    #[case("\"burger\"", Token::String("burger".into()))]
    #[case(r#""\"burger\"""#, Token::String(r#"\"burger\""#.into()))]
    #[case(r#"0"#, 0.into())]
    #[case(r#"12389"#, 12389.into())]
    #[case(r#"-12389"#, (-12389).into())]
    // #[case(r#"5.8888"#, 5.888.into())]
    #[case(r#"-0"#, Token::Number("-0".into()))]
    // #[case(r#"-1e5"#, Token::Number("-1e5".into()))]
    // #[case(r#"-1.48e50"#, Token::Number("-1.48e50".into()))]
    fn primitive_template(#[case] json: &str, #[case] expected: Token) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: Token) {
        assert_eq!(
            str_to_tokens(json),
            Ok(vec![TokenWithContext {
                token: expected,
                range: 0..json.len()
            }])
        );
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: Token) {
        let json = format!(
            r#"{{
                "rust": {primitive}
            }}"#
        );
        assert_eq!(
            str_to_tokens(&json).unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust".into()),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: expected,
                    range: 26..(26 + primitive.len())
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: (json.len() - 1)..json.len()
                }
            ]
        )
    }

    fn json_to_json_and_error(
        json: &'static str,
        kind: ErrorKind,
        range: Option<Range<usize>>,
    ) -> (&'static str, Error) {
        let error = match range {
            Some(range) => Error::new(kind, range, json),
            None => Error::from_unterminated(kind, json),
        };
        (json, error)
    }

    #[rstest::rstest]
    #[case(json_to_json_and_error(
        "a",
        ErrorKind::UnexpectedCharacter('a'.into()),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(
        "n",
        ErrorKind::UnexpectedCharacter('n'.into()),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(r#""hi"#, ErrorKind::ExpectedQuote, None))]
    #[case(json_to_json_and_error(
        r#""
    
    ""#,
        ErrorKind::UnexpectedControlCharacterInString('\n'.into()),
        Some(1..2)
    ))]
    fn should_not_parse_invalid_syntax(#[case] (json, error): (&str, Error)) {
        assert_eq!(str_to_tokens(json), Err(error));
    }

    #[test]
    fn multiple_keys() {
        assert_eq!(
            str_to_tokens(
                r#"{
                "rust": "is a must",
                "name": "ferris"
            }"#
            )
            .unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust".into()),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: Token::String("is a must".into()),
                    range: 26..37
                },
                TokenWithContext {
                    token: Token::Comma,
                    range: 37..38
                },
                TokenWithContext {
                    token: Token::String("name".into()),
                    range: 55..61
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 61..62
                },
                TokenWithContext {
                    token: Token::String("ferris".into()),
                    range: 63..71
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: 84..85
                }
            ]
        );
    }
}
