//! selection ranges go from the current position to its container, to its container
//! until the cursor hits the document

use std::range::Range as ByteRange;

use gen_lsp_types::*;
use jjpwrgem_parse::{Kind, Node, PathKey, ast::Document};

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

fn inside_container(node: Node<'_, String>, offset: usize) -> Option<ByteRange<usize>> {
    let span = node.span();
    let inside_brackets = ByteRange::from(span.start + BRACKET_LEN..span.end - BRACKET_LEN);

    let offset_in_container = matches!(
        node.kind(),
        Kind::Array(_) | Kind::Object(_) | Kind::String(_)
    ) && contains_inclusive(inside_brackets, offset);
    offset_in_container.then_some(inside_brackets)
}

fn value_and_interior(
    node: Node<'_, String>,
    offset: usize,
) -> impl Iterator<Item = ByteRange<usize>> {
    std::iter::once(node.span()).chain(inside_container(node, offset))
}

/// values from outer to inner
fn steps(root: Node<'_, String>, offset: usize) -> Vec<ByteRange<usize>> {
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
            PathKey::Name(name) if contains_inclusive(name.span(), offset) => {
                spans.push(value.member_span(key));
                spans.push(name.span());
                if contains_inclusive(name.raw_span(), offset) {
                    spans.push(name.raw_span());
                }
            }
            PathKey::Name(_) => {
                spans.push(value.member_span(key));
                spans.extend(value_and_interior(value, offset));
            }
            PathKey::Index(_) => spans.extend(value_and_interior(value, offset)),
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

    fn range((sl, sc): (u32, u32), (el, ec): (u32, u32)) -> Range {
        Range::new(Position::new(sl, sc), Position::new(el, ec))
    }

    /// ranges from innermost to outermost at `position`
    fn chain_at(text: &str, position: Position) -> Vec<Range> {
        let document = parse(text);
        let selection = selection_ranges(&document, PositionEncoding::Utf16, &[position]).remove(0);
        std::iter::successors(Some(&selection), |selection| selection.parent.as_deref())
            .map(|selection| selection.range)
            .collect()
    }

    #[rstest]
    #[case(
        r#"{"a":{"b":2}}"#,
        (0, 7),
        vec![
            range((0, 7), (0, 8)),
            range((0, 6), (0, 9)),
            range((0, 6), (0, 11)),
            range((0, 5), (0, 12)),
            range((0, 1), (0, 12)),
            range((0, 0), (0, 13)),
        ],
    )]
    #[case(
        "{\n  \"a\": {\n    \"b\": 2\n  }\n}",
        (2, 5),
        vec![
            range((2, 5), (2, 6)),
            range((2, 4), (2, 7)),
            range((2, 4), (2, 10)),
            range((1, 8), (3, 2)),
            range((1, 7), (3, 3)),
            range((1, 2), (3, 3)),
            range((0, 1), (4, 0)),
            range((0, 0), (4, 1)),
        ],
    )]
    #[case(
        r#"[{"k": "ab", "z": 1}]"#,
        (0, 9),
        vec![
            range((0, 8), (0, 10)),
            range((0, 7), (0, 11)),
            range((0, 2), (0, 11)),
            range((0, 2), (0, 19)),
            range((0, 1), (0, 20)),
            range((0, 0), (0, 21)),
        ],
    )]
    #[case(
        "{\n  \"a\": [1, 2],\n  \"b\": 3\n}",
        (1, 10),
        vec![
            range((1, 8), (1, 12)),
            range((1, 7), (1, 13)),
            range((1, 2), (1, 13)),
            range((0, 1), (3, 0)),
            range((0, 0), (3, 1)),
        ],
    )]
    #[case(
        r#"[10, "x"]"#,
        (0, 6),
        vec![
            range((0, 6), (0, 7)),
            range((0, 5), (0, 8)),
            range((0, 1), (0, 8)),
            range((0, 0), (0, 9)),
        ],
    )]
    #[case(
        r#"[10, "x"]"#,
        (0, 1),
        vec![range((0, 1), (0, 3)), range((0, 1), (0, 8)), range((0, 0), (0, 9))],
    )]
    fn selection_chain(
        #[case] text: &str,
        #[case] (line, character): (u32, u32),
        #[case] expected: Vec<Range>,
    ) {
        assert_eq!(chain_at(text, Position::new(line, character)), expected);
    }
}
