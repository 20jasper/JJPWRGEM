use core::range::Range;
use std::borrow::Cow;

use crate::ast::{Document, Key, Value};

/// A value together with the document it was parsed from.
pub struct Node<'a, S> {
    doc: &'a Document<S>,
    value: &'a Value,
}

impl<S> Clone for Node<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Node<'_, S> {}

impl<S> core::fmt::Debug for Node<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Null,
    Bool,
    String,
    Number,
    Array,
    Object,
}

/// Source text of a number: the mantissa with its sign, and the exponent after `e` or `E`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberText<'a> {
    pub mantissa: &'a str,
    pub exponent: Option<&'a str>,
}

impl<S> Document<S> {
    pub fn root_node(&self) -> Node<'_, S> {
        Node {
            doc: self,
            value: &self.root,
        }
    }
}

impl<'a, S: AsRef<str>> Node<'a, S> {
    pub fn span(self) -> Range<usize> {
        self.value.span()
    }

    pub fn kind(self) -> Kind {
        match self.value {
            Value::Null(_) => Kind::Null,
            Value::Boolean(..) => Kind::Bool,
            Value::String(_) => Kind::String,
            Value::Number { .. } => Kind::Number,
            Value::Array(..) => Kind::Array,
            Value::Object(..) => Kind::Object,
        }
    }

    /// escape hatch for code that needs raw ranges
    pub fn value(self) -> &'a Value {
        self.value
    }

    pub fn is_container(self) -> bool {
        self.value.is_container()
    }

    pub fn as_bool(self) -> Option<bool> {
        match self.value {
            Value::Boolean(_, b) => Some(*b),
            _ => None,
        }
    }

    /// unescaped text, `None` only when the node is not a string
    pub fn as_str(self) -> Option<Cow<'a, str>> {
        match self.value {
            Value::String(r) => Some(unescape(self.doc.slice(*r))),
            _ => None,
        }
    }

    /// `None` only when the node is not a number
    pub fn as_number(self) -> Option<NumberText<'a>> {
        match self.value {
            Value::Number { mantissa, exponent } => Some(NumberText {
                mantissa: self.doc.slice(*mantissa),
                exponent: exponent.map(|e| self.doc.slice(e)),
            }),
            _ => None,
        }
    }

    /// direct children in document order
    pub fn children(self) -> impl DoubleEndedIterator<Item = (Key, Node<'a, S>)> {
        let doc = self.doc;
        self.value
            .children()
            .map(move |(key, value)| (key, Node { doc, value }))
    }

    /// preorder walk over this node and its descendants
    pub fn traverse(self) -> impl Iterator<Item = Node<'a, S>> {
        let doc = self.doc;
        self.value.traverse().map(move |value| Node { doc, value })
    }

    /// the chain of descendants containing `offset`, outermost first
    pub fn path_at(self, offset: usize) -> impl Iterator<Item = (Key, Node<'a, S>)> {
        let doc = self.doc;
        self.value
            .path_at(offset)
            .map(move |(key, value)| (key, Node { doc, value }))
    }

    pub fn member_span(self, key: Key) -> Range<usize> {
        self.value.member_span(key)
    }

    /// object member by unescaped name
    pub fn get(self, name: &str) -> Option<Node<'a, S>> {
        let Value::Object(_, entries) = self.value else {
            return None;
        };
        entries
            .0
            .iter()
            .find(|(key, _)| unescape(self.doc.slice(*key)) == name)
            .map(|(_, value)| Node {
                doc: self.doc,
                value,
            })
    }

    /// unescaped member name, `None` only for array indices
    pub fn key_name(self, key: Key) -> Option<Cow<'a, str>> {
        match key {
            Key::Name(range) => Some(unescape(self.doc.slice(range))),
            Key::Index(_) => None,
        }
    }
}

/// Decodes JSON escapes in `raw`, borrowing when there are none.
/// Input from the parser is well formed, so the malformed cases are only handled deterministically:
/// an unknown escape is kept as written, and a bad `\u` or a lone surrogate becomes U+FFFD.
pub(crate) fn unescape(raw: &str) -> Cow<'_, str> {
    if !raw.contains('\\') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            None => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => out.push(unicode_escape(&mut chars)),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    Cow::Owned(out)
}

fn is_high_surrogate(unit: u32) -> bool {
    matches!(unit, 0xD800..=0xDBFF)
}

fn is_low_surrogate(unit: u32) -> bool {
    matches!(unit, 0xDC00..=0xDFFF)
}

/// Reads the digits after `\u`, joining a surrogate pair when a low surrogate escape follows.
fn unicode_escape(chars: &mut core::str::Chars<'_>) -> char {
    let Some(unit) = hex4(chars) else {
        return char::REPLACEMENT_CHARACTER;
    };
    if is_high_surrogate(unit) {
        let low = chars.as_str().strip_prefix("\\u").and_then(|rest| {
            let mut ahead = rest.chars();
            hex4(&mut ahead)
                .filter(|low| is_low_surrogate(*low))
                .map(|low| (low, ahead))
        });
        if let Some((low, ahead)) = low {
            *chars = ahead;
            let combined = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00);
            return char::from_u32(combined).unwrap_or(char::REPLACEMENT_CHARACTER);
        }
    }
    char::from_u32(unit).unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// Reads four hex digits and advances past them, consuming nothing when they are not there.
fn hex4(chars: &mut core::str::Chars<'_>) -> Option<u32> {
    let hex = chars.as_str().get(..4)?;
    let unit = hex
        .chars()
        .try_fold(0u32, |acc, c| Some(acc << 4 | c.to_digit(16)?))?;
    chars.nth(3);
    Some(unit)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::SyntaxOptions;

    fn parse(json: &str) -> Document<&str> {
        Document::parse(json, SyntaxOptions::JSON).unwrap()
    }

    #[test]
    fn plain_stays_borrowed() {
        assert!(matches!(unescape("abc"), Cow::Borrowed("abc")));
        assert!(matches!(unescape(""), Cow::Borrowed("")));
    }

    #[rstest]
    #[case(r#"\""#, "\"")]
    #[case(r"\\", "\\")]
    #[case(r"\/", "/")]
    #[case(r"\b", "\u{8}")]
    #[case(r"\f", "\u{c}")]
    #[case(r"\n", "\n")]
    #[case(r"\r", "\r")]
    #[case(r"\t", "\t")]
    #[case(r"ab", "ab")]
    #[case(r"é", "é")]
    #[case("é", "é")]
    #[case("", "")]
    #[case(r"😀", "😀")]
    #[case(r"😀", "😀")]
    #[case(r"\ud83d", "\u{FFFD}")]
    #[case(r"\ud83dA", "\u{FFFD}A")]
    #[case(r"\ud83d\n", "\u{FFFD}\n")]
    #[case(r"\ud83dA", "\u{FFFD}A")]
    #[case(r"\ude00", "\u{FFFD}")]
    #[case(r"\\u0041", "\\u0041")]
    #[case(r"\u0000", "\0")]
    #[case(r"Ab", "Ab")]
    #[case(r#"a\"b\\c"#, "a\"b\\c")]
    fn decodes(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(unescape(raw), expected);
    }

    #[rstest]
    #[case(r"abc\", r"abc\")]
    #[case(r"\x", r"\x")]
    #[case(r"\u12", "\u{FFFD}12")]
    #[case(r"\uzzzz", "\u{FFFD}zzzz")]
    #[case(r"\u+123", "\u{FFFD}+123")]
    #[case(r"\ud83d\uzzzz", "\u{FFFD}\u{FFFD}zzzz")]
    #[case(r"\ud83d\u12", "\u{FFFD}\u{FFFD}12")]
    #[case(r"\uéé", "\u{FFFD}éé")]
    #[case(r"\u123é", "\u{FFFD}123é")]
    #[case(r"\u00é", "\u{FFFD}00é")]
    fn malformed_input_never_panics(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(unescape(raw), expected);
    }

    #[test]
    fn kinds() {
        let doc = parse(r#"[null, true, 1, "a", [], {}]"#);
        let kinds = doc
            .root_node()
            .children()
            .map(|(_, node)| node.kind())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                Kind::Null,
                Kind::Bool,
                Kind::Number,
                Kind::String,
                Kind::Array,
                Kind::Object
            ]
        );
        assert_eq!(doc.root_node().kind(), Kind::Array);
    }

    #[test]
    fn children_navigate_with_keys() {
        let doc = parse(r#"{"a": [10, 20], "b": true}"#);
        let root = doc.root_node();
        let children = root.children().collect::<Vec<_>>();
        assert_eq!(children.len(), 2);
        assert!(matches!(children[0].0, Key::Name(_)));
        assert_eq!(children[1].1.as_bool(), Some(true));

        let items = children[0].1.children().collect::<Vec<_>>();
        assert_eq!(items[1].0, Key::Index(1));
        assert_eq!(
            items[1].1.as_number(),
            Some(NumberText {
                mantissa: "20",
                exponent: None
            })
        );
        assert_eq!(root.children().count(), 2);
        assert_eq!(items[0].1.children().count(), 0);
    }

    #[test]
    fn as_str_unescapes_and_borrows_when_plain() {
        let doc = parse(r#"["plain", "a\nbé", 1]"#);
        let items = doc
            .root_node()
            .children()
            .map(|(_, n)| n)
            .collect::<Vec<_>>();
        assert!(matches!(items[0].as_str(), Some(Cow::Borrowed("plain"))));
        assert_eq!(items[1].as_str().as_deref(), Some("a\nbé"));
        assert_eq!(items[2].as_str(), None);
    }

    #[test]
    fn get_matches_unescaped_names() {
        let doc = parse(r#"{"ab": 1, "x\"y": 2, "plain": "v"}"#);
        let root = doc.root_node();
        assert_eq!(
            root.get("ab").and_then(Node::as_number).unwrap().mantissa,
            "1"
        );
        assert_eq!(
            root.get("x\"y").and_then(Node::as_number).unwrap().mantissa,
            "2"
        );
        assert_eq!(root.get("plain").unwrap().as_str().as_deref(), Some("v"));
        assert!(root.get("a\\u0062").is_none());
        assert!(root.get("missing").is_none());
        assert!(root.get("plain").unwrap().get("x").is_none());
    }

    #[test]
    fn key_name_unescapes_member_names_only() {
        let doc = parse(r#"{"kA": [1]}"#);
        let root = doc.root_node();
        let (key, array) = root.children().next().unwrap();
        assert_eq!(root.key_name(key).as_deref(), Some("kA"));
        let (index, _) = array.children().next().unwrap();
        assert_eq!(array.key_name(index), None);
    }

    #[rstest]
    #[case("42", "42", None)]
    #[case("-7", "-7", None)]
    #[case("1.50", "1.50", None)]
    #[case("1.5e1", "1.5", Some("1"))]
    #[case("2E+10", "2", Some("10"))]
    #[case("9223372036854775808", "9223372036854775808", None)]
    fn as_number_text(#[case] json: &str, #[case] mantissa: &str, #[case] exponent: Option<&str>) {
        let doc = parse(json);
        assert_eq!(
            doc.root_node().as_number(),
            Some(NumberText { mantissa, exponent })
        );
    }

    #[test]
    fn accessors_are_none_for_other_kinds() {
        let doc = parse(r#""s""#);
        let node = doc.root_node();
        assert_eq!(node.as_bool(), None);
        assert_eq!(node.as_number(), None);
        assert_eq!(node.get("x").map(Node::kind), None);
    }

    #[test]
    fn path_at_and_span_follow_the_value() {
        let source = r#"{"a": [10, {"bb": 2}]}"#;
        let doc = parse(source);
        let path = doc
            .root_node()
            .path_at(source.find('2').unwrap())
            .map(|(_, node)| &source[node.span()])
            .collect::<Vec<_>>();
        assert_eq!(path, [r#"[10, {"bb": 2}]"#, r#"{"bb": 2}"#, "2"]);
    }
}
