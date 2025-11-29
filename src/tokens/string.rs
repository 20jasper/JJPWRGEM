use crate::{
    Error, ErrorKind, Result,
    tokens::{CharWithContext, JsonChar, Token, TokenWithContext},
};
use core::ops::Range;
use std::iter::Peekable;

enum StringState {
    Open,
    // TODO track last escaped u escapes
    CharOrEscapeOrEnd {
        string_range: Range<usize>,
        quote_range: Range<usize>,
    },
    Escape {
        string_range: Range<usize>,
        quote_range: Range<usize>,
        slash_range: Range<usize>,
    },
    UEscape {
        string_range: Range<usize>,
        quote_range: Range<usize>,
        u_range: Range<usize>,
        slash_range: Range<usize>,
        digits: Vec<JsonChar>,
    },
    End(TokenWithContext),
}

impl StringState {
    fn process(
        self,
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
        input: &str,
    ) -> Result<Self> {
        let res = match self {
            StringState::Open => {
                let Some(CharWithContext(starting_quote, JsonChar('"'))) = chars.next() else {
                    unreachable!("must start with a quote");
                };

                StringState::CharOrEscapeOrEnd {
                    string_range: starting_quote.clone(),
                    quote_range: starting_quote.clone(),
                }
            }
            StringState::CharOrEscapeOrEnd {
                string_range,
                quote_range,
            } => match chars.next() {
                Some(CharWithContext(r, JsonChar('\\'))) => StringState::Escape {
                    string_range: string_range.start..r.end,
                    quote_range,
                    slash_range: r.clone(),
                },
                Some(CharWithContext(r, JsonChar('"'))) => StringState::End(TokenWithContext {
                    token: Token::String(input[quote_range.end..r.start].into()),
                    range: string_range.start..r.end,
                }),
                Some(CharWithContext(r, c)) if c.is_control() => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedControlCharacterInString(c),
                        r,
                        input,
                    ));
                }
                Some(CharWithContext(r, _)) => StringState::CharOrEscapeOrEnd {
                    string_range: string_range.start..r.end,
                    quote_range,
                },
                None => {
                    return Err(Error::from_unterminated(
                        ErrorKind::ExpectedQuote {
                            open_range: quote_range.clone(),
                            string_range: string_range.clone(),
                        },
                        input,
                    ));
                }
            },
            StringState::Escape {
                string_range,
                quote_range,
                slash_range,
            } => match chars.next() {
                Some(CharWithContext(r, c)) if c.can_be_escaped_directly() => {
                    StringState::CharOrEscapeOrEnd {
                        string_range: string_range.start..r.end,
                        quote_range,
                    }
                }
                Some(CharWithContext(r, JsonChar('u'))) => StringState::UEscape {
                    string_range,
                    quote_range,
                    u_range: r,
                    slash_range,
                    digits: vec![],
                },
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedEscape {
                            maybe_c: c,
                            slash_range: slash_range.clone(),
                            string_range: string_range.clone(),
                            quote_range: quote_range.clone(),
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            StringState::UEscape {
                string_range,
                quote_range,
                u_range,
                slash_range,
                mut digits,
            } => match chars.next() {
                Some(CharWithContext(r, c)) if c.is_hexdigit() => {
                    let string_range = string_range.start..r.end;
                    if digits.len() == 3 {
                        StringState::CharOrEscapeOrEnd {
                            string_range,
                            quote_range,
                        }
                    } else {
                        digits.push(c);
                        StringState::UEscape {
                            string_range,
                            quote_range,
                            u_range,
                            slash_range,
                            digits,
                        }
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedHexDigit {
                            quote_range: quote_range.clone(),
                            slash_range: slash_range.clone(),
                            u_range: u_range.clone(),
                            maybe_c: c,
                            digit_idx: digits.len() + 1,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            StringState::End(_) => self,
        };

        Ok(res)
    }
}

pub fn parse_string(
    input: &str,
    chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
) -> Result<TokenWithContext> {
    let mut state = StringState::Open;

    loop {
        state = state.process(chars, input)?;
        if let StringState::End(tok) = state {
            break Ok(tok);
        }
    }
}
