use std::collections::HashMap;

mod string {
    use core::{iter::Peekable, str::CharIndices};

    pub fn build_str_while<'a>(
        start: usize,
        input: &'a str,
        chars: &mut Peekable<CharIndices<'a>>,
    ) -> &'a str {
        let mut end = start;

        while let Some((i, c)) = chars.next_if(|(_, c)| *c != '"') {
            end = i + c.len_utf8();
        }
        chars.next();

        &input[start..end]
    }
}

mod tokens {
    use crate::string::build_str_while;
    use crate::{Error, Result};

    #[derive(Debug, PartialEq, Eq)]
    pub enum Token {
        OpenCurlyBracket,
        ClosedCurlyBracket,
        Colon,
        String(String),
    }

    pub fn str_to_tokens(s: &str) -> Result<Vec<Token>> {
        let mut chars = s.char_indices().peekable();

        let mut res = vec![];

        while let Some((i, c)) = chars.next() {
            if c.is_whitespace() {
                continue;
            }
            let val = match c {
                '{' => Token::OpenCurlyBracket,
                '}' => Token::ClosedCurlyBracket,
                ':' => Token::Colon,
                '"' => Token::String(build_str_while(i + 1, s, &mut chars).into()),
                invalid => return Err(Error::UnexpectedCharacter(invalid)),
            };
            res.push(val);
        }

        Ok(res)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_parse_single_key_object() {
            assert_eq!(
                str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
                [
                    Token::OpenCurlyBracket,
                    Token::String("rust".into()),
                    Token::Colon,
                    Token::String("is a must".into()),
                    Token::ClosedCurlyBracket,
                ]
            )
        }

        fn should_not_parse_invalid_syntax() {
            assert_eq!(str_to_tokens(r#"a"#), Err(Error::UnexpectedCharacter('a')));
        }
    }
}

mod error;

use error::{Error, Result};

use crate::tokens::{str_to_tokens, Token};

enum State {
    Init,
    Object,
    End,
    Key,
    Value,
}

pub fn parse(json: &str) -> Result<HashMap<String, String>> {
    let tokens = str_to_tokens(json)?;

    let mut state = State::Init;

    if tokens.is_empty() {
        return Err(Error::Empty);
    }

    let mut key = None::<String>;
    let mut value = None::<String>;

    for token in tokens {
        match state {
            State::Init => match token {
                Token::OpenCurlyBracket => {
                    state = State::Object;
                }
                Token::ClosedCurlyBracket => return Err(Error::Unmatched(token)),
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Object => match token {
                Token::ClosedCurlyBracket => {
                    state = State::End;
                }
                Token::String(s) => {
                    key = Some(s);
                    state = State::Key;
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Key => match token {
                Token::Colon => state = State::Value,
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Value => match token {
                Token::String(s) => {
                    value = Some(s);
                    state = State::Object;
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::End => return Err(Error::TokenAfterEnd(token)),
        }
    }

    let mut map = HashMap::new();

    if let (Some(k), Some(v)) = (key, value) {
        map.insert(k, v);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(parse("").unwrap_err(), Error::Empty);
    }

    #[test]
    fn unmatched() {
        assert_eq!(
            parse("}").unwrap_err(),
            Error::Unmatched(Token::ClosedCurlyBracket)
        );
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse("{}").unwrap(), HashMap::new());
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse(r#"{"hi":"bye"}"#).unwrap(),
            [("hi", "bye")]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect()
        );
    }

    #[test]
    fn key_with_braces() {
        assert_eq!(
            parse(r#"{"h{}{}i":"bye"}"#).unwrap(),
            [("h{}{}i", "bye")]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect()
        );
    }

    #[test]
    fn finished_object_then_another_char() {
        assert_eq!(
            parse("{}{").unwrap_err(),
            Error::TokenAfterEnd(Token::OpenCurlyBracket)
        );
    }
}
