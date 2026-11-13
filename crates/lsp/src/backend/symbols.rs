use std::range::Range as Span;

use jjpwrgem_parse::ast::{Document, Key, Value, quoted};
use tower_lsp_server::ls_types::{DocumentSymbol, SymbolKind};

use crate::range::{PositionEncoding, Positions};

const MAX_SYMBOLS: usize = 5000;
const MAX_DETAIL_CHARS: usize = 100;

pub(crate) fn document_symbols(
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
    value: &Value,
    remaining: &mut usize,
) -> Option<Vec<DocumentSymbol>> {
    matches!(value, Value::Object(..) | Value::Array(..)).then(|| {
        value
            .children()
            .map_while(|(key, child)| {
                *remaining = remaining.checked_sub(1)?;
                Some(source.symbol(key, child, map_children(source, child, remaining)))
            })
            .collect()
    })
}

impl Source<'_> {
    fn symbol(
        &self,
        key: Key,
        value: &Value,
        children: Option<Vec<DocumentSymbol>>,
    ) -> DocumentSymbol {
        let (name, selection) = match key {
            Key::Name(key) => (name(self.document.slice(key)), quoted(key)),
            Key::Index(index) => (index.to_string(), value.span()),
        };
        let full = Span::from(selection.start..value.span().end);
        let (kind, detail) = match value {
            Value::Object(..) => (SymbolKind::OBJECT, None),
            Value::Array(..) => (SymbolKind::ARRAY, None),
            Value::String(..) => (SymbolKind::STRING, Some(value.range())),
            Value::Number { .. } => (SymbolKind::NUMBER, Some(value.range())),
            Value::Boolean(..) => (SymbolKind::BOOLEAN, Some(value.range())),
            Value::Null(..) => (SymbolKind::NULL, Some(value.range())),
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
