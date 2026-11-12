use std::range::Range as Span;

use jjpwrgem_parse::ast::{Document, Value};
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
        let document = self.document;
        match value {
            Value::Object(_, entries) => {
                let mut symbols = Vec::new();
                for (key, value) in entries.iter() {
                    if self.remaining == 0 {
                        break;
                    }
                    self.remaining -= 1;
                    let selection = quoted(key);
                    symbols.push(self.symbol(
                        name(document.slice(key)),
                        selection,
                        Span::from(selection.start..span(value).end),
                        value,
                    ));
                }
                Some(symbols)
            }
            Value::Array(_, items) => {
                let mut symbols = Vec::new();
                for (index, value) in items.iter().enumerate() {
                    if self.remaining == 0 {
                        break;
                    }
                    self.remaining -= 1;
                    symbols.push(self.symbol(index.to_string(), span(value), span(value), value));
                }
                Some(symbols)
            }
            _ => None,
        }
    }

    fn symbol(
        &mut self,
        name: String,
        selection: Span<usize>,
        full: Span<usize>,
        value: &Value,
    ) -> DocumentSymbol {
        let (kind, detail) = match value {
            Value::Object(..) => (SymbolKind::OBJECT, None),
            Value::Array(..) => (SymbolKind::ARRAY, None),
            Value::String(range) => (SymbolKind::STRING, Some(*range)),
            Value::Number { .. } => (SymbolKind::NUMBER, Some(value.range())),
            Value::Boolean(range, _) => (SymbolKind::BOOLEAN, Some(*range)),
            Value::Null(range) => (SymbolKind::NULL, Some(*range)),
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

fn span(value: &Value) -> Span<usize> {
    match value {
        Value::String(range) => quoted(*range),
        _ => value.range(),
    }
}

fn quoted(range: Span<usize>) -> Span<usize> {
    Span::from(range.start - 1..range.end + 1)
}
