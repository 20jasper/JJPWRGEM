//! Generated client-to-server methods: raw [`Protocol`] methods plus simpler
//! `Client` shortcuts.

use gen_lsp_types::*;
use serde::de::DeserializeOwned;

use super::Client;

/// Builds a params value from the fields a shape supplies.
fn shape<P: DeserializeOwned>(fields: serde_json::Value) -> P {
    serde_json::from_value(fields).expect("shape must cover required params")
}

trait FromUri {
    fn from_uri(uri: &Uri) -> Self;
}

trait FromPosition {
    fn from_position(uri: &Uri, position: Position) -> Self;
}

trait FromRange {
    fn from_range(uri: &Uri, range: Range) -> Self;
}

macro_rules! impl_from_uri {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromUri for $ty {
            fn from_uri(uri: &Uri) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

macro_rules! impl_from_position {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromPosition for $ty {
            fn from_position(uri: &Uri, position: Position) -> Self {
                Self {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        position,
                    },
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

macro_rules! impl_from_range {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromRange for $ty {
            fn from_range(uri: &Uri, range: Range) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    range,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

/// Builds a shortcut's doc comment and link to its raw params method.
macro_rules! shaped_doc {
    ($item:ty, $name:ident) => {
        concat!(
            "Sends a [`",
            stringify!($item),
            "`].\n\n",
            "Use [`Protocol::protocol_",
            stringify!($name),
            "`] for full params."
        )
    };
    ($description:literal, $item:ty, $name:ident) => {
        concat!(
            "Sends a [`",
            stringify!($item),
            "`] ",
            $description,
            ".\n\n",
            "Use [`Protocol::protocol_",
            stringify!($name),
            "`] for full params."
        )
    };
}

/// Generates `Client`'s shortcut for one table row.
macro_rules! shaped {
    (position, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `uri` at `position`", $item, $name)]
        pub fn $name(&mut self, uri: &Uri, position: Position) -> $ret {
            self.$protocol(<$params as FromPosition>::from_position(uri, position))
        }
    };
    (document, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `uri`", $item, $name)]
        pub fn $name(&mut self, uri: &Uri) -> $ret {
            self.$protocol(<$params as FromUri>::from_uri(uri))
        }
    };
    (range, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `range` of `uri`", $item, $name)]
        pub fn $name(&mut self, uri: &Uri, range: Range) -> $ret {
            self.$protocol(<$params as FromRange>::from_range(uri, range))
        }
    };
    (query, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `query`", $item, $name)]
        pub fn $name(&mut self, query: impl Into<String>) -> $ret {
            let query: String = query.into();
            self.$protocol(shape(serde_json::json!({ "query": query })))
        }
    };
    (position_include_declaration, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `uri` at `position`", $item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            position: Position,
            include_declaration: bool,
        ) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position,
                "context": { "includeDeclaration": include_declaration },
            })))
        }
    };
    (position_new_name, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `uri` at `position`", $item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            position: Position,
            new_name: impl Into<String>,
        ) -> $ret {
            let new_name: String = new_name.into();
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position,
                "newName": new_name,
            })))
        }
    };
    (range_options, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!("for `range` of `uri`", $item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            range: Range,
            options: FormattingOptions,
        ) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "range": range,
                "options": options,
            })))
        }
    };
    (document_reason, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, uri: &Uri, reason: TextDocumentSaveReason) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "reason": reason,
            })))
        }
    };
    (positions, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, uri: &Uri, positions: Vec<Position>) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "positions": positions,
            })))
        }
    };
    (on_type_formatting, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            position: Position,
            ch: impl Into<String>,
            options: FormattingOptions,
        ) -> $ret {
            let ch: String = ch.into();
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position,
                "ch": ch,
                "options": options,
            })))
        }
    };
    (color_presentation, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, uri: &Uri, color: Color, range: Range) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "color": color,
                "range": range,
            })))
        }
    };
    (execute_command, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(
            &mut self,
            command: impl Into<String>,
            arguments: Option<Vec<LspAny>>,
        ) -> $ret {
            let command: String = command.into();
            self.$protocol(shape(serde_json::json!({
                "command": command,
                "arguments": arguments,
            })))
        }
    };
    (semantic_tokens_delta, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            previous_result_id: impl Into<String>,
        ) -> $ret {
            let previous_result_id: String = previous_result_id.into();
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "previousResultId": previous_result_id,
            })))
        }
    };
    (progress, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, token: ProgressToken, value: LspAny) -> $ret {
            self.$protocol(shape(serde_json::json!({ "token": token, "value": value })))
        }
    };
    (set_trace, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, value: TraceValue) -> $ret {
            self.$protocol(shape(serde_json::json!({ "value": value })))
        }
    };
    (settings, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, settings: LspAny) -> $ret {
            self.$protocol(shape(serde_json::json!({ "settings": settings })))
        }
    };
    (watched_files, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, changes: Vec<FileEvent>) -> $ret {
            self.$protocol(shape(serde_json::json!({ "changes": changes })))
        }
    };
    (workspace_folders, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, event: WorkspaceFoldersChangeEvent) -> $ret {
            self.$protocol(shape(serde_json::json!({ "event": event })))
        }
    };
    (cancel, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, id: Id) -> $ret {
            self.$protocol(shape(serde_json::json!({ "id": id })))
        }
    };
    (call_hierarchy_item, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, item: CallHierarchyItem) -> $ret {
            self.$protocol(shape(serde_json::json!({ "item": item })))
        }
    };
    (type_hierarchy_item, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, item: TypeHierarchyItem) -> $ret {
            self.$protocol(shape(serde_json::json!({ "item": item })))
        }
    };
    (ranges_options, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(
            &mut self,
            uri: &Uri,
            ranges: Vec<Range>,
            options: FormattingOptions,
        ) -> $ret {
            self.$protocol(shape(serde_json::json!({
                "textDocument": { "uri": uri },
                "ranges": ranges,
                "options": options,
            })))
        }
    };
    (uri, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = shaped_doc!($item, $name)]
        pub fn $name(&mut self, uri: &Uri) -> $ret {
            self.$protocol(shape(serde_json::json!({ "uri": uri })))
        }
    };
    (params, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = concat!("Sends a [`", stringify!($item), "`].")]
        pub fn $name(&mut self, params: $params) -> $ret {
            self.$protocol(params)
        }
    };
    (manual, $($rest:tt)*) => {};
}

impl_from_uri!(
    DocumentSymbolParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    FoldingRangeParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    CodeLensParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    DocumentLinkParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    DocumentColorParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    SemanticTokensParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_uri!(
    DocumentDiagnosticParams,
    identifier: None,
    previous_result_id: None,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);

impl_from_position!(
    CompletionParams,
    context: None,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    DeclarationParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    DefinitionParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    DocumentHighlightParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    HoverParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    ImplementationParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    LinkedEditingRangeParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    MonikerParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position!(
    CallHierarchyPrepareParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    PrepareRenameParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    TypeHierarchyPrepareParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    SignatureHelpParams,
    context: None,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_position!(
    TypeDefinitionParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);

impl_from_range!(
    InlayHintParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_range!(
    SemanticTokensRangeParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);

macro_rules! check {
    (open, $self:ident, $params:ident) => {
        $self.state.require_initialized()?;
    };
}

macro_rules! client_methods {
    (
        requests {
            skip: [$($skipped_request:ident),* $(,)?]
            $($request_name:ident: $request_variant:ident => $request:ident,
                $request_shape:ident $(, $request_check:ident)?;)*
        }
        notifications {
            skip: [$($skipped_notification:ident),* $(,)?]
            $($notification_name:ident: $notification_variant:ident => $notification:ident,
                $notification_shape:ident $(, $notification_check:ident)?;)*
        }
    ) => { pastey::paste! {
        /// Every generated method, taking its raw params.
        pub trait Protocol {
            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($request), "`](gen_lsp_types::",
                    stringify!($request), ")."
                )]
                fn [<protocol_ $request_name>](
                    &mut self,
                    params: <gen_lsp_types::$request as Request>::Params,
                ) -> crate::Result<<gen_lsp_types::$request as Request>::Result>;
            )*
            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($notification), "`](gen_lsp_types::",
                    stringify!($notification), ")."
                )]
                fn [<protocol_ $notification_name>](
                    &mut self,
                    params: <gen_lsp_types::$notification as Notification>::Params,
                ) -> crate::Result<()>;
            )*
        }

        impl Protocol for Client {
            $(
                fn [<protocol_ $request_name>](
                    &mut self,
                    params: <gen_lsp_types::$request as Request>::Params,
                ) -> crate::Result<<gen_lsp_types::$request as Request>::Result> {
                    const {
                        assert!(matches!(
                            <gen_lsp_types::$request as Request>::METHOD,
                            LspRequestMethod::$request_variant
                        ));
                    }
                    $(check!($request_check, self, params);)?
                    self.request::<gen_lsp_types::$request>(params)
                }
            )*
            $(
                fn [<protocol_ $notification_name>](
                    &mut self,
                    params: <gen_lsp_types::$notification as Notification>::Params,
                ) -> crate::Result<()> {
                    const {
                        assert!(matches!(
                            <gen_lsp_types::$notification as Notification>::METHOD,
                            LspNotificationMethod::$notification_variant
                        ));
                    }
                    $(check!($notification_check, self, params);)?
                    self.notify::<gen_lsp_types::$notification>(params)
                }
            )*
        }

        impl Client {
            $(
                shaped!(
                    $request_shape,
                    $request_name,
                    [<protocol_ $request_name>],
                    gen_lsp_types::$request,
                    <gen_lsp_types::$request as Request>::Params,
                    crate::Result<<gen_lsp_types::$request as Request>::Result>
                );
            )*
            $(
                shaped!(
                    $notification_shape,
                    $notification_name,
                    [<protocol_ $notification_name>],
                    gen_lsp_types::$notification,
                    <gen_lsp_types::$notification as Notification>::Params,
                    crate::Result<()>
                );
            )*
        }

        // Each match fails to compile when gen-lsp-types adds an unlisted method.
        const _: fn(LspRequestMethod<'_>) = |method| match method {
            $(LspRequestMethod::$request_variant)|*
            | $(LspRequestMethod::$skipped_request)|*
            | LspRequestMethod::Custom(_) => {}
        };

        const _: fn(LspNotificationMethod<'_>) = |method| match method {
            $(LspNotificationMethod::$notification_variant)|*
            | $(LspNotificationMethod::$skipped_notification)|*
            | LspNotificationMethod::Custom(_) => {}
        };
    }};
}

client_methods! {
    requests {
        skip: [
            // Hand-written: they drive the lifecycle.
            Initialize,
            Shutdown,
            // Sent by the server; answered by `core::server_requests`.
            ClientRegisterCapability,
            ClientUnregisterCapability,
            WindowShowDocument,
            WindowShowMessageRequest,
            WindowWorkDoneProgressCreate,
            WorkspaceApplyEdit,
            WorkspaceCodeLensRefresh,
            WorkspaceConfiguration,
            WorkspaceDiagnosticRefresh,
            WorkspaceFoldingRangeRefresh,
            WorkspaceInlayHintRefresh,
            WorkspaceInlineValueRefresh,
            WorkspaceSemanticTokensRefresh,
            WorkspaceTextDocumentContentRefresh,
            WorkspaceWorkspaceFolders,
        ]
        call_hierarchy_incoming_calls: CallHierarchyIncomingCalls => CallHierarchyIncomingCallsRequest, call_hierarchy_item;
        call_hierarchy_outgoing_calls: CallHierarchyOutgoingCalls => CallHierarchyOutgoingCallsRequest, call_hierarchy_item;
        code_action: TextDocumentCodeAction => CodeActionRequest, manual, open;
        code_action_resolve: CodeActionResolve => CodeActionResolveRequest, params;
        code_lens: TextDocumentCodeLens => CodeLensRequest, document, open;
        code_lens_resolve: CodeLensResolve => CodeLensResolveRequest, params;
        color_presentation: TextDocumentColorPresentation => ColorPresentationRequest, color_presentation, open;
        completion: TextDocumentCompletion => CompletionRequest, position, open;
        completion_item_resolve: CompletionItemResolve => CompletionResolveRequest, params;
        declaration: TextDocumentDeclaration => DeclarationRequest, position, open;
        definition: TextDocumentDefinition => DefinitionRequest, position, open;
        diagnostic: TextDocumentDiagnostic => DocumentDiagnosticRequest, document, open;
        document_color: TextDocumentDocumentColor => DocumentColorRequest, document, open;
        document_highlight: TextDocumentDocumentHighlight => DocumentHighlightRequest, position, open;
        document_link: TextDocumentDocumentLink => DocumentLinkRequest, document, open;
        document_link_resolve: DocumentLinkResolve => DocumentLinkResolveRequest, params;
        document_symbol: TextDocumentDocumentSymbol => DocumentSymbolRequest, document, open;
        folding_range: TextDocumentFoldingRange => FoldingRangeRequest, document, open;
        formatting: TextDocumentFormatting => DocumentFormattingRequest, manual, open;
        hover: TextDocumentHover => HoverRequest, position, open;
        implementation: TextDocumentImplementation => ImplementationRequest, position, open;
        inlay_hint: TextDocumentInlayHint => InlayHintRequest, range, open;
        inlay_hint_resolve: InlayHintResolve => InlayHintResolveRequest, params;
        inline_completion: TextDocumentInlineCompletion => InlineCompletionRequest, params, open;
        inline_value: TextDocumentInlineValue => InlineValueRequest, params, open;
        linked_editing_range: TextDocumentLinkedEditingRange => LinkedEditingRangeRequest, position, open;
        moniker: TextDocumentMoniker => MonikerRequest, position, open;
        on_type_formatting: TextDocumentOnTypeFormatting => DocumentOnTypeFormattingRequest, on_type_formatting, open;
        prepare_call_hierarchy: TextDocumentPrepareCallHierarchy => CallHierarchyPrepareRequest, position, open;
        prepare_rename: TextDocumentPrepareRename => PrepareRenameRequest, position, open;
        prepare_type_hierarchy: TextDocumentPrepareTypeHierarchy => TypeHierarchyPrepareRequest, position, open;
        range_formatting: TextDocumentRangeFormatting => DocumentRangeFormattingRequest, range_options, open;
        ranges_formatting: TextDocumentRangesFormatting => DocumentRangesFormattingRequest, ranges_options, open;
        references: TextDocumentReferences => ReferencesRequest, position_include_declaration, open;
        rename: TextDocumentRename => RenameRequest, position_new_name, open;
        selection_range: TextDocumentSelectionRange => SelectionRangeRequest, positions, open;
        semantic_tokens_full: TextDocumentSemanticTokensFull => SemanticTokensRequest, document, open;
        semantic_tokens_full_delta: TextDocumentSemanticTokensFullDelta => SemanticTokensDeltaRequest, semantic_tokens_delta, open;
        semantic_tokens_range: TextDocumentSemanticTokensRange => SemanticTokensRangeRequest, range, open;
        signature_help: TextDocumentSignatureHelp => SignatureHelpRequest, position, open;
        type_definition: TextDocumentTypeDefinition => TypeDefinitionRequest, position, open;
        type_hierarchy_subtypes: TypeHierarchySubtypes => TypeHierarchySubtypesRequest, type_hierarchy_item;
        type_hierarchy_supertypes: TypeHierarchySupertypes => TypeHierarchySupertypesRequest, type_hierarchy_item;
        will_save_wait_until: TextDocumentWillSaveWaitUntil => WillSaveTextDocumentWaitUntilRequest, document_reason, open;
        workspace_diagnostic: WorkspaceDiagnostic => WorkspaceDiagnosticRequest, params;
        workspace_execute_command: WorkspaceExecuteCommand => ExecuteCommandRequest, execute_command;
        workspace_symbol: WorkspaceSymbol => WorkspaceSymbolRequest, query;
        workspace_symbol_resolve: WorkspaceSymbolResolve => WorkspaceSymbolResolveRequest, params;
        workspace_text_document_content: WorkspaceTextDocumentContent => TextDocumentContentRequest, uri;
        workspace_will_create_files: WorkspaceWillCreateFiles => WillCreateFilesRequest, params;
        workspace_will_delete_files: WorkspaceWillDeleteFiles => WillDeleteFilesRequest, params;
        workspace_will_rename_files: WorkspaceWillRenameFiles => WillRenameFilesRequest, params;
    }
    notifications {
        skip: [
            // Hand-written: lifecycle.
            Initialized,
            Exit,
            // Document tracking.
            TextDocumentDidOpen,
            TextDocumentDidChange,
            TextDocumentDidClose,
            // Sent by the server, not the client.
            LogTrace,
            TelemetryEvent,
            TextDocumentPublishDiagnostics,
            WindowLogMessage,
            WindowShowMessage,
        ]
        cancel_request: CancelRequest => CancelNotification, cancel;
        did_save: TextDocumentDidSave => DidSaveTextDocumentNotification, manual, open;
        notebook_document_did_change: NotebookDocumentDidChange => DidChangeNotebookDocumentNotification, params;
        notebook_document_did_close: NotebookDocumentDidClose => DidCloseNotebookDocumentNotification, params;
        notebook_document_did_open: NotebookDocumentDidOpen => DidOpenNotebookDocumentNotification, params;
        notebook_document_did_save: NotebookDocumentDidSave => DidSaveNotebookDocumentNotification, params;
        progress: Progress => ProgressNotification, progress;
        set_trace: SetTrace => SetTraceNotification, set_trace;
        will_save: TextDocumentWillSave => WillSaveTextDocumentNotification, document_reason, open;
        window_work_done_progress_cancel: WindowWorkDoneProgressCancel => WorkDoneProgressCancelNotification, params;
        workspace_did_change_configuration: WorkspaceDidChangeConfiguration => DidChangeConfigurationNotification, settings;
        workspace_did_change_watched_files: WorkspaceDidChangeWatchedFiles => DidChangeWatchedFilesNotification, watched_files;
        workspace_did_change_workspace_folders: WorkspaceDidChangeWorkspaceFolders => DidChangeWorkspaceFoldersNotification, workspace_folders;
        workspace_did_create_files: WorkspaceDidCreateFiles => DidCreateFilesNotification, params;
        workspace_did_delete_files: WorkspaceDidDeleteFiles => DidDeleteFilesNotification, params;
        workspace_did_rename_files: WorkspaceDidRenameFiles => DidRenameFilesNotification, params;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri() -> Uri {
        "file:///a.json".parse().unwrap()
    }

    #[test]
    fn document_shape_sets_the_uri() {
        let uri = uri();
        let params = DocumentSymbolParams::from_uri(&uri);
        assert_eq!(params.text_document.uri, uri);
    }

    #[test]
    fn position_shape_sets_the_uri_and_position() {
        let uri = uri();
        let position = Position::new(1, 2);
        let params = HoverParams::from_position(&uri, position);
        assert_eq!(params.text_document_position_params.text_document.uri, uri);
        assert_eq!(params.text_document_position_params.position, position);
    }

    #[test]
    fn range_shape_sets_the_uri_and_range() {
        let uri = uri();
        let range = Range::new(Position::new(0, 0), Position::new(1, 1));
        let params = InlayHintParams::from_range(&uri, range);
        assert_eq!(params.text_document.uri, uri);
        assert_eq!(params.range, range);
    }
}
