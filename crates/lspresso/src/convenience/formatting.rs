use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`textDocument/formatting`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_formatting).
    ///
    /// Use [`Protocol::protocol_formatting`] for full params.
    pub fn formatting(
        &mut self,
        uri: &Uri,
        options: FormattingOptions,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.protocol_formatting(DocumentFormattingParams {
            text_document: document(uri),
            options,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/rangeFormatting`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_rangeFormatting).
    ///
    /// Use [`Protocol::protocol_range_formatting`] for full params.
    pub fn range_formatting(
        &mut self,
        uri: &Uri,
        range: Range,
        options: FormattingOptions,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.protocol_range_formatting(DocumentRangeFormattingParams {
            text_document: document(uri),
            range,
            options,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/rangesFormatting`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_rangeFormatting).
    ///
    /// Use [`Protocol::protocol_ranges_formatting`] for full params.
    pub fn ranges_formatting(
        &mut self,
        uri: &Uri,
        ranges: Vec<Range>,
        options: FormattingOptions,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.protocol_ranges_formatting(DocumentRangesFormattingParams {
            text_document: document(uri),
            ranges,
            options,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/onTypeFormatting`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_onTypeFormatting).
    ///
    /// Use [`Protocol::protocol_on_type_formatting`] for full params.
    pub fn on_type_formatting(
        &mut self,
        uri: &Uri,
        position: Position,
        ch: impl Into<String>,
        options: FormattingOptions,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.protocol_on_type_formatting(DocumentOnTypeFormattingParams {
            text_document: document(uri),
            position,
            ch: ch.into(),
            options,
        })
    }
}
