use jjpwrgem_parse::ast::{Document, Value};
use tower_lsp_server::ls_types::FoldingRange;

use crate::range::{PositionEncoding, Positions};

const MAX_FOLDING_RANGES: usize = 5000;

pub(crate) fn folding_ranges(
    document: &Document<String>,
    encoding: PositionEncoding,
) -> Vec<FoldingRange> {
    let positions = Positions::new(document.source(), encoding);
    let mut ranges: Vec<FoldingRange> = Vec::new();
    for value in document.root().traverse() {
        if ranges.len() == MAX_FOLDING_RANGES {
            break;
        }
        if !matches!(value, Value::Object(..) | Value::Array(..)) {
            continue;
        }
        let range = value.range();
        let start_line = positions.line(range.start);
        let close_line = positions.line(range.end - 1);
        if close_line > start_line + 1
            && ranges
                .last()
                .is_none_or(|last| last.start_line != start_line)
        {
            ranges.push(FoldingRange {
                start_line,
                end_line: close_line - 1,
                ..Default::default()
            });
        }
    }
    ranges
}
