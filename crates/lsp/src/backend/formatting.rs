use gen_lsp_types::*;
use jjpwrgem_parse::format::{JsonMode, format_document};

use crate::{backend::Backend, range::FULL_DOCUMENT_RANGE};

impl Backend {
    pub(super) fn formatting(&self, params: &DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let mode = JsonMode::from_settings(&self.format);
        self.with_document(&params.text_document.uri, |document| {
            vec![TextEdit {
                range: FULL_DOCUMENT_RANGE,
                new_text: format_document(document, &mode),
            }]
        })
    }
}
