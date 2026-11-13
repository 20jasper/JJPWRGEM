use std::range::Range as Span;

use itertools::{Either, Itertools};
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
        .map(|&position| {
            let offset = positions.offset(document.source(), position);
            steps(document.root(), offset)
                .dedup()
                .fold(None, |parent, span| {
                    Some(SelectionRange {
                        range: positions.range(span),
                        parent: parent.map(Box::new),
                    })
                })
                .unwrap_or(SelectionRange {
                    range: Range::new(position, position),
                    parent: None,
                })
        })
        .collect()
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
    let text = matches!(value, Value::String(..)).then(|| value.range());
    std::iter::once(value.span())
        .chain(text.filter(|text| contains(*text, offset)))
        .chain(interior(value, offset))
}

fn member_steps(offset: usize, key: Key, value: &Value) -> impl Iterator<Item = Span<usize>> + '_ {
    let member = std::iter::once(value.member_span(key));
    match key {
        Key::Name(name) if contains(quoted(name), offset) => Either::Left(
            member
                .chain([quoted(name)])
                .chain(Some(name).filter(|name| contains(*name, offset))),
        ),
        _ => Either::Right(member.chain(value_steps(value, offset))),
    }
}

fn steps(root: &Value, offset: usize) -> impl Iterator<Item = Span<usize>> {
    std::iter::once(root.span())
        .filter(move |span| contains(*span, offset))
        .chain(interior(root, offset))
        .chain(
            root.path_at(offset)
                .flat_map(move |(key, value)| member_steps(offset, key, value)),
        )
}
