use std::range::Range as Span;

use gen_lsp_types::*;
use jjpwrgem_parse::{Kind, Node, Str, ast::Document};

use crate::{
    backend::Backend,
    range::{PositionEncoding, Positions},
    transport::Sink,
};

const MAX_SYMBOLS: usize = 5000;
const MAX_DETAIL_CHARS: usize = 100;

fn document_symbols(
    document: &Document<String>,
    encoding: PositionEncoding,
) -> Vec<DocumentSymbol> {
    let source = Source {
        document,
        positions: Positions::new(document.source(), encoding),
    };
    let mut remaining = MAX_SYMBOLS;
    map_children(&source, document.root_node(), &mut remaining).unwrap_or_default()
}

struct Source<'a> {
    document: &'a Document<String>,
    positions: Positions,
}

enum Member<'a> {
    Name(Str<'a, String>),
    Index(usize),
}

fn map_children(
    source: &Source<'_>,
    node: Node<'_, String>,
    remaining: &mut usize,
) -> Option<Vec<DocumentSymbol>> {
    let members: Box<dyn Iterator<Item = (Member<'_>, Node<'_, String>)>> = match node.kind() {
        Kind::Object(object) => Box::new(
            object
                .members()
                .map(|(key, child)| (Member::Name(key), child)),
        ),
        Kind::Array(array) => Box::new(
            array
                .items()
                .enumerate()
                .map(|(index, child)| (Member::Index(index), child)),
        ),
        _ => return None,
    };
    Some(
        members
            .map_while(|(member, child)| {
                *remaining = remaining.checked_sub(1)?;
                Some(source.symbol(&member, child, map_children(source, child, remaining)))
            })
            .collect(),
    )
}

impl Source<'_> {
    fn symbol(
        &self,
        member: &Member<'_>,
        node: Node<'_, String>,
        children: Option<Vec<DocumentSymbol>>,
    ) -> DocumentSymbol {
        let (name, selection) = match member {
            Member::Name(key) => (name(key.raw()), key.span()),
            Member::Index(index) => (index.to_string(), node.span()),
        };
        let full = Span::from(selection.start..node.span().end);
        // detail shows the raw source between the quotes, so it reads the unquoted range off the value
        let detail = Some(node.value().range());
        let (kind, detail) = match node.kind() {
            Kind::Object(_) => (SymbolKind::Object, None),
            Kind::Array(_) => (SymbolKind::Array, None),
            Kind::String(_) => (SymbolKind::String, detail),
            Kind::Number(_) => (SymbolKind::Number, detail),
            Kind::Bool(_) => (SymbolKind::Boolean, detail),
            Kind::Null => (SymbolKind::Null, detail),
        };
        #[expect(deprecated, reason = "the field is required to build the struct")]
        DocumentSymbol {
            name,
            detail: detail.map(|range| truncated(self.document.slice(range))),
            kind,
            tags: None,
            deprecated: None,
            range: self.positions.range(full),
            selection_range: self.positions.range(selection),
            children,
        }
    }
}

fn name(key: &str) -> String {
    if key.trim().is_empty() {
        format!("\"{key}\"")
    } else {
        key.to_owned()
    }
}

fn truncated(text: &str) -> String {
    match text.char_indices().nth(MAX_DETAIL_CHARS) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_owned(),
    }
}

impl<S: Sink> Backend<S> {
    pub(super) fn document_symbol(
        &mut self,
        params: &DocumentSymbolParams,
    ) -> Option<DocumentSymbolResponse> {
        self.with_document(&params.text_document.uri, |document| {
            DocumentSymbolResponse::DocumentSymbolList(document_symbols(
                document,
                self.position_encoding,
            ))
        })
    }
}
