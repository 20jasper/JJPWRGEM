use core::range::Range;

use crate::ast::{Document, Key, ObjectEntries, Value, quoted};

#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    source: &'a str,
    value: &'a Value,
}

#[derive(Debug, Clone, Copy)]
pub enum Kind<'a> {
    Null,
    Bool(bool),
    String(Str<'a>),
    Number(Number<'a>),
    Array(Array<'a>),
    Object(Object<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum PathKey<'a> {
    Name(Str<'a>),
    Index(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Str<'a> {
    source: &'a str,
    range: Range<usize>,
}

impl<'a> Str<'a> {
    pub fn span(self) -> Range<usize> {
        quoted(self.range)
    }

    pub fn raw_span(self) -> Range<usize> {
        self.range
    }

    pub fn raw(self) -> &'a str {
        &self.source[self.range]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Array<'a> {
    node: Node<'a>,
    items: &'a [Value],
}

impl<'a> Array<'a> {
    pub fn iter(self) -> Items<'a> {
        Items {
            source: self.node.source,
            iter: self.items.iter(),
        }
    }

    pub fn len(self) -> usize {
        self.items.len()
    }

    pub fn is_empty(self) -> bool {
        self.items.is_empty()
    }

    pub fn span(self) -> Range<usize> {
        self.node.span()
    }

    pub fn node(self) -> Node<'a> {
        self.node
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Object<'a> {
    node: Node<'a>,
    entries: &'a ObjectEntries,
}

impl<'a> Object<'a> {
    pub fn iter(self) -> Members<'a> {
        Members {
            source: self.node.source,
            iter: self.entries.0.iter(),
        }
    }

    pub fn get(self, name: &str) -> Option<Node<'a>> {
        self.iter()
            .find(|(key, _)| key.raw() == name)
            .map(|(_, node)| node)
    }

    pub fn span(self) -> Range<usize> {
        self.node.span()
    }

    pub fn node(self) -> Node<'a> {
        self.node
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Number<'a> {
    node: Node<'a>,
    mantissa: Range<usize>,
    exponent: Option<Range<usize>>,
}

impl<'a> Number<'a> {
    pub fn mantissa(self) -> &'a str {
        &self.node.source[self.mantissa]
    }

    pub fn exponent(self) -> Option<&'a str> {
        self.exponent.map(|e| &self.node.source[e])
    }

    pub fn integer(self) -> &'a str {
        let unsigned = self.mantissa().trim_start_matches('-');
        unsigned.split_once('.').map_or(unsigned, |(int, _)| int)
    }

    pub fn fraction(self) -> Option<&'a str> {
        self.mantissa().split_once('.').map(|(_, frac)| frac)
    }

    pub fn has_minus(self) -> bool {
        self.mantissa().starts_with('-')
    }

    pub fn span(self) -> Range<usize> {
        self.node.span()
    }

    pub fn node(self) -> Node<'a> {
        self.node
    }
}

#[derive(Debug, Clone)]
pub struct Items<'a> {
    source: &'a str,
    iter: core::slice::Iter<'a, Value>,
}

impl<'a> Iterator for Items<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        self.iter.next().map(|value| Node { source, value })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl DoubleEndedIterator for Items<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let source = self.source;
        self.iter.next_back().map(|value| Node { source, value })
    }
}

impl ExactSizeIterator for Items<'_> {}

impl<'a> IntoIterator for Array<'a> {
    type Item = Node<'a>;
    type IntoIter = Items<'a>;

    fn into_iter(self) -> Items<'a> {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Members<'a> {
    source: &'a str,
    iter: core::slice::Iter<'a, (Range<usize>, Value)>,
}

impl<'a> Iterator for Members<'a> {
    type Item = (Str<'a>, Node<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        self.iter.next().map(|(range, value)| {
            (
                Str {
                    source,
                    range: *range,
                },
                Node { source, value },
            )
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl DoubleEndedIterator for Members<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let source = self.source;
        self.iter.next_back().map(|(range, value)| {
            (
                Str {
                    source,
                    range: *range,
                },
                Node { source, value },
            )
        })
    }
}

impl ExactSizeIterator for Members<'_> {}

impl<'a> IntoIterator for Object<'a> {
    type Item = (Str<'a>, Node<'a>);
    type IntoIter = Members<'a>;

    fn into_iter(self) -> Members<'a> {
        self.iter()
    }
}

impl<S: AsRef<str>> Document<S> {
    pub fn root(&self) -> Node<'_> {
        Node {
            source: self.source(),
            value: &self.root,
        }
    }
}

impl<'a> Node<'a> {
    pub fn span(self) -> Range<usize> {
        self.value.span()
    }

    pub fn kind(self) -> Kind<'a> {
        let source = self.source;
        match self.value {
            Value::Null(_) => Kind::Null,
            Value::Boolean(_, b) => Kind::Bool(*b),
            Value::String(range) => Kind::String(Str {
                source,
                range: *range,
            }),
            Value::Number { mantissa, exponent } => Kind::Number(Number {
                node: self,
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

    pub fn as_str(self) -> Option<Str<'a>> {
        match self.kind() {
            Kind::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(self) -> Option<Number<'a>> {
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

    pub fn as_array(self) -> Option<Array<'a>> {
        match self.kind() {
            Kind::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(self) -> Option<Object<'a>> {
        match self.kind() {
            Kind::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn text(self) -> &'a str {
        &self.source[self.value.span()]
    }

    pub fn traverse(self) -> impl Iterator<Item = Node<'a>> {
        let source = self.source;
        self.value
            .traverse()
            .map(move |value| Node { source, value })
    }

    pub fn path_at(self, offset: usize) -> impl Iterator<Item = (PathKey<'a>, Node<'a>)> {
        let source = self.source;
        self.value.path_at(offset).map(move |(key, value)| {
            let key = match key {
                Key::Name(range) => PathKey::Name(Str { source, range }),
                Key::Index(index) => PathKey::Index(index),
            };
            (key, Node { source, value })
        })
    }

    pub fn member_span(self, key: PathKey<'a>) -> Range<usize> {
        let span = self.span();
        match key {
            PathKey::Name(name) => name.span().start..span.end,
            PathKey::Index(_) => span,
        }
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

    fn name(kind: &Kind<'_>) -> &'static str {
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
            .iter()
            .map(|node| name(&node.kind()))
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            ["null", "bool", "number", "string", "array", "object"]
        );
        assert_eq!(array.len(), 6);
        assert_eq!(array.span(), doc.root().span());
    }

    #[test]
    fn container_views_answer_span() {
        let doc = parse(r#"{"a": [1, 2], "b": {"c": 3}}"#);
        let object = doc.root().as_object().unwrap();
        assert_eq!(object.span(), object.node().span());
        let array = object.get("a").unwrap().as_array().unwrap();
        assert_eq!(doc.slice(array.span()), "[1, 2]");
        assert_eq!(array.span(), array.node().span());
        let inner = object.get("b").unwrap().as_object().unwrap();
        assert_eq!(doc.slice(inner.span()), r#"{"c": 3}"#);
    }

    #[test]
    fn one_kind_accessors() {
        let doc = parse(r#"["a", 1, true, [], {}]"#);
        let items = doc.root().as_array().unwrap().iter().collect::<Vec<_>>();
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
        let members = object.iter().collect::<Vec<_>>();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].0.raw(), "a");
        assert!(matches!(members[1].1.kind(), Kind::Bool(true)));

        let Kind::Array(array) = members[0].1.kind() else {
            panic!("member is an array");
        };
        let items = array.iter().collect::<Vec<_>>();
        assert!(matches!(
            items[1].kind(),
            Kind::Number(n) if n.mantissa() == "20" && n.exponent().is_none()
        ));
        assert_eq!(object.span(), doc.root().span());
        assert_eq!(object.iter().next_back().unwrap().0.raw(), "b");
    }

    #[test]
    fn str_raw_and_span() {
        let doc = parse(r#"["plain", "a\nbé"]"#);
        let Kind::Array(array) = doc.root().kind() else {
            panic!("root is an array");
        };
        let strs = array
            .iter()
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

    #[rstest]
    #[case("-12.50", "12", Some("50"), true)]
    #[case("7", "7", None, false)]
    #[case("-0", "0", None, true)]
    fn number_helpers(
        #[case] json: &str,
        #[case] integer: &str,
        #[case] fraction: Option<&str>,
        #[case] has_minus: bool,
    ) {
        let doc = parse(json);
        let Kind::Number(number) = doc.root().kind() else {
            panic!("number");
        };
        assert_eq!(number.integer(), integer);
        assert_eq!(number.fraction(), fraction);
        assert_eq!(number.has_minus(), has_minus);
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
