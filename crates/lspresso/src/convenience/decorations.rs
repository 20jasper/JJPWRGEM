use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Requests code lenses for `uri`.
    ///
    /// Use [`Protocol::protocol_code_lens`] for full params.
    pub fn code_lens(&mut self, uri: &Uri) -> Result<Option<Vec<CodeLens>>> {
        self.protocol_code_lens(CodeLensParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Resolves the rest of a code `lens`.
    pub fn code_lens_resolve(&mut self, lens: CodeLens) -> Result<CodeLens> {
        self.protocol_code_lens_resolve(lens)
    }

    /// Requests the colors in `uri`.
    ///
    /// Use [`Protocol::protocol_document_color`] for full params.
    pub fn document_color(&mut self, uri: &Uri) -> Result<Vec<ColorInformation>> {
        self.protocol_document_color(DocumentColorParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Requests presentations of `color` at `range` of `uri`.
    ///
    /// Use [`Protocol::protocol_color_presentation`] for full params.
    pub fn color_presentation(
        &mut self,
        uri: &Uri,
        color: Color,
        range: Range,
    ) -> Result<Vec<ColorPresentation>> {
        self.protocol_color_presentation(ColorPresentationParams {
            text_document: document(uri),
            color,
            range,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Requests the links in `uri`.
    ///
    /// Use [`Protocol::protocol_document_link`] for full params.
    pub fn document_link(&mut self, uri: &Uri) -> Result<Option<Vec<DocumentLink>>> {
        self.protocol_document_link(DocumentLinkParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Resolves the rest of a document `link`.
    pub fn document_link_resolve(&mut self, link: DocumentLink) -> Result<DocumentLink> {
        self.protocol_document_link_resolve(link)
    }

    /// Requests the folding ranges in `uri`.
    ///
    /// Use [`Protocol::protocol_folding_range`] for full params.
    pub fn folding_range(&mut self, uri: &Uri) -> Result<Option<Vec<FoldingRange>>> {
        self.protocol_folding_range(FoldingRangeParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Requests inlay hints for `range` of `uri`.
    ///
    /// Use [`Protocol::protocol_inlay_hint`] for full params.
    pub fn inlay_hint(&mut self, uri: &Uri, range: Range) -> Result<Option<Vec<InlayHint>>> {
        self.protocol_inlay_hint(InlayHintParams {
            text_document: document(uri),
            range,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Resolves the rest of an inlay `hint`.
    pub fn inlay_hint_resolve(&mut self, hint: InlayHint) -> Result<InlayHint> {
        self.protocol_inlay_hint_resolve(hint)
    }

    /// Requests inline values.
    pub fn inline_value(&mut self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        self.protocol_inline_value(params)
    }

    /// Requests semantic tokens for all of `uri`.
    ///
    /// Use [`Protocol::protocol_semantic_tokens_full`] for full params.
    pub fn semantic_tokens_full(&mut self, uri: &Uri) -> Result<Option<SemanticTokens>> {
        self.protocol_semantic_tokens_full(SemanticTokensParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Requests semantic token changes for `uri` since `previous_result_id`.
    ///
    /// Use [`Protocol::protocol_semantic_tokens_full_delta`] for full params.
    pub fn semantic_tokens_full_delta(
        &mut self,
        uri: &Uri,
        previous_result_id: impl Into<String>,
    ) -> Result<Option<SemanticTokensDeltaResponse>> {
        self.protocol_semantic_tokens_full_delta(SemanticTokensDeltaParams {
            text_document: document(uri),
            previous_result_id: previous_result_id.into(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Requests semantic tokens for `range` of `uri`.
    ///
    /// Use [`Protocol::protocol_semantic_tokens_range`] for full params.
    pub fn semantic_tokens_range(
        &mut self,
        uri: &Uri,
        range: Range,
    ) -> Result<Option<SemanticTokens>> {
        self.protocol_semantic_tokens_range(SemanticTokensRangeParams {
            text_document: document(uri),
            range,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
}
