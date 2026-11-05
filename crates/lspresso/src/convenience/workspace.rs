use gen_lsp_types::*;

use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`workspace/executeCommand`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_executeCommand).
    ///
    /// Use [`Protocol::protocol_workspace_execute_command`] for full params.
    pub fn workspace_execute_command(
        &mut self,
        command: impl Into<String>,
        arguments: Option<Vec<LspAny>>,
    ) -> Result<Option<LspAny>> {
        self.protocol_workspace_execute_command(ExecuteCommandParams {
            command: command.into(),
            arguments,
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`workspace/textDocumentContent`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_textDocumentContent).
    ///
    /// Use [`Protocol::protocol_workspace_text_document_content`] for full params.
    pub fn workspace_text_document_content(
        &mut self,
        uri: &Uri,
    ) -> Result<TextDocumentContentResult> {
        self.protocol_workspace_text_document_content(TextDocumentContentParams {
            uri: uri.clone(),
        })
    }

    /// Sends [`workspace/didChangeConfiguration`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didChangeConfiguration).
    ///
    /// Use [`Protocol::protocol_workspace_did_change_configuration`] for full params.
    pub fn workspace_did_change_configuration(&mut self, settings: LspAny) -> Result<()> {
        self.protocol_workspace_did_change_configuration(DidChangeConfigurationParams { settings })
    }

    /// Sends [`workspace/didChangeWatchedFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didChangeWatchedFiles).
    ///
    /// Use [`Protocol::protocol_workspace_did_change_watched_files`] for full params.
    pub fn workspace_did_change_watched_files(&mut self, changes: Vec<FileEvent>) -> Result<()> {
        self.protocol_workspace_did_change_watched_files(DidChangeWatchedFilesParams { changes })
    }

    /// Sends [`workspace/didChangeWorkspaceFolders`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didChangeWorkspaceFolders).
    ///
    /// Use [`Protocol::protocol_workspace_did_change_workspace_folders`] for full params.
    pub fn workspace_did_change_workspace_folders(
        &mut self,
        event: WorkspaceFoldersChangeEvent,
    ) -> Result<()> {
        self.protocol_workspace_did_change_workspace_folders(DidChangeWorkspaceFoldersParams {
            event,
        })
    }

    /// Sends [`workspace/willCreateFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_willCreateFiles).
    ///
    /// Use [`Protocol::protocol_workspace_will_create_files`] for full params.
    pub fn workspace_will_create_files(
        &mut self,
        params: CreateFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_create_files(params)
    }

    /// Sends [`workspace/willRenameFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_willRenameFiles).
    ///
    /// Use [`Protocol::protocol_workspace_will_rename_files`] for full params.
    pub fn workspace_will_rename_files(
        &mut self,
        params: RenameFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_rename_files(params)
    }

    /// Sends [`workspace/willDeleteFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_willDeleteFiles).
    ///
    /// Use [`Protocol::protocol_workspace_will_delete_files`] for full params.
    pub fn workspace_will_delete_files(
        &mut self,
        params: DeleteFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_delete_files(params)
    }

    /// Sends [`workspace/didCreateFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didCreateFiles).
    ///
    /// Use [`Protocol::protocol_workspace_did_create_files`] for full params.
    pub fn workspace_did_create_files(&mut self, params: CreateFilesParams) -> Result<()> {
        self.protocol_workspace_did_create_files(params)
    }

    /// Sends [`workspace/didRenameFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didRenameFiles).
    ///
    /// Use [`Protocol::protocol_workspace_did_rename_files`] for full params.
    pub fn workspace_did_rename_files(&mut self, params: RenameFilesParams) -> Result<()> {
        self.protocol_workspace_did_rename_files(params)
    }

    /// Sends [`workspace/didDeleteFiles`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_didDeleteFiles).
    ///
    /// Use [`Protocol::protocol_workspace_did_delete_files`] for full params.
    pub fn workspace_did_delete_files(&mut self, params: DeleteFilesParams) -> Result<()> {
        self.protocol_workspace_did_delete_files(params)
    }
}
