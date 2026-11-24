use gen_lsp_types::*;
use jjpwrgem_parse::{
    SyntaxOptions,
    format::{JsonMode, format_document},
};

use crate::{backend::Backend, range::FULL_DOCUMENT_RANGE, transport::Sink};

impl<S: Sink> Backend<S> {
    pub(super) fn formatting(
        &mut self,
        params: &DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        let uri = &params.text_document.uri;
        if let Some(open) = self.documents.get_mut(uri)
            && open.syntax == SyntaxOptions::JSONC
        {
            if !open.warned_formatting {
                open.warned_formatting.toggle();
                self.show_warning("formatting jsonc is not supported yet");
            }
            return None;
        }
        let mode = JsonMode::from_settings(&self.format);
        self.with_document(uri, |document| {
            vec![TextEdit {
                range: FULL_DOCUMENT_RANGE,
                new_text: format_document(document, &mode),
            }]
        })
    }
}
