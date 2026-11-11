use std::range::Range as Span;

use jjpwrgem_parse::ast::{Document, Value};
use line_index::LineIndex;
use tower_lsp_server::ls_types::{DocumentSymbol, SymbolKind};

use crate::range::{PositionEncoding, span_to_range_indexed};

pub(crate) fn document_symbols(
    document: &Document<String>,
    encoding: PositionEncoding,
) -> Vec<DocumentSymbol> {
    let symbols = Symbols {
        document,
        index: LineIndex::new(document.source()),
        encoding,
    };
    symbols.children(document.root()).unwrap_or_default()
}

struct Symbols<'a> {
    document: &'a Document<String>,
    index: LineIndex,
    encoding: PositionEncoding,
}

impl Symbols<'_> {
    fn children(&self, value: &Value) -> Option<Vec<DocumentSymbol>> {
        match value {
            Value::Object(_, entries) => Some(
                entries
                    .iter()
                    .map(|(key, value)| {
                        let selection = quoted(key);
                        self.symbol(
                            name(self.document.slice(key)),
                            selection,
                            Span::from(selection.start..span(value).end),
                            value,
                        )
                    })
                    .collect(),
            ),
            Value::Array(_, items) => Some(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        self.symbol(index.to_string(), span(value), span(value), value)
                    })
                    .collect(),
            ),
            _ => None,
        }
    }

    fn symbol(
        &self,
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
            detail: detail.map(|range| self.document.slice(range).to_owned()),
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

fn span(value: &Value) -> Span<usize> {
    match value {
        Value::String(range) => quoted(*range),
        _ => value.range(),
    }
}

fn quoted(range: Span<usize>) -> Span<usize> {
    Span::from(range.start - 1..range.end + 1)
}
