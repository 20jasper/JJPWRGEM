use std::range::Range as Span;

use gen_lsp_types::*;
use jjpwrgem_parse::{Kind, Node, ast::Document};

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
    map_children(&source, document.root(), &mut remaining).unwrap_or_default()
}

struct Source<'a> {
    document: &'a Document<String>,
    positions: Positions,
}

fn map_children(
    source: &Source<'_>,
    node: Node<'_, String>,
    remaining: &mut usize,
) -> Option<Vec<DocumentSymbol>> {
    match node.kind() {
        Kind::Object(object) => Some(
            object
                .members()
                .map_while(|(key, child)| {
                    source.child(name(key.raw()), key.span(), child, remaining)
                })
                .collect(),
        ),
        Kind::Array(array) => Some(
            array
                .items()
                .enumerate()
                .map_while(|(index, child)| {
                    source.child(index.to_string(), child.span(), child, remaining)
                })
                .collect(),
        ),
        _ => None,
    }
}

impl Source<'_> {
    /// Returns `None` once the symbol cap is exhausted, which ends the caller's iteration.
    fn child(
        &self,
        name: String,
        selection: Span<usize>,
        node: Node<'_, String>,
        remaining: &mut usize,
    ) -> Option<DocumentSymbol> {
        *remaining = remaining.checked_sub(1)?;
        let children = map_children(self, node, remaining);
        Some(self.symbol(name, selection, node, children))
    }

    fn symbol(
        &self,
        name: String,
        selection: Span<usize>,
        node: Node<'_, String>,
        children: Option<Vec<DocumentSymbol>>,
    ) -> DocumentSymbol {
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
