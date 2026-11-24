use gen_lsp_types::*;
use itertools::Itertools;
use jjpwrgem_parse::ast::{Document, Value};

use crate::{
    backend::Backend,
    range::{PositionEncoding, Positions},
    transport::Sink,
};

const CLOSING_BRACKET_LEN: usize = 1;
const MAX_FOLDING_RANGES: usize = 5000;

fn folding_ranges(document: &Document<String>, encoding: PositionEncoding) -> Vec<FoldingRange> {
    let positions = Positions::new(document.source(), encoding);
    document
        .root()
        .traverse()
        .filter(|value| matches!(value, Value::Object(..) | Value::Array(..)))
        .filter_map(|value| {
            let range = value.range();
            let start_line = positions.line(range.start);
            let close_line = positions.line(range.end - CLOSING_BRACKET_LEN);
            let end_line = close_line.checked_sub(1)?;
            (end_line > start_line).then_some((start_line, end_line))
        })
        .dedup_by(|a, b| a.0 == b.0)
        .take(MAX_FOLDING_RANGES)
        .map(|(start_line, end_line)| FoldingRange {
            start_line,
            end_line,
            ..Default::default()
        })
        .collect()
}

impl<S: Sink> Backend<S> {
    pub(super) fn folding_range(
        &mut self,
        params: &FoldingRangeParams,
    ) -> Option<Vec<FoldingRange>> {
        self.with_document(&params.text_document.uri, |document| {
            folding_ranges(document, self.position_encoding)
        })
    }
}
