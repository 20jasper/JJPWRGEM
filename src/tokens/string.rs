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
                None => return Err(Error::from_unterminated(ErrorKind::ExpectedQuote, input)),
            },
            StringState::Escape {
                string_range,
                quote_range,
            } => match chars.next() {
                Some(CharWithContext(r, c)) if c.can_be_escaped_directly() => {
                    StringState::CharOrEscapeOrEnd {
                        string_range: string_range.start..r.end,
                        quote_range,
                    }
                }
                Some(CharWithContext(r, JsonChar('u'))) => {
                    todo!()
                }
                _ => todo!(),
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
