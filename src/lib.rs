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

        &input[start..end]
    }
}

mod error;

use error::{Error, Result};
use string::build_str_while;

enum State {
    Init,
    Object,
    End,
    Key,
    KeyEnd,
    ValueStart,
    Value,
}

pub fn parse(json: &str) -> Result<HashMap<String, String>> {
    let mut state = State::Init;

    if json.is_empty() {
        return Err(Error::Empty);
    }

    let mut key = None::<String>;
    let mut value = None::<String>;

    let mut chars = json.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match state {
            State::Init => match c {
                '{' => {
                    state = State::Object;
                }
                '}' => return Err(Error::Unmatched(c)),
                invalid => return Err(Error::Unrecognized(invalid)),
            },
            State::Object => match c {
                '}' => {
                    state = State::End;
                }
                '"' => {
                    state = State::Key;
                }
                invalid => return Err(Error::Unrecognized(invalid)),
            },
            State::Key => match c {
                '"' => state = State::KeyEnd,
                _ => {
                    key = Some(build_str_while(i, json, &mut chars).into());
                }
            },
            State::KeyEnd => match c {
                ':' => state = State::ValueStart,
                invalid => return Err(Error::Unrecognized(invalid)),
            },
            State::Value => match c {
                '"' => state = State::Object,
                _ => {
                    value = Some(build_str_while(i, json, &mut chars).into());
                }
            },
            State::ValueStart => match c {
                '"' => {
                    state = State::Value;
                }
                invalid => return Err(Error::Unrecognized(invalid)),
            },
            State::End => return Err(Error::CharacterAfterEnd(c)),
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
    fn unrecognized() {
        assert_eq!(parse("a").unwrap_err(), Error::Unrecognized('a'));
    }

    #[test]
    fn unmatched() {
        assert_eq!(parse("}").unwrap_err(), Error::Unmatched('}'));
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
        assert_eq!(parse("{}{").unwrap_err(), Error::CharacterAfterEnd('{'));
    }
}
