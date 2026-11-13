//! Selection ranges go from the current position to its container, then to that container's
//! parent, until the cursor reaches the document.

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

/// Gets a single selection range.
fn selection_range(
    document: &Document<String>,
    positions: &Positions,
    position: Position,
) -> SelectionRange {
    let offset = positions.offset(document.source(), position);
    // Spans are outer-to-inner; each inner range links back to the previous outer range.
    let chain = steps(document.root(), offset)
        .into_iter()
        .map(|span| SelectionRange {
            range: positions.range(span),
            parent: None,
        })
        .reduce(|parent, inner| SelectionRange {
            range: inner.range,
            // Linked list of selection ranges, tracking parents.
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

fn push_value_steps(spans: &mut Vec<Span<usize>>, value: &Value, offset: usize) {
    spans.push(value.span());
    if let Value::String(range) = value
        && contains(*range, offset)
    {
        spans.push(*range);
    }
    spans.extend(interior(value, offset));
}

/// Returns selection spans from outer to inner.
fn steps(root: &Value, offset: usize) -> Vec<Span<usize>> {
    let mut spans = Vec::new();
    let root_span = root.span();
    // Add the whole document if the offset is inside it.
    if contains(root_span, offset) {
        spans.push(root_span);
    }
    spans.extend(interior(root, offset));

    // Iterate from the outermost matching child to the innermost value.
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
                push_value_steps(&mut spans, value, offset);
            }
            Key::Index(_) => push_value_steps(&mut spans, value, offset),
        }
    }

    spans.dedup();
    spans
}
