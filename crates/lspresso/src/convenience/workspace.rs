use gen_lsp_types::*;

use crate::{Client, Protocol, Result};

impl Client {
    /// Runs `command` on the server with `arguments`.
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

    /// Requests the content of the virtual document at `uri`.
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

    /// Notifies the server that its configuration changed to `settings`.
    ///
    /// Use [`Protocol::protocol_workspace_did_change_configuration`] for full params.
    pub fn workspace_did_change_configuration(&mut self, settings: LspAny) -> Result<()> {
        self.protocol_workspace_did_change_configuration(DidChangeConfigurationParams { settings })
    }

    /// Notifies the server that watched files changed.
    ///
    /// Use [`Protocol::protocol_workspace_did_change_watched_files`] for full params.
    pub fn workspace_did_change_watched_files(&mut self, changes: Vec<FileEvent>) -> Result<()> {
        self.protocol_workspace_did_change_watched_files(DidChangeWatchedFilesParams { changes })
    }

    /// Notifies the server that workspace folders changed.
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

    /// Asks the server for edits to apply before files are created.
    pub fn workspace_will_create_files(
        &mut self,
        params: CreateFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_create_files(params)
    }

    /// Asks the server for edits to apply before files are renamed.
    pub fn workspace_will_rename_files(
        &mut self,
        params: RenameFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_rename_files(params)
    }

    /// Asks the server for edits to apply before files are deleted.
    pub fn workspace_will_delete_files(
        &mut self,
        params: DeleteFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        self.protocol_workspace_will_delete_files(params)
    }

    /// Notifies the server that files were created.
    pub fn workspace_did_create_files(&mut self, params: CreateFilesParams) -> Result<()> {
        self.protocol_workspace_did_create_files(params)
    }

    /// Notifies the server that files were renamed.
    pub fn workspace_did_rename_files(&mut self, params: RenameFilesParams) -> Result<()> {
        self.protocol_workspace_did_rename_files(params)
    }

    /// Notifies the server that files were deleted.
    pub fn workspace_did_delete_files(&mut self, params: DeleteFilesParams) -> Result<()> {
        self.protocol_workspace_did_delete_files(params)
    }
}
