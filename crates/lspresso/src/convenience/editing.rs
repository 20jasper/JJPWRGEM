use gen_lsp_types::*;

use super::{at, document};
use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`textDocument/completion`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_completion).
    ///
    /// Use [`Protocol::protocol_completion`] for full params.
    pub fn completion(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<CompletionResponse>> {
        self.protocol_completion(CompletionParams {
            context: None,
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`completionItem/resolve`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#completionItem_resolve).
    ///
    /// Use [`Protocol::protocol_completion_item_resolve`] for full params.
    pub fn completion_item_resolve(&mut self, item: CompletionItem) -> Result<CompletionItem> {
        self.protocol_completion_item_resolve(item)
    }

    /// Sends [`textDocument/inlineCompletion`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_inlineCompletion).
    ///
    /// Use [`Protocol::protocol_inline_completion`] for full params.
    pub fn inline_completion(
        &mut self,
        params: InlineCompletionParams,
    ) -> Result<Option<InlineCompletionResponse>> {
        self.protocol_inline_completion(params)
    }

    /// Sends [`textDocument/signatureHelp`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_signatureHelp).
    ///
    /// Use [`Protocol::protocol_signature_help`] for full params.
    pub fn signature_help(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<SignatureHelp>> {
        self.protocol_signature_help(SignatureHelpParams {
            context: None,
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/codeAction`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_codeAction).
    ///
    /// Use [`Protocol::protocol_code_action`] for full params.
    pub fn code_action(
        &mut self,
        uri: &Uri,
        range: Range,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<Option<Vec<CodeActionResponse>>> {
        self.protocol_code_action(CodeActionParams {
            text_document: document(uri),
            range,
            context: CodeActionContext {
                diagnostics,
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`codeAction/resolve`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#codeAction_resolve).
    ///
    /// Use [`Protocol::protocol_code_action_resolve`] for full params.
    pub fn code_action_resolve(&mut self, action: CodeAction) -> Result<CodeAction> {
        self.protocol_code_action_resolve(action)
    }

    /// Sends [`textDocument/prepareRename`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_prepareRename).
    ///
    /// Use [`Protocol::protocol_prepare_rename`] for full params.
    pub fn prepare_rename(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<PrepareRenameResult>> {
        self.protocol_prepare_rename(PrepareRenameParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/rename`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_rename).
    ///
    /// Use [`Protocol::protocol_rename`] for full params.
    pub fn rename(
        &mut self,
        uri: &Uri,
        position: Position,
        new_name: impl Into<String>,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_rename(RenameParams {
            new_name: new_name.into(),
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/linkedEditingRange`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_linkedEditingRange).
    ///
    /// Use [`Protocol::protocol_linked_editing_range`] for full params.
    pub fn linked_editing_range(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<LinkedEditingRanges>> {
        self.protocol_linked_editing_range(LinkedEditingRangeParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/selectionRange`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_selectionRange).
    ///
    /// Use [`Protocol::protocol_selection_range`] for full params.
    pub fn selection_range(
        &mut self,
        uri: &Uri,
        positions: Vec<Position>,
    ) -> Result<Option<Vec<SelectionRange>>> {
        self.protocol_selection_range(SelectionRangeParams {
            text_document: document(uri),
            positions,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
}
