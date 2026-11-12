use std::range::Range as Span;

use jjpwrgem_parse::ast::{Document, Key, Value};
use line_index::LineIndex;
use tower_lsp_server::ls_types::{DocumentSymbol, SymbolKind};

use crate::range::{PositionEncoding, span_to_range_indexed};

const MAX_SYMBOLS: usize = 5000;
const MAX_DETAIL_CHARS: usize = 100;

pub(crate) fn document_symbols(
    document: &Document<String>,
    encoding: PositionEncoding,
) -> Vec<DocumentSymbol> {
    let mut symbols = Symbols {
        document,
        index: LineIndex::new(document.source()),
        encoding,
        remaining: MAX_SYMBOLS,
    };
    symbols.children(document.root()).unwrap_or_default()
}

struct Symbols<'a> {
    document: &'a Document<String>,
    index: LineIndex,
    encoding: PositionEncoding,
    remaining: usize,
}

impl Symbols<'_> {
    fn children(&mut self, value: &Value) -> Option<Vec<DocumentSymbol>> {
        matches!(value, Value::Object(..) | Value::Array(..)).then(|| {
            value
                .children()
                .map_while(|(key, child)| {
                    self.remaining = self.remaining.checked_sub(1)?;
                    Some(self.symbol(key, child))
                })
                .collect()
        })
    }

    fn symbol(&mut self, key: Key, value: &Value) -> DocumentSymbol {
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
            range: span_to_range_indexed(&self.index, full, self.encoding),
            selection_range: span_to_range_indexed(&self.index, selection, self.encoding),
            children: self.children(value),
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

fn quoted(range: Span<usize>) -> Span<usize> {
    Span::from(range.start - 1..range.end + 1)
}
