use core::range::Range;
use std::borrow::Cow;

use crate::{
    ast::{Document, Key, ObjectEntries, Value, quoted},
    tokens::lexical::{JsonByte, UNICODE_ESCAPE_DIGITS},
};

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

/// The kind of a node, carrying a typed view of its data.
#[derive(Debug)]
pub enum Kind<'a, S> {
    Null,
    Bool(bool),
    String(Str<'a, S>),
    Number(Number<'a, S>),
    Array(Array<'a, S>),
    Object(Object<'a, S>),
}

impl<S> Clone for Kind<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Kind<'_, S> {}

/// A string literal, decoded only when asked.
pub struct Str<'a, S> {
    doc: &'a Document<S>,
    range: Range<usize>,
}

impl<S> Clone for Str<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Str<'_, S> {}

impl<S> core::fmt::Debug for Str<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Str")
            .field("range", &self.range)
            .finish_non_exhaustive()
    }
}

impl<'a, S: AsRef<str>> Str<'a, S> {
    /// span including the surrounding quotes
    pub fn span(self) -> Range<usize> {
        quoted(self.range)
    }

    /// source text between the quotes, escapes untouched
    pub fn raw(self) -> &'a str {
        self.doc.slice(self.range)
    }

    pub fn unescape(self) -> Cow<'a, str> {
        unescape(self.raw())
    }
}

pub struct Array<'a, S> {
    node: Node<'a, S>,
    items: &'a [Value],
}

impl<S> Clone for Array<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Array<'_, S> {}

impl<S> core::fmt::Debug for Array<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Array")
            .field("items", &self.items)
            .finish_non_exhaustive()
    }
}

impl<'a, S: AsRef<str>> Array<'a, S> {
    pub fn items(self) -> impl DoubleEndedIterator<Item = Node<'a, S>> {
        let doc = self.node.doc;
        self.items.iter().map(move |value| Node { doc, value })
    }

    pub fn len(self) -> usize {
        self.items.len()
    }

    pub fn is_empty(self) -> bool {
        self.items.is_empty()
    }

    pub fn node(self) -> Node<'a, S> {
        self.node
    }
}

pub struct Object<'a, S> {
    node: Node<'a, S>,
    entries: &'a ObjectEntries,
}

impl<S> Clone for Object<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Object<'_, S> {}

impl<S> core::fmt::Debug for Object<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Object")
            .field("entries", &self.entries)
            .finish_non_exhaustive()
    }
}

impl<'a, S: AsRef<str>> Object<'a, S> {
    pub fn members(self) -> impl DoubleEndedIterator<Item = (Str<'a, S>, Node<'a, S>)> {
        let doc = self.node.doc;
        self.entries
            .0
            .iter()
            .map(move |(range, value)| (Str { doc, range: *range }, Node { doc, value }))
    }

    /// member by unescaped name
    pub fn get(self, name: &str) -> Option<Node<'a, S>> {
        self.members()
            .find(|(key, _)| key.unescape() == name)
            .map(|(_, node)| node)
    }

    pub fn node(self) -> Node<'a, S> {
        self.node
    }
}

/// Source text of a number: the mantissa with its sign, and the exponent after `e` or `E`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberText<'a> {
    mantissa: &'a str,
    exponent: Option<&'a str>,
}

impl<'a> NumberText<'a> {
    pub fn mantissa(self) -> &'a str {
        self.mantissa
    }

    pub fn exponent(self) -> Option<&'a str> {
        self.exponent
    }
}

/// A number literal, sliced only when asked.
pub struct Number<'a, S> {
    doc: &'a Document<S>,
    mantissa: Range<usize>,
    exponent: Option<Range<usize>>,
}

impl<S> Clone for Number<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Number<'_, S> {}

impl<S> core::fmt::Debug for Number<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Number")
            .field("mantissa", &self.mantissa)
            .field("exponent", &self.exponent)
            .finish_non_exhaustive()
    }
}

impl<'a, S: AsRef<str>> Number<'a, S> {
    /// mantissa with its sign
    pub fn mantissa(self) -> &'a str {
        self.doc.slice(self.mantissa)
    }

    /// exponent digits after `e` or `E`, with their sign
    pub fn exponent(self) -> Option<&'a str> {
        self.exponent.map(|e| self.doc.slice(e))
    }

    pub fn span(self) -> Range<usize> {
        let end = self.exponent.map_or(self.mantissa.end, |e| e.end);
        Range {
            start: self.mantissa.start,
            end,
        }
    }

    pub fn text(self) -> NumberText<'a> {
        NumberText {
            mantissa: self.mantissa(),
            exponent: self.exponent(),
        }
    }
}

impl<S> Document<S> {
    pub fn root(&self) -> Node<'_, S> {
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

    pub fn kind(self) -> Kind<'a, S> {
        let doc = self.doc;
        match self.value {
            Value::Null(_) => Kind::Null,
            Value::Boolean(_, b) => Kind::Bool(*b),
            Value::String(range) => Kind::String(Str { doc, range: *range }),
            Value::Number { mantissa, exponent } => Kind::Number(Number {
                doc,
                mantissa: *mantissa,
                exponent: *exponent,
            }),
            Value::Array(_, items) => Kind::Array(Array { node: self, items }),
            Value::Object(_, entries) => Kind::Object(Object {
                node: self,
                entries,
            }),
        }
    }

    /// source text of the whole node, including string quotes
    pub fn text(self) -> &'a str {
        self.doc.slice(self.value.span())
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
}

/// Decodes JSON escapes in `raw`, borrowing when there are none.
/// Input from the parser is well formed, so the malformed cases are only handled deterministically:
/// an unknown escape stands for itself, and a bad `\u` or a lone surrogate becomes U+FFFD.
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
        let Some(escape) = chars.next() else { break };
        if escape == 'u' {
            out.push(unicode_escape(&mut chars));
        } else {
            out.push(
                u8::try_from(escape)
                    .ok()
                    .and_then(|b| JsonByte(b).unescaped())
                    .unwrap_or(escape),
            );
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
    let hex = chars.as_str().get(..UNICODE_ESCAPE_DIGITS)?;
    let unit = hex
        .chars()
        .try_fold(0u32, |acc, c| Some(acc << 4 | c.to_digit(16)?))?;
    chars.nth(UNICODE_ESCAPE_DIGITS - 1);
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
    #[case("", "")]
    #[case(r"😀", "😀")]
    #[case(r"\ud83d", "\u{FFFD}")]
    #[case(r"\ud83dA", "\u{FFFD}A")]
    #[case(r"\ud83d\n", "\u{FFFD}\n")]
    #[case(r"\ude00", "\u{FFFD}")]
    #[case(r"\\u0041", "\\u0041")]
    #[case(r"\u0000", "\0")]
    #[case(r"Ab", "Ab")]
    #[case(r#"a\"b\\c"#, "a\"b\\c")]
    fn decodes(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(unescape(raw), expected);
    }

    fn name(kind: &Kind<'_, &str>) -> &'static str {
        match kind {
            Kind::Null => "null",
            Kind::Bool(_) => "bool",
            Kind::String(_) => "string",
            Kind::Number(_) => "number",
            Kind::Array(_) => "array",
            Kind::Object(_) => "object",
        }
    }

    #[test]
    fn kinds() {
        let doc = parse(r#"[null, true, 1, "a", [], {}]"#);
        let Kind::Array(array) = doc.root().kind() else {
            panic!("root is an array");
        };
        let kinds = array
            .items()
            .map(|node| name(&node.kind()))
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            ["null", "bool", "number", "string", "array", "object"]
        );
        assert_eq!(array.len(), 6);
        assert_eq!(array.node().span(), doc.root().span());
    }

    #[test]
    fn members_and_items_navigate() {
        let doc = parse(r#"{"a": [10, 20], "b": true}"#);
        let Kind::Object(object) = doc.root().kind() else {
            panic!("root is an object");
        };
        let members = object.members().collect::<Vec<_>>();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].0.raw(), "a");
        assert!(matches!(members[1].1.kind(), Kind::Bool(true)));

        let Kind::Array(array) = members[0].1.kind() else {
            panic!("member is an array");
        };
        let items = array.items().collect::<Vec<_>>();
        assert!(matches!(
            items[1].kind(),
            Kind::Number(n) if n.text().mantissa() == "20" && n.text().exponent().is_none()
        ));
        assert_eq!(object.node().span(), doc.root().span());
        assert_eq!(object.members().next_back().unwrap().0.raw(), "b");
    }

    #[test]
    fn str_unescapes_and_borrows_when_plain() {
        let doc = parse(r#"["plain", "a\nbé"]"#);
        let Kind::Array(array) = doc.root().kind() else {
            panic!("root is an array");
        };
        let strs = array
            .items()
            .map(|node| match node.kind() {
                Kind::String(s) => s,
                _ => panic!("string"),
            })
            .collect::<Vec<_>>();
        assert!(matches!(strs[0].unescape(), Cow::Borrowed("plain")));
        assert_eq!(strs[1].unescape(), "a\nbé");
        assert_eq!(strs[1].raw(), "a\\nbé");
        assert_eq!(doc.slice(strs[0].span()), "\"plain\"");
    }

    #[test]
    fn get_matches_unescaped_names() {
        let doc = parse(r#"{"ab": 1, "x\"y": 2, "plain": "v"}"#);
        let Kind::Object(root) = doc.root().kind() else {
            panic!("root is an object");
        };
        let mantissa = |name| match root.get(name).unwrap().kind() {
            Kind::Number(n) => n.mantissa(),
            _ => panic!("number"),
        };
        assert_eq!(mantissa("ab"), "1");
        assert_eq!(mantissa("x\"y"), "2");
        assert!(matches!(root.get("plain").unwrap().kind(), Kind::String(_)));
        assert!(root.get("a\\u0062").is_none());
        assert!(root.get("missing").is_none());
    }

    #[test]
    fn member_names_unescape() {
        let doc = parse(r#"{"k\u0041": 1}"#);
        let Kind::Object(root) = doc.root().kind() else {
            panic!("root is an object");
        };
        let (key, _) = root.members().next().unwrap();
        assert_eq!(key.unescape(), "kA");
        assert_eq!(key.raw(), "k\\u0041");
    }

    #[rstest]
    #[case("42", "42", None)]
    #[case("-7", "-7", None)]
    #[case("1.50", "1.50", None)]
    #[case("1.5e1", "1.5", Some("1"))]
    #[case("2E+10", "2", Some("10"))]
    #[case("9223372036854775808", "9223372036854775808", None)]
    fn number_text(#[case] json: &str, #[case] mantissa: &str, #[case] exponent: Option<&str>) {
        let doc = parse(json);
        let Kind::Number(number) = doc.root().kind() else {
            panic!("number");
        };
        assert_eq!(number.text().mantissa(), mantissa);
        assert_eq!(number.text().exponent(), exponent);
        assert_eq!(&json[number.span()], json);
    }

    #[test]
    fn path_at_and_span_follow_the_value() {
        let source = r#"{"a": [10, {"bb": 2}]}"#;
        let doc = parse(source);
        let path = doc
            .root()
            .path_at(source.find('2').unwrap())
            .map(|(_, node)| &source[node.span()])
            .collect::<Vec<_>>();
        assert_eq!(path, [r#"[10, {"bb": 2}]"#, r#"{"bb": 2}"#, "2"]);
    }
}
