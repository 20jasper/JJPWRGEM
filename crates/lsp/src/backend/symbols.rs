use std::{
    collections::{HashMap, VecDeque},
    range::Range as Span,
};

use jjpwrgem_parse::ast::{Document, Value};
use line_index::LineIndex;
use tower_lsp_server::ls_types::{DocumentSymbol, SymbolKind};

use crate::range::{PositionEncoding, span_to_range_indexed};

const MAX_SYMBOLS: usize = 5000;

pub(crate) fn document_symbols(
    document: &Document<String>,
    encoding: PositionEncoding,
) -> Vec<DocumentSymbol> {
    document_symbols_limited(document, encoding, MAX_SYMBOLS)
}

fn document_symbols_limited(
    document: &Document<String>,
    encoding: PositionEncoding,
    limit: usize,
) -> Vec<DocumentSymbol> {
    let symbols = Symbols {
        document,
        index: LineIndex::new(document.source()),
        encoding,
        kept: kept_children(document.root(), limit),
    };
    symbols.children(document.root()).unwrap_or_default()
}

fn child_values(value: &Value) -> Box<dyn Iterator<Item = &Value> + '_> {
    match value {
        Value::Object(_, entries) => Box::new(entries.iter().map(|(_, value)| value)),
        Value::Array(_, items) => Box::new(items.iter()),
        _ => Box::new(std::iter::empty()),
    }
}

fn kept_children(root: &Value, limit: usize) -> HashMap<*const Value, usize> {
    let mut kept = HashMap::new();
    let mut budget = limit;
    let mut queue = VecDeque::from([root]);
    while let Some(container) = queue.pop_front() {
        if budget == 0 {
            break;
        }
        let keep = child_values(container).count().min(budget);
        budget -= keep;
        kept.insert(std::ptr::from_ref(container), keep);
        queue.extend(child_values(container).take(keep));
    }
    kept
}

struct Symbols<'a> {
    document: &'a Document<String>,
    index: LineIndex,
    encoding: PositionEncoding,
    kept: HashMap<*const Value, usize>,
}

impl Symbols<'_> {
    fn children(&self, value: &Value) -> Option<Vec<DocumentSymbol>> {
        let keep = self
            .kept
            .get(&std::ptr::from_ref(value))
            .copied()
            .unwrap_or(0);
        match value {
            Value::Object(_, entries) => Some(
                entries
                    .iter()
                    .take(keep)
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
                    .take(keep)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn limited(limit: usize) -> Vec<DocumentSymbol> {
        let document = Document::parse(r#"{"a": [1, 2, 3], "b": [4, 5], "c": 6}"#.to_owned())
            .expect("valid json");
        document_symbols_limited(&document, PositionEncoding::Utf16, limit)
    }

    fn names(symbols: &[DocumentSymbol]) -> Vec<&str> {
        symbols.iter().map(|symbol| symbol.name.as_str()).collect()
    }

    fn child_names(symbol: &DocumentSymbol) -> Vec<&str> {
        names(symbol.children.as_deref().expect("expected a container"))
    }

    #[test]
    fn a_limit_below_the_top_level_truncates_it() {
        let symbols = limited(2);
        assert_eq!(names(&symbols), ["a", "b"]);
        assert_eq!(child_names(&symbols[0]), Vec::<&str>::new());
        assert_eq!(child_names(&symbols[1]), Vec::<&str>::new());
    }

    #[test]
    fn deeper_levels_fill_the_remaining_budget_in_document_order() {
        let symbols = limited(5);
        assert_eq!(names(&symbols), ["a", "b", "c"]);
        assert_eq!(child_names(&symbols[0]), ["0", "1"]);
        assert_eq!(child_names(&symbols[1]), Vec::<&str>::new());
        assert_eq!(symbols[2].children, None);
    }

    #[test]
    fn documents_under_the_limit_are_complete() {
        let symbols = limited(8);
        assert_eq!(names(&symbols), ["a", "b", "c"]);
        assert_eq!(child_names(&symbols[0]), ["0", "1", "2"]);
        assert_eq!(child_names(&symbols[1]), ["0", "1"]);
    }
}
