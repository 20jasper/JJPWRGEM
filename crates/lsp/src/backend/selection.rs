//! selection ranges go from the current position to its container, to its container
//! until the cursor hits the document

use std::range::Range as Span;

use jjpwrgem_parse::ast::{Document, Key, Value, quoted};
use tower_lsp_server::ls_types::{Position, Range, SelectionRange};

use crate::range::{PositionEncoding, Positions};

const BRACKET_LEN: usize = 1;

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
        range: Range::new(position, position),
        parent: None,
    })
}

fn contains(span: Span<usize>, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}

fn interior(value: &Value, offset: usize) -> Option<Span<usize>> {
    let span = value.span();
    matches!(value, Value::Object(..) | Value::Array(..))
        .then(|| Span::from(span.start + BRACKET_LEN..span.end - BRACKET_LEN))
        .filter(|inner| contains(*inner, offset))
}

fn value_steps(value: &Value, offset: usize) -> impl Iterator<Item = Span<usize>> {
    let span = value.span();
    let text = matches!(value, Value::String(..)).then(|| value.range());
    std::iter::once(span)
        .chain(text.filter(move |range| contains(*range, offset)))
        .chain(interior(value, offset))
}

/// values from outer to inner
fn steps(root: &Value, offset: usize) -> Vec<Span<usize>> {
    let mut spans = Vec::new();
    let root_span = root.span();
    // add whole document if offset is inside
    if contains(root_span, offset) {
        spans.push(root_span);
    }
    spans.extend(interior(root, offset));

    // iterate outer to inner
    for (key, value) in root.path_at(offset) {
        match key {
            Key::Name(name) if contains(quoted(name), offset) => {
                spans.push(value.member_span(key));
                spans.push(quoted(name));
                if contains(name, offset) {
                    spans.push(name);
                }
            }
            Key::Name(_) => {
                spans.push(value.member_span(key));
                spans.extend(value_steps(value, offset));
            }
            Key::Index(_) => spans.extend(value_steps(value, offset)),
        }
    }

    spans.dedup();
    spans
}
