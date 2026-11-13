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

fn selection_range(
    document: &Document<String>,
    positions: &Positions,
    position: Position,
) -> SelectionRange {
    let offset = positions.offset(document.source(), position);
    let mut chain = None;
    for span in steps(document.root(), offset) {
        chain = Some(SelectionRange {
            range: positions.range(span),
            parent: chain.map(Box::new),
        });
    }
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

fn steps(root: &Value, offset: usize) -> Vec<Span<usize>> {
    let mut spans = Vec::new();
    if contains(root.span(), offset) {
        spans.push(root.span());
    }
    spans.extend(interior(root, offset));

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
