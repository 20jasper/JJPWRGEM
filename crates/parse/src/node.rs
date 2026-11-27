use core::range::Range;

use crate::ast::{Document, Key, ObjectEntries, Value, quoted};

/// A value together with the document it was parsed from.
#[derive(Debug)]
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
#[derive(Debug)]
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

impl<'a, S: AsRef<str>> Str<'a, S> {
    /// span including the surrounding quotes
    pub fn span(self) -> Range<usize> {
        quoted(self.range)
    }

    /// source text between the quotes, escapes untouched
    pub fn raw(self) -> &'a str {
        self.doc.slice(self.range)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

impl<'a, S: AsRef<str>> Object<'a, S> {
    pub fn members(self) -> impl DoubleEndedIterator<Item = (Str<'a, S>, Node<'a, S>)> {
        let doc = self.node.doc;
        self.entries
            .0
            .iter()
            .map(move |(range, value)| (Str { doc, range: *range }, Node { doc, value }))
    }

    /// member by raw name, escapes untouched
    pub fn get(self, name: &str) -> Option<Node<'a, S>> {
        self.members()
            .find(|(key, _)| key.raw() == name)
            .map(|(_, node)| node)
    }

    pub fn node(self) -> Node<'a, S> {
        self.node
    }
}

/// A number literal, sliced only when asked.
#[derive(Debug)]
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

    pub fn as_str(self) -> Option<Str<'a, S>> {
        match self.kind() {
            Kind::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(self) -> Option<Number<'a, S>> {
        match self.kind() {
            Kind::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        match self.kind() {
            Kind::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<Array<'a, S>> {
        match self.kind() {
            Kind::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(self) -> Option<Object<'a, S>> {
        match self.kind() {
            Kind::Object(o) => Some(o),
            _ => None,
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::SyntaxOptions;

    fn parse(json: &str) -> Document<&str> {
        Document::parse(json, SyntaxOptions::JSON).unwrap()
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
    fn one_kind_accessors() {
        let doc = parse(r#"["a", 1, true, [], {}]"#);
        let items = doc.root().as_array().unwrap().items().collect::<Vec<_>>();
        assert_eq!(items[0].as_str().unwrap().raw(), "a");
        assert!(items[1].as_str().is_none());
        assert_eq!(items[1].as_number().unwrap().mantissa(), "1");
        assert!(items[0].as_number().is_none());
        assert_eq!(items[2].as_bool(), Some(true));
        assert_eq!(items[1].as_bool(), None);
        assert!(items[3].as_array().is_some());
        assert!(items[4].as_array().is_none());
        assert!(items[4].as_object().is_some());
        assert!(items[3].as_object().is_none());
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
            Kind::Number(n) if n.mantissa() == "20" && n.exponent().is_none()
        ));
        assert_eq!(object.node().span(), doc.root().span());
        assert_eq!(object.members().next_back().unwrap().0.raw(), "b");
    }

    #[test]
    fn str_raw_and_span() {
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
        assert_eq!(strs[0].raw(), "plain");
        assert_eq!(strs[1].raw(), "a\\nbé");
        assert_eq!(doc.slice(strs[0].span()), "\"plain\"");
    }

    #[test]
    fn get_matches_raw_names() {
        let doc = parse(r#"{"ab": 1, "x\"y": 2, "plain": "v"}"#);
        let Kind::Object(root) = doc.root().kind() else {
            panic!("root is an object");
        };
        let mantissa = |name| match root.get(name).unwrap().kind() {
            Kind::Number(n) => n.mantissa(),
            _ => panic!("number"),
        };
        assert_eq!(mantissa("ab"), "1");
        assert_eq!(mantissa("x\\\"y"), "2");
        assert!(matches!(root.get("plain").unwrap().kind(), Kind::String(_)));
        assert!(root.get("x\"y").is_none());
        assert!(root.get("missing").is_none());
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
        assert_eq!(number.mantissa(), mantissa);
        assert_eq!(number.exponent(), exponent);
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
