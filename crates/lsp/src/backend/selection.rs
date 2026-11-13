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
    let chain = steps(document.root(), offset)
        .into_iter()
        .map(|span| SelectionRange {
            range: positions.range(span),
            parent: None,
        })
        .reduce(|parent, inner| SelectionRange {
            range: inner.range,
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

fn push_interior(spans: &mut Vec<Span<usize>>, value: &Value, offset: usize) {
    if matches!(value, Value::Object(..) | Value::Array(..)) {
        let span = value.span();
        let inner = Span::from(span.start + BRACKET_LEN..span.end - BRACKET_LEN);
        if contains(inner, offset) {
            spans.push(inner);
        }
    }
}

fn push_value_steps(spans: &mut Vec<Span<usize>>, value: &Value, offset: usize) {
    spans.push(value.span());
    if let Value::String(range) = value
        && contains(*range, offset)
    {
        spans.push(*range);
    }
    push_interior(spans, value, offset);
}

fn steps(root: &Value, offset: usize) -> Vec<Span<usize>> {
    let mut spans = Vec::new();
    let root_span = root.span();
    if contains(root_span, offset) {
        spans.push(root_span);
    }
    push_interior(&mut spans, root, offset);

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
