//! selection ranges go from the current position to its container, to its container
//! until the cursor hits the document

use std::range::Range as ByteRange;

use gen_lsp_types::*;
use jjpwrgem_parse::ast::{Document, Key, Value, quoted};

use crate::{
    backend::Backend,
    range::{PositionEncoding, Positions},
    transport::Sink,
};

const BRACKET_LEN: usize = 1;

/// get all selection ranges for each cursor at `at`
pub(crate) fn selection_ranges(
    document: &Document<String>,
    encoding: PositionEncoding,
    at: &[Position],
) -> Vec<SelectionRange> {
    let positions = Positions::new(document.source(), encoding);
    at.iter()
        .map(|&position| selection_range(document, &positions, position))
        .collect()
}

/// get a single selection range
fn selection_range(
    document: &Document<String>,
    positions: &Positions,
    position: Position,
) -> SelectionRange {
    let offset = positions.offset(document.source(), position);
    // iterate outer to inner to track selection chain
    let chain = steps(document.root(), offset)
        .into_iter()
        .map(|span| SelectionRange {
            range: positions.range(span),
            parent: None,
        })
        .reduce(|parent, inner| SelectionRange {
            range: inner.range,
            // linked list of selection ranges
            // track parents
            parent: Some(Box::new(parent)),
        });
    chain.unwrap_or(SelectionRange {
        range: Range {
            start: position,
            end: position,
        },
        parent: None,
    })
}

fn contains_inclusive(span: ByteRange<usize>, offset: usize) -> bool {
    (span.start..=span.end).contains(&offset)
}

fn inside_container(value: &Value, offset: usize) -> Option<ByteRange<usize>> {
    let span = value.span();
    let inside_brackets = ByteRange::from(span.start + BRACKET_LEN..span.end - BRACKET_LEN);

    let offset_in_container = (value.is_container() || matches!(value, Value::String(..)))
        && contains_inclusive(inside_brackets, offset);
    offset_in_container.then_some(inside_brackets)
}

fn value_and_interior(value: &Value, offset: usize) -> impl Iterator<Item = ByteRange<usize>> {
    std::iter::once(value.span()).chain(inside_container(value, offset))
}

/// values from outer to inner
fn steps(root: &Value, offset: usize) -> Vec<ByteRange<usize>> {
    let mut spans = Vec::new();
    let root_span = root.span();
    // add whole document if offset is inside
    if contains_inclusive(root_span, offset) {
        spans.push(root_span);
    }
    spans.extend(inside_container(root, offset));

    // iterate outer to inner
    for (key, value) in root.path_at(offset) {
        match key {
            Key::Name(name) if contains_inclusive(quoted(name), offset) => {
                spans.push(value.member_span(key));
                spans.push(quoted(name));
                if contains_inclusive(name, offset) {
                    spans.push(name);
                }
            }
            Key::Name(_) => {
                spans.push(value.member_span(key));
                spans.extend(value_and_interior(value, offset));
            }
            Key::Index(_) => spans.extend(value_and_interior(value, offset)),
        }
    }

    spans.dedup();
    spans
}

impl<S: Sink> Backend<S> {
    pub(super) fn selection_range(
        &mut self,
        params: &SelectionRangeParams,
    ) -> Option<Vec<SelectionRange>> {
        self.with_document(&params.text_document.uri, |document| {
            selection_ranges(document, self.position_encoding, &params.positions)
        })
    }
}

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::SyntaxOptions;
    use rstest::rstest;

    use super::*;

    fn parse(text: &str) -> Document<String> {
        Document::parse(text.to_owned(), SyntaxOptions::JSON).expect("valid json")
    }

    #[rstest]
    #[case(r#"{"a":1}"#, 0, None)]
    #[case(r#"{"a":1}"#, 1, Some(1..6))]
    #[case(r#"{"a":1}"#, 7, None)]
    #[case("[1,2]", 4, Some(1..4))]
    #[case("{}", 1, Some(1..1))]
    #[case(r#""ab""#, 3, Some(1..3))]
    #[case("12", 1, None)]
    fn inside_container_for_root(
        #[case] text: &str,
        #[case] offset: usize,
        #[case] expected: Option<std::ops::Range<usize>>,
    ) {
        let document = parse(text);
        assert_eq!(
            inside_container(document.root(), offset),
            expected.map(ByteRange::from)
        );
    }

    #[test]
    fn selection_chain_for_nested_object_key() {
        let document = parse(r#"{"a":{"b":2}}"#);
        let selection =
            selection_ranges(&document, PositionEncoding::Utf16, &[Position::new(0, 7)]).remove(0);

        let chain: Vec<_> =
            std::iter::successors(Some(&selection), |selection| selection.parent.as_deref())
                .map(|selection| selection.range)
                .collect();

        let line_range = |start, end| Range::new(Position::new(0, start), Position::new(0, end));
        assert_eq!(
            chain,
            [
                line_range(7, 8),
                line_range(6, 9),
                line_range(6, 11),
                line_range(5, 12),
                line_range(1, 12),
                line_range(0, 13),
            ]
        );
    }
}
