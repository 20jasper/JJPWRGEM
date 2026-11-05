use gen_lsp_types::*;

use super::at;
use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`textDocument/declaration`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_declaration).
    ///
    /// Use [`Protocol::protocol_declaration`] for full params.
    pub fn declaration(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<DeclarationResponse>> {
        self.protocol_declaration(DeclarationParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/definition`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_definition).
    ///
    /// Use [`Protocol::protocol_definition`] for full params.
    pub fn definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<DefinitionResponse>> {
        self.protocol_definition(DefinitionParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/typeDefinition`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_typeDefinition).
    ///
    /// Use [`Protocol::protocol_type_definition`] for full params.
    pub fn type_definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<TypeDefinitionResponse>> {
        self.protocol_type_definition(TypeDefinitionParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/implementation`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_implementation).
    ///
    /// Use [`Protocol::protocol_implementation`] for full params.
    pub fn implementation(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<ImplementationResponse>> {
        self.protocol_implementation(ImplementationParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/references`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_references).
    ///
    /// Use [`Protocol::protocol_references`] for full params.
    pub fn references(
        &mut self,
        uri: &Uri,
        position: Position,
        include_declaration: bool,
    ) -> Result<Option<Vec<Location>>> {
        self.protocol_references(ReferenceParams {
            context: ReferenceContext {
                include_declaration,
            },
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/documentHighlight`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_documentHighlight).
    ///
    /// Use [`Protocol::protocol_document_highlight`] for full params.
    pub fn document_highlight(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.protocol_document_highlight(DocumentHighlightParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/hover`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_hover).
    ///
    /// Use [`Protocol::protocol_hover`] for full params.
    pub fn hover(&mut self, uri: &Uri, position: Position) -> Result<Option<Hover>> {
        self.protocol_hover(HoverParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`textDocument/moniker`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_moniker).
    ///
    /// Use [`Protocol::protocol_moniker`] for full params.
    pub fn moniker(&mut self, uri: &Uri, position: Position) -> Result<Option<Vec<Moniker>>> {
        self.protocol_moniker(MonikerParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
}
