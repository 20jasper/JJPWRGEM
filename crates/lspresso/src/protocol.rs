//! The low-level layer: one method per client-to-server LSP method, taking
//! its raw params. `Client`'s convenience methods build on these.

use gen_lsp_types::*;

use crate::{Client, Result};

macro_rules! protocol_methods {
    (
        requests {
            skip: [$($skipped_request:ident),* $(,)?]
            $($request_name:ident: $request_variant:ident => $request:ident;)*
        }
        notifications {
            skip: [$($skipped_notification:ident),* $(,)?]
            $($notification_name:ident: $notification_variant:ident => $notification:ident;)*
        }
    ) => { pastey::paste! {
        /// Every client-to-server LSP method, taking its raw params.
        ///
        /// Implementors supply [`request`](Protocol::request) and
        /// [`notify`](Protocol::notify); every `protocol_*` method is built on
        /// those two.
        pub trait Protocol {
            /// Sends any request and waits for its response.
            fn request<R: Request>(&mut self, params: R::Params) -> Result<R::Result>;

            /// Sends any notification.
            fn notify<N: Notification>(&mut self, params: N::Params) -> Result<()>;

            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($request), "`](gen_lsp_types::",
                    stringify!($request), ")."
                )]
                fn [<protocol_ $request_name>](
                    &mut self,
                    params: <$request as Request>::Params,
                ) -> Result<<$request as Request>::Result> {
                    const {
                        assert!(matches!(
                            <$request as Request>::METHOD,
                            LspRequestMethod::$request_variant
                        ));
                    }
                    self.request::<$request>(params)
                }
            )*
            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($notification), "`](gen_lsp_types::",
                    stringify!($notification), ")."
                )]
                fn [<protocol_ $notification_name>](
                    &mut self,
                    params: <$notification as Notification>::Params,
                ) -> Result<()> {
                    const {
                        assert!(matches!(
                            <$notification as Notification>::METHOD,
                            LspNotificationMethod::$notification_variant
                        ));
                    }
                    self.notify::<$notification>(params)
                }
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

impl Protocol for Client {
    fn request<R: Request>(&mut self, params: R::Params) -> Result<R::Result> {
        Self::request::<R>(self, params)
    }

    fn notify<N: Notification>(&mut self, params: N::Params) -> Result<()> {
        Self::notify::<N>(self, params)
    }
}

protocol_methods! {
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
        call_hierarchy_incoming_calls: CallHierarchyIncomingCalls => CallHierarchyIncomingCallsRequest;
        call_hierarchy_outgoing_calls: CallHierarchyOutgoingCalls => CallHierarchyOutgoingCallsRequest;
        code_action: TextDocumentCodeAction => CodeActionRequest;
        code_action_resolve: CodeActionResolve => CodeActionResolveRequest;
        code_lens: TextDocumentCodeLens => CodeLensRequest;
        code_lens_resolve: CodeLensResolve => CodeLensResolveRequest;
        color_presentation: TextDocumentColorPresentation => ColorPresentationRequest;
        completion: TextDocumentCompletion => CompletionRequest;
        completion_item_resolve: CompletionItemResolve => CompletionResolveRequest;
        declaration: TextDocumentDeclaration => DeclarationRequest;
        definition: TextDocumentDefinition => DefinitionRequest;
        diagnostic: TextDocumentDiagnostic => DocumentDiagnosticRequest;
        document_color: TextDocumentDocumentColor => DocumentColorRequest;
        document_highlight: TextDocumentDocumentHighlight => DocumentHighlightRequest;
        document_link: TextDocumentDocumentLink => DocumentLinkRequest;
        document_link_resolve: DocumentLinkResolve => DocumentLinkResolveRequest;
        document_symbol: TextDocumentDocumentSymbol => DocumentSymbolRequest;
        folding_range: TextDocumentFoldingRange => FoldingRangeRequest;
        formatting: TextDocumentFormatting => DocumentFormattingRequest;
        hover: TextDocumentHover => HoverRequest;
        implementation: TextDocumentImplementation => ImplementationRequest;
        inlay_hint: TextDocumentInlayHint => InlayHintRequest;
        inlay_hint_resolve: InlayHintResolve => InlayHintResolveRequest;
        inline_completion: TextDocumentInlineCompletion => InlineCompletionRequest;
        inline_value: TextDocumentInlineValue => InlineValueRequest;
        linked_editing_range: TextDocumentLinkedEditingRange => LinkedEditingRangeRequest;
        moniker: TextDocumentMoniker => MonikerRequest;
        on_type_formatting: TextDocumentOnTypeFormatting => DocumentOnTypeFormattingRequest;
        prepare_call_hierarchy: TextDocumentPrepareCallHierarchy => CallHierarchyPrepareRequest;
        prepare_rename: TextDocumentPrepareRename => PrepareRenameRequest;
        prepare_type_hierarchy: TextDocumentPrepareTypeHierarchy => TypeHierarchyPrepareRequest;
        range_formatting: TextDocumentRangeFormatting => DocumentRangeFormattingRequest;
        ranges_formatting: TextDocumentRangesFormatting => DocumentRangesFormattingRequest;
        references: TextDocumentReferences => ReferencesRequest;
        rename: TextDocumentRename => RenameRequest;
        selection_range: TextDocumentSelectionRange => SelectionRangeRequest;
        semantic_tokens_full: TextDocumentSemanticTokensFull => SemanticTokensRequest;
        semantic_tokens_full_delta: TextDocumentSemanticTokensFullDelta => SemanticTokensDeltaRequest;
        semantic_tokens_range: TextDocumentSemanticTokensRange => SemanticTokensRangeRequest;
        signature_help: TextDocumentSignatureHelp => SignatureHelpRequest;
        type_definition: TextDocumentTypeDefinition => TypeDefinitionRequest;
        type_hierarchy_subtypes: TypeHierarchySubtypes => TypeHierarchySubtypesRequest;
        type_hierarchy_supertypes: TypeHierarchySupertypes => TypeHierarchySupertypesRequest;
        will_save_wait_until: TextDocumentWillSaveWaitUntil => WillSaveTextDocumentWaitUntilRequest;
        workspace_diagnostic: WorkspaceDiagnostic => WorkspaceDiagnosticRequest;
        workspace_execute_command: WorkspaceExecuteCommand => ExecuteCommandRequest;
        workspace_symbol: WorkspaceSymbol => WorkspaceSymbolRequest;
        workspace_symbol_resolve: WorkspaceSymbolResolve => WorkspaceSymbolResolveRequest;
        workspace_text_document_content: WorkspaceTextDocumentContent => TextDocumentContentRequest;
        workspace_will_create_files: WorkspaceWillCreateFiles => WillCreateFilesRequest;
        workspace_will_delete_files: WorkspaceWillDeleteFiles => WillDeleteFilesRequest;
        workspace_will_rename_files: WorkspaceWillRenameFiles => WillRenameFilesRequest;
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
        cancel_request: CancelRequest => CancelNotification;
        did_save: TextDocumentDidSave => DidSaveTextDocumentNotification;
        notebook_document_did_change: NotebookDocumentDidChange => DidChangeNotebookDocumentNotification;
        notebook_document_did_close: NotebookDocumentDidClose => DidCloseNotebookDocumentNotification;
        notebook_document_did_open: NotebookDocumentDidOpen => DidOpenNotebookDocumentNotification;
        notebook_document_did_save: NotebookDocumentDidSave => DidSaveNotebookDocumentNotification;
        progress: Progress => ProgressNotification;
        set_trace: SetTrace => SetTraceNotification;
        will_save: TextDocumentWillSave => WillSaveTextDocumentNotification;
        window_work_done_progress_cancel: WindowWorkDoneProgressCancel => WorkDoneProgressCancelNotification;
        workspace_did_change_configuration: WorkspaceDidChangeConfiguration => DidChangeConfigurationNotification;
        workspace_did_change_watched_files: WorkspaceDidChangeWatchedFiles => DidChangeWatchedFilesNotification;
        workspace_did_change_workspace_folders: WorkspaceDidChangeWorkspaceFolders => DidChangeWorkspaceFoldersNotification;
        workspace_did_create_files: WorkspaceDidCreateFiles => DidCreateFilesNotification;
        workspace_did_delete_files: WorkspaceDidDeleteFiles => DidDeleteFilesNotification;
        workspace_did_rename_files: WorkspaceDidRenameFiles => DidRenameFilesNotification;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records what each call sends instead of talking to a server.
    #[derive(Default)]
    struct Recorder {
        sent: Vec<(&'static str, serde_json::Value)>,
    }

    impl Protocol for Recorder {
        fn request<R: Request>(&mut self, params: R::Params) -> Result<R::Result> {
            self.sent
                .push((R::METHOD.as_str(), serde_json::to_value(params).unwrap()));
            Ok(serde_json::from_value(serde_json::Value::Null).unwrap())
        }

        fn notify<N: Notification>(&mut self, params: N::Params) -> Result<()> {
            self.sent
                .push((N::METHOD.as_str(), serde_json::to_value(params).unwrap()));
            Ok(())
        }
    }

    #[test]
    fn defaults_send_their_method_with_the_params_given() {
        let mut recorder = Recorder::default();
        recorder
            .protocol_workspace_symbol(WorkspaceSymbolParams {
                query: "widget".to_owned(),
                ..Default::default()
            })
            .unwrap();
        recorder
            .protocol_set_trace(SetTraceParams {
                value: TraceValue::Verbose,
            })
            .unwrap();

        assert_eq!(
            recorder.sent,
            [
                ("workspace/symbol", serde_json::json!({ "query": "widget" })),
                ("$/setTrace", serde_json::json!({ "value": "verbose" })),
            ]
        );
    }
}
