//! One method per client-to-server request and notification, generated from
//! the tables below in two layers: [`Protocol`]'s `protocol_` methods take
//! the raw params, and `Client`'s own take just what each shape of request
//! needs (a document, a position, a range). Each table must name every
//! [`LspRequestMethod`] or [`LspNotificationMethod`], so a gen-lsp-types
//! bump that adds one fails to compile until it's listed here or skipped.

use gen_lsp_types::{
    LspNotificationMethod, LspRequestMethod, Notification as LspNotification, Position, Range,
    Request as LspRequest, Uri, WithUri,
};
use serde::de::DeserializeOwned;

use super::Client;

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for Client {}

/// Builds params of whatever type from the fields a shape supplies; every
/// other field must be optional, which `every_shaped_method_builds_its_params`
/// checks for each shaped method.
fn shape<P: DeserializeOwned>(fields: serde_json::Value) -> P {
    serde_json::from_value(fields)
        .expect("a shaped method's other params are optional, as its test checks")
}

fn document(uri: &Uri) -> serde_json::Value {
    serde_json::json!({ "textDocument": { "uri": uri } })
}

fn at(uri: &Uri, position: Position) -> serde_json::Value {
    serde_json::json!({ "textDocument": { "uri": uri }, "position": position })
}

fn within(uri: &Uri, range: Range) -> serde_json::Value {
    serde_json::json!({ "textDocument": { "uri": uri }, "range": range })
}

/// `Client`'s own method for one row: built from its shape, or (`params`)
/// taking the raw params when those are already the natural argument, or
/// (`manual`) none, when a hand-written one exists.
macro_rules! shaped {
    (position, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = concat!("Sends a [`", stringify!($item), "`] for `uri` at `position`.")]
        pub fn $name(&mut self, uri: &Uri, position: Position) -> $ret {
            self.$protocol(shape(at(uri, position)))
        }
    };
    (document, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = concat!("Sends a [`", stringify!($item), "`] for `uri`.")]
        pub fn $name(&mut self, uri: &Uri) -> $ret {
            self.$protocol(shape(document(uri)))
        }
    };
    (range, $name:ident, $protocol:ident, $item:ty, $params:ty, $ret:ty) => {
        #[doc = concat!("Sends a [`", stringify!($item), "`] for `range` of `uri`.")]
        pub fn $name(&mut self, uri: &Uri, range: Range) -> $ret {
            self.$protocol(shape(within(uri, range)))
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

/// Builds a shaped method's params, as its generated method would.
#[cfg(test)]
macro_rules! check_shape {
    (position, $params:ty, $uri:ident, $position:ident, $range:ident) => {
        drop(shape::<$params>(at(&$uri, $position)));
    };
    (document, $params:ty, $uri:ident, $position:ident, $range:ident) => {
        drop(shape::<$params>(document(&$uri)));
    };
    (range, $params:ty, $uri:ident, $position:ident, $range:ident) => {
        drop(shape::<$params>(within(&$uri, $range)));
    };
    ($other:ident, $($rest:tt)*) => {};
}

macro_rules! check {
    (open, $self:ident, $params:ident) => {
        $self.state.require_initialized()?;
        $self.docs.require_open($params.uri())?;
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
        /// Every generated request and notification, taking its raw params.
        /// In scope only when imported, so `Client`'s own simpler methods
        /// come first; implemented only by `Client`.
        pub trait Protocol: sealed::Sealed {
            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($request), "`](gen_lsp_types::",
                    stringify!($request), ")."
                )]
                fn [<protocol_ $request_name>](
                    &mut self,
                    params: <gen_lsp_types::$request as LspRequest>::Params,
                ) -> crate::Result<<gen_lsp_types::$request as LspRequest>::Result>;
            )*
            $(
                #[doc = concat!(
                    "Sends a [`", stringify!($notification), "`](gen_lsp_types::",
                    stringify!($notification), ")."
                )]
                fn [<protocol_ $notification_name>](
                    &mut self,
                    params: <gen_lsp_types::$notification as LspNotification>::Params,
                ) -> crate::Result<()>;
            )*
        }

        impl Protocol for Client {
            $(
                fn [<protocol_ $request_name>](
                    &mut self,
                    params: <gen_lsp_types::$request as LspRequest>::Params,
                ) -> crate::Result<<gen_lsp_types::$request as LspRequest>::Result> {
                    const {
                        assert!(matches!(
                            <gen_lsp_types::$request as LspRequest>::METHOD,
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
                    params: <gen_lsp_types::$notification as LspNotification>::Params,
                ) -> crate::Result<()> {
                    const {
                        assert!(matches!(
                            <gen_lsp_types::$notification as LspNotification>::METHOD,
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
                    <gen_lsp_types::$request as LspRequest>::Params,
                    crate::Result<<gen_lsp_types::$request as LspRequest>::Result>
                );
            )*
            $(
                shaped!(
                    $notification_shape,
                    $notification_name,
                    [<protocol_ $notification_name>],
                    gen_lsp_types::$notification,
                    <gen_lsp_types::$notification as LspNotification>::Params,
                    crate::Result<()>
                );
            )*
        }

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

        #[cfg(test)]
        #[test]
        fn every_shaped_method_builds_its_params() {
            let uri: Uri = "file:///a.json".parse().unwrap();
            let position = Position::new(0, 0);
            let range = Range::new(position, position);
            $(check_shape!(
                $request_shape,
                <gen_lsp_types::$request as LspRequest>::Params,
                uri, position, range
            );)*
            $(check_shape!(
                $notification_shape,
                <gen_lsp_types::$notification as LspNotification>::Params,
                uri, position, range
            );)*
        }
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
        call_hierarchy_incoming_calls: CallHierarchyIncomingCalls => CallHierarchyIncomingCallsRequest, params;
        call_hierarchy_outgoing_calls: CallHierarchyOutgoingCalls => CallHierarchyOutgoingCallsRequest, params;
        code_action: TextDocumentCodeAction => CodeActionRequest, manual, open;
        code_action_resolve: CodeActionResolve => CodeActionResolveRequest, params;
        code_lens: TextDocumentCodeLens => CodeLensRequest, document, open;
        code_lens_resolve: CodeLensResolve => CodeLensResolveRequest, params;
        color_presentation: TextDocumentColorPresentation => ColorPresentationRequest, params, open;
        completion: TextDocumentCompletion => CompletionRequest, position, open;
        completion_item_resolve: CompletionItemResolve => CompletionResolveRequest, params;
        declaration: TextDocumentDeclaration => DeclarationRequest, position, open;
        definition: TextDocumentDefinition => DefinitionRequest, position, open;
        // `Client::pull_diagnostics` tracks result ids across pulls.
        diagnostic: TextDocumentDiagnostic => DocumentDiagnosticRequest, manual, open;
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
        on_type_formatting: TextDocumentOnTypeFormatting => DocumentOnTypeFormattingRequest, params, open;
        prepare_call_hierarchy: TextDocumentPrepareCallHierarchy => CallHierarchyPrepareRequest, position, open;
        prepare_rename: TextDocumentPrepareRename => PrepareRenameRequest, position, open;
        prepare_type_hierarchy: TextDocumentPrepareTypeHierarchy => TypeHierarchyPrepareRequest, position, open;
        range_formatting: TextDocumentRangeFormatting => DocumentRangeFormattingRequest, params, open;
        ranges_formatting: TextDocumentRangesFormatting => DocumentRangesFormattingRequest, params, open;
        references: TextDocumentReferences => ReferencesRequest, params, open;
        rename: TextDocumentRename => RenameRequest, params, open;
        selection_range: TextDocumentSelectionRange => SelectionRangeRequest, params, open;
        semantic_tokens_full: TextDocumentSemanticTokensFull => SemanticTokensRequest, document, open;
        semantic_tokens_full_delta: TextDocumentSemanticTokensFullDelta => SemanticTokensDeltaRequest, params, open;
        semantic_tokens_range: TextDocumentSemanticTokensRange => SemanticTokensRangeRequest, range, open;
        signature_help: TextDocumentSignatureHelp => SignatureHelpRequest, position, open;
        type_definition: TextDocumentTypeDefinition => TypeDefinitionRequest, position, open;
        type_hierarchy_subtypes: TypeHierarchySubtypes => TypeHierarchySubtypesRequest, params;
        type_hierarchy_supertypes: TypeHierarchySupertypes => TypeHierarchySupertypesRequest, params;
        will_save_wait_until: TextDocumentWillSaveWaitUntil => WillSaveTextDocumentWaitUntilRequest, params, open;
        workspace_diagnostic: WorkspaceDiagnostic => WorkspaceDiagnosticRequest, params;
        workspace_execute_command: WorkspaceExecuteCommand => ExecuteCommandRequest, params;
        workspace_symbol: WorkspaceSymbol => WorkspaceSymbolRequest, params;
        workspace_symbol_resolve: WorkspaceSymbolResolve => WorkspaceSymbolResolveRequest, params;
        workspace_text_document_content: WorkspaceTextDocumentContent => TextDocumentContentRequest, params;
        workspace_will_create_files: WorkspaceWillCreateFiles => WillCreateFilesRequest, params;
        workspace_will_delete_files: WorkspaceWillDeleteFiles => WillDeleteFilesRequest, params;
        workspace_will_rename_files: WorkspaceWillRenameFiles => WillRenameFilesRequest, params;
    }
    notifications {
        skip: [
            // Hand-written: they drive the lifecycle or track documents.
            Initialized,
            Exit,
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
        cancel_request: CancelRequest => CancelNotification, params;
        did_save: TextDocumentDidSave => DidSaveTextDocumentNotification, document, open;
        notebook_document_did_change: NotebookDocumentDidChange => DidChangeNotebookDocumentNotification, params;
        notebook_document_did_close: NotebookDocumentDidClose => DidCloseNotebookDocumentNotification, params;
        notebook_document_did_open: NotebookDocumentDidOpen => DidOpenNotebookDocumentNotification, params;
        notebook_document_did_save: NotebookDocumentDidSave => DidSaveNotebookDocumentNotification, params;
        progress: Progress => ProgressNotification, params;
        set_trace: SetTrace => SetTraceNotification, params;
        will_save: TextDocumentWillSave => WillSaveTextDocumentNotification, params, open;
        window_work_done_progress_cancel: WindowWorkDoneProgressCancel => WorkDoneProgressCancelNotification, params;
        workspace_did_change_configuration: WorkspaceDidChangeConfiguration => DidChangeConfigurationNotification, params;
        workspace_did_change_watched_files: WorkspaceDidChangeWatchedFiles => DidChangeWatchedFilesNotification, params;
        workspace_did_change_workspace_folders: WorkspaceDidChangeWorkspaceFolders => DidChangeWorkspaceFoldersNotification, params;
        workspace_did_create_files: WorkspaceDidCreateFiles => DidCreateFilesNotification, params;
        workspace_did_delete_files: WorkspaceDidDeleteFiles => DidDeleteFilesNotification, params;
        workspace_did_rename_files: WorkspaceDidRenameFiles => DidRenameFilesNotification, params;
    }
}
