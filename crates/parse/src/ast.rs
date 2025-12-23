use crate::{Result, tokens::TokenStream, traverse::parse_tokens};
use std::borrow::Cow;
use visitor::AstVisitor;

#[derive(Debug, Clone, Default, Eq)]
pub struct ObjectEntries<'a>(pub Vec<(&'a str, Value<'a>)>);

impl<'a> ObjectEntries<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, k: &'a str, v: Value<'a>) {
        self.0.push((k, v));
    }

    pub fn get(&self, k: &'a str) -> Option<&Value<'a>> {
        self.0.iter().find_map(|(k2, v)| (k == *k2).then_some(v))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<Vec<(&'a str, Value<'a>)>> for ObjectEntries<'a> {
    fn from(value: Vec<(&'a str, Value<'a>)>) -> Self {
        ObjectEntries(value)
    }
}

impl<'a> PartialEq for ObjectEntries<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<'a> {
    Null,
    String(&'a str),
    Number(Cow<'a, str>),
    Object(ObjectEntries<'a>),
    Array(Vec<Value<'a>>),
    Boolean(bool),
}

pub fn parse_str<'a>(json: &'a str) -> Result<'a, Value<'a>> {
    let mut ast = AstVisitor::new();
    parse_tokens(&mut TokenStream::new(json), json, true, &mut ast)?;
    Ok(ast
        .finish()
        .expect("visitor should error if empty or unfinished"))
}

mod visitor {
    use crate::{
        ast::{ObjectEntries, Value},
        tokens::TokenWithContext,
        traverse::ParseVisitor,
    };
    use core::ops::Range;
    use std::borrow::Cow;

    #[derive(Debug, Default)]
    pub struct AstVisitor<'a> {
        stack: Vec<AstFrame<'a>>,
        result: Option<Value<'a>>,
    }

    #[derive(Debug)]
    enum AstFrame<'a> {
        Object {
            entries: ObjectEntries<'a>,
            current_key: Option<&'a str>,
        },
        Array {
            items: Vec<Value<'a>>,
        },
    }

    impl<'a> AstVisitor<'a> {
        pub fn new() -> Self {
            Self {
                stack: Vec::new(),
                result: None,
            }
        }

        fn emit_value(&mut self, value: Value<'a>) {
            match self.stack.last_mut() {
                // top-level value
                None => {
                    self.result = Some(value);
                }
                Some(frame) => match frame {
                    AstFrame::Array { items } => items.push(value),
                    AstFrame::Object {
                        entries,
                        current_key,
                    } => {
                        let k = current_key
                            .take()
                            .expect("the traverser should not emit a value before the key");
                        entries.push(k, value);
                    }
                },
            }
        }

        pub fn finish(self) -> Option<Value<'a>> {
            self.result
        }
    }

    impl<'a> ParseVisitor<'a> for AstVisitor<'a> {
        fn on_object_open(&mut self, _open_ctx: TokenWithContext<'a>) {
            self.stack.push(AstFrame::Object {
                entries: ObjectEntries::new(),
                current_key: None,
            });
        }

        fn on_object_key(&mut self, key: &'a str) {
            if let Some(AstFrame::Object { current_key, .. }) = self.stack.last_mut() {
                *current_key = Some(key);
            }
        }

        fn on_object_close(&mut self, _range: Range<usize>) {
            let frame = self
                .stack
                .pop()
                .expect("traverser will not emit unbalanced brackets");
            if let AstFrame::Object { entries, .. } = frame {
                self.emit_value(Value::Object(entries));
            }
        }

        fn on_array_open(&mut self, _open_ctx: TokenWithContext<'a>) {
            self.stack.push(AstFrame::Array { items: Vec::new() });
        }

        fn on_array_close(&mut self, _range: Range<usize>) {
            let frame = self
                .stack
                .pop()
                .expect("traverser will not emit unbalanced brackets");
            if let AstFrame::Array { items } = frame {
                self.emit_value(Value::Array(items));
            }
        }

        fn on_null(&mut self) {
            self.emit_value(Value::Null);
        }

        fn on_string(&mut self, s: &'a str) {
            self.emit_value(Value::String(s));
        }

        fn on_number(&mut self, n: Cow<'a, str>) {
            self.emit_value(Value::Number(n));
        }

        fn on_boolean(&mut self, b: bool) {
            self.emit_value(Value::Boolean(b));
        }

        fn on_object_key_val_delim(&mut self) {}
        fn on_item_delim(&mut self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv_to_map<'a>(tuples: &[(&'a str, Value<'a>)]) -> Value<'a> {
        Value::Object(tuples.to_vec().into())
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse_str("{}").unwrap(), kv_to_map(&[]));
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse_str(r#"{"hi":"bye"}"#).unwrap(),
            kv_to_map(&[("hi", Value::String("bye"))])
        );
    }

    #[test]
    fn nested_object() {
        let nested = |val| kv_to_map(&[("rust", val)]);
        assert_eq!(
            parse_str(
                r#"
                {
                    "rust": {
                        "rust": {
                            "rust": {
                                "rust": "rust"
                            }   
                        }   
                    }
                }        
            "#
            )
            .unwrap(),
            nested(nested(nested(nested(Value::String("rust")))))
        );
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", Value::Null)]
    #[case("true", Value::Boolean(true))]
    #[case("false", Value::Boolean(false))]
    #[case("\"burger\"", Value::String("burger".into()))]
    fn primitive_template(#[case] primitive: &str, #[case] expected: Value) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: Value) {
        assert_eq!(
            parse_str(&format!(
                r#"{{
                "rust": {primitive}
            }}"#
            ))
            .unwrap(),
            kv_to_map(&[("rust", expected)])
        )
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: Value) {
        assert_eq!(parse_str(json), Ok(expected));
    }
}
