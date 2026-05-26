use core::range::Range;

use visitor::AstVisitor;

use crate::{Result, tokens::TokenStream, traverse::parse_tokens};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectEntries(pub(crate) Vec<(Range<usize>, Value)>);

impl ObjectEntries {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn push(&mut self, key_range: Range<usize>, v: Value) {
        self.0.push((key_range, v));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn find<'a>(&'a self, source: &str, key: &str) -> Option<&'a Value> {
        self.0
            .iter()
            .find(|(kr, _)| &source[*kr] == key)
            .map(|(_, v)| v)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Null,
    /// excludes surrounding quotes
    String(Range<usize>),
    Number {
        mantissa: Range<usize>,
        exponent: Option<Range<usize>>,
    },
    Object(ObjectEntries),
    Array(Vec<Value>),
    Boolean(bool),
}

impl Value {
    pub(crate) fn to_f64(&self, source: &str) -> Option<f64> {
        let Value::Number { mantissa, exponent } = self else {
            return None;
        };
        let m = &source[*mantissa];
        if let Some(exponent) = exponent {
            let e = &source[*exponent];
            format!("{m}e{e}").parse().ok()
        } else {
            m.parse().ok()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<S> {
    pub(crate) source: S,
    pub(crate) root: Value,
}

impl<S: AsRef<str>> Document<S> {
    pub fn parse(source: S) -> Result<Self> {
        let root = {
            let text = source.as_ref();
            parse_at(text, 0..text.len())?.root
        };
        Ok(Self { source, root })
    }

    pub fn root(&self) -> &Value {
        &self.root
    }

    pub fn slice(&self, range: Range<usize>) -> &str {
        &self.source.as_ref()[range]
    }

    pub fn get_object_value<'a>(&self, entries: &'a ObjectEntries, key: &str) -> Option<&'a Value> {
        entries.find(self.source.as_ref(), key)
    }

    pub fn as_str<'a>(&'a self, value: &Value) -> Option<&'a str> {
        match value {
            Value::String(r) => Some(self.slice(*r)),
            _ => None,
        }
    }

    pub fn parse_f64(&self, value: &Value) -> Option<f64> {
        value.to_f64(self.source.as_ref())
    }
}

pub(crate) fn parse_at(full_source: &str, range: Range<usize>) -> Result<Document<&str>> {
    let mut ast = AstVisitor::new();
    parse_tokens(
        &mut TokenStream::new_at_range(full_source, range),
        full_source,
        true,
        &mut ast,
    )?;
    let root = ast
        .finish()
        .expect("visitor should error if empty or unfinished");
    Ok(Document {
        source: full_source,
        root,
    })
}

mod visitor {
    use core::range::Range;

    use crate::{
        ast::{ObjectEntries, Value},
        traverse::Visitor,
    };

    #[derive(Debug, Default)]
    pub struct AstVisitor {
        stack: Vec<AstFrame>,
        result: Option<Value>,
    }

    #[derive(Debug)]
    enum AstFrame {
        Object {
            entries: ObjectEntries,
            current_key: Option<Range<usize>>,
        },
        Array {
            items: Vec<Value>,
        },
    }

    impl AstVisitor {
        pub fn new() -> Self {
            Self {
                stack: Vec::new(),
                result: None,
            }
        }

        fn last_emitted_mut(&mut self) -> &mut Value {
            match self.stack.last_mut() {
                None => self.result.as_mut().expect("must have emitted a value"),
                Some(AstFrame::Array { items }) => items.last_mut().expect("must have item"),
                Some(AstFrame::Object { entries, .. }) => {
                    &mut entries.0.last_mut().expect("must have entry").1
                }
            }
        }

        fn emit_value(&mut self, value: Value) {
            match self.stack.last_mut() {
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

        pub fn finish(self) -> Option<Value> {
            self.result
        }
    }

    impl<'a> Visitor<'a> for AstVisitor {
        fn on_object_open(&mut self) {
            self.stack.push(AstFrame::Object {
                entries: ObjectEntries::new(),
                current_key: None,
            });
        }

        fn on_object_key(&mut self, range: Range<usize>, _key: &'a str) {
            if let Some(AstFrame::Object { current_key, .. }) = self.stack.last_mut() {
                *current_key = Some(range);
            } else {
                unreachable!("must be in object for object key")
            }
        }

        fn on_object_close(&mut self) {
            let frame = self
                .stack
                .pop()
                .expect("traverser will not emit unbalanced brackets");
            if let AstFrame::Object { entries, .. } = frame {
                self.emit_value(Value::Object(entries));
            } else {
                unreachable!("must be an object to close object")
            }
        }

        fn on_array_open(&mut self) {
            self.stack.push(AstFrame::Array { items: Vec::new() });
        }

        fn on_array_close(&mut self) {
            let frame = self
                .stack
                .pop()
                .expect("traverser will not emit unbalanced brackets");
            if let AstFrame::Array { items } = frame {
                self.emit_value(Value::Array(items));
            } else {
                unreachable!("must be an array to close array")
            }
        }

        fn on_null(&mut self) {
            self.emit_value(Value::Null);
        }

        fn on_string(&mut self, range: Range<usize>, _s: &'a str) {
            self.emit_value(Value::String(range));
        }

        fn on_mantissa(&mut self, range: Range<usize>, _mantissa: &'a str) {
            self.emit_value(Value::Number {
                mantissa: range,
                exponent: None,
            });
        }

        fn on_exponent(&mut self, range: Range<usize>, _exponent: &'a str) {
            let Value::Number { exponent, .. } = self.last_emitted_mut() else {
                unreachable!("exponent must follow mantissa")
            };
            *exponent = Some(range);
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

    fn range_of(source: &str, needle: &str) -> Range<usize> {
        let start = source.find(needle).expect("needle in source");
        start..start + needle.len()
    }

    fn make_obj(entries: Vec<(&str, Value)>, source: &str) -> Value {
        let entries = entries
            .into_iter()
            .map(|(k, v)| (range_of(source, k), v))
            .collect::<Vec<_>>();
        Value::Object(ObjectEntries(entries))
    }

    #[test]
    fn empty_object() {
        let doc = Document::parse("{}").unwrap();
        assert_eq!(doc.root(), &Value::Object(ObjectEntries::new()));
    }

    #[test]
    fn one_key_value_pair() {
        let json = r#"{"hi":"bye"}"#;
        let doc = Document::parse(json).unwrap();
        assert_eq!(
            doc.root(),
            &make_obj(vec![("hi", Value::String(range_of(json, "bye")))], json)
        );
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", |_: &str| Value::Null)]
    #[case("true", |_: &str| Value::Boolean(true))]
    #[case("false", |_: &str| Value::Boolean(false))]
    #[case("\"burger\"", |s: &str| Value::String(range_of(s, "burger")))]
    fn primitive_template(#[case] primitive: &str, #[case] expected: fn(&str) -> Value) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: fn(&str) -> Value) {
        let json = format!(
            r#"{{
                "rust": {primitive}
            }}"#
        );
        let doc = Document::parse(&json).unwrap();
        let want = make_obj(vec![("rust", expected(&json))], &json);
        assert_eq!(doc.root(), &want);
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: fn(&str) -> Value) {
        let doc = Document::parse(json).unwrap();
        assert_eq!(doc.root(), &expected(json));
    }
}
