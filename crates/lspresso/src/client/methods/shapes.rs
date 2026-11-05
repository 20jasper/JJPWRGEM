//! Typed params constructors for the `Client` shortcuts.

use gen_lsp_types::*;

pub(super) trait FromUri {
    fn from_uri(uri: &Uri) -> Self;
}

pub(super) trait FromPosition {
    fn from_position(uri: &Uri, position: Position) -> Self;
}

pub(super) trait FromRange {
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

pub(super) trait FromQuery {
    fn from_query(query: String) -> Self;
}

macro_rules! impl_from_query {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromQuery for $ty {
            fn from_query(query: String) -> Self {
                Self { query, $($extra: $extra_value),* }
            }
        }
    };
}

pub(super) trait FromPositionIncludeDeclaration {
    fn from_position_include_declaration(
        uri: &Uri,
        position: Position,
        include_declaration: bool,
    ) -> Self;
}

macro_rules! impl_from_position_include_declaration {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromPositionIncludeDeclaration for $ty {
            fn from_position_include_declaration(
                uri: &Uri,
                position: Position,
                include_declaration: bool,
            ) -> Self {
                Self {
                    context: ReferenceContext { include_declaration },
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

pub(super) trait FromPositionNewName {
    fn from_position_new_name(uri: &Uri, position: Position, new_name: String) -> Self;
}

macro_rules! impl_from_position_new_name {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromPositionNewName for $ty {
            fn from_position_new_name(uri: &Uri, position: Position, new_name: String) -> Self {
                Self {
                    new_name,
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

pub(super) trait FromRangeOptions {
    fn from_range_options(uri: &Uri, range: Range, options: FormattingOptions) -> Self;
}

macro_rules! impl_from_range_options {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromRangeOptions for $ty {
            fn from_range_options(uri: &Uri, range: Range, options: FormattingOptions) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    range,
                    options,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

pub(super) trait FromDocumentReason {
    fn from_document_reason(uri: &Uri, reason: TextDocumentSaveReason) -> Self;
}

macro_rules! impl_from_document_reason {
    ($ty:ty) => {
        impl FromDocumentReason for $ty {
            fn from_document_reason(uri: &Uri, reason: TextDocumentSaveReason) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    reason,
                }
            }
        }
    };
}

pub(super) trait FromPositions {
    fn from_positions(uri: &Uri, positions: Vec<Position>) -> Self;
}

macro_rules! impl_from_positions {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromPositions for $ty {
            fn from_positions(uri: &Uri, positions: Vec<Position>) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    positions,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

pub(super) trait FromOnTypeFormatting {
    fn from_on_type_formatting(
        uri: &Uri,
        position: Position,
        ch: String,
        options: FormattingOptions,
    ) -> Self;
}

macro_rules! impl_from_on_type_formatting {
    ($ty:ty) => {
        impl FromOnTypeFormatting for $ty {
            fn from_on_type_formatting(
                uri: &Uri,
                position: Position,
                ch: String,
                options: FormattingOptions,
            ) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position,
                    ch,
                    options,
                }
            }
        }
    };
}

pub(super) trait FromColorPresentation {
    fn from_color_presentation(uri: &Uri, color: Color, range: Range) -> Self;
}

macro_rules! impl_from_color_presentation {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromColorPresentation for $ty {
            fn from_color_presentation(uri: &Uri, color: Color, range: Range) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    color,
                    range,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

pub(super) trait FromExecuteCommand {
    fn from_execute_command(command: String, arguments: Option<Vec<LspAny>>) -> Self;
}

macro_rules! impl_from_execute_command {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromExecuteCommand for $ty {
            fn from_execute_command(command: String, arguments: Option<Vec<LspAny>>) -> Self {
                Self { command, arguments, $($extra: $extra_value),* }
            }
        }
    };
}

pub(super) trait FromSemanticTokensDelta {
    fn from_semantic_tokens_delta(uri: &Uri, previous_result_id: String) -> Self;
}

macro_rules! impl_from_semantic_tokens_delta {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromSemanticTokensDelta for $ty {
            fn from_semantic_tokens_delta(uri: &Uri, previous_result_id: String) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    previous_result_id,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

pub(super) trait FromProgress {
    fn from_progress(token: ProgressToken, value: LspAny) -> Self;
}

macro_rules! impl_from_progress {
    ($ty:ty) => {
        impl FromProgress for $ty {
            fn from_progress(token: ProgressToken, value: LspAny) -> Self {
                Self { token, value }
            }
        }
    };
}

pub(super) trait FromSetTrace {
    fn from_set_trace(value: TraceValue) -> Self;
}

macro_rules! impl_from_set_trace {
    ($ty:ty) => {
        impl FromSetTrace for $ty {
            fn from_set_trace(value: TraceValue) -> Self {
                Self { value }
            }
        }
    };
}

pub(super) trait FromSettings {
    fn from_settings(settings: LspAny) -> Self;
}

macro_rules! impl_from_settings {
    ($ty:ty) => {
        impl FromSettings for $ty {
            fn from_settings(settings: LspAny) -> Self {
                Self { settings }
            }
        }
    };
}

pub(super) trait FromWatchedFiles {
    fn from_watched_files(changes: Vec<FileEvent>) -> Self;
}

macro_rules! impl_from_watched_files {
    ($ty:ty) => {
        impl FromWatchedFiles for $ty {
            fn from_watched_files(changes: Vec<FileEvent>) -> Self {
                Self { changes }
            }
        }
    };
}

pub(super) trait FromWorkspaceFolders {
    fn from_workspace_folders(event: WorkspaceFoldersChangeEvent) -> Self;
}

macro_rules! impl_from_workspace_folders {
    ($ty:ty) => {
        impl FromWorkspaceFolders for $ty {
            fn from_workspace_folders(event: WorkspaceFoldersChangeEvent) -> Self {
                Self { event }
            }
        }
    };
}

pub(super) trait FromCancel {
    fn from_cancel(id: Id) -> Self;
}

macro_rules! impl_from_cancel {
    ($ty:ty) => {
        impl FromCancel for $ty {
            fn from_cancel(id: Id) -> Self {
                Self { id }
            }
        }
    };
}

pub(super) trait FromCallHierarchyItem {
    fn from_call_hierarchy_item(item: CallHierarchyItem) -> Self;
}

macro_rules! impl_from_call_hierarchy_item {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromCallHierarchyItem for $ty {
            fn from_call_hierarchy_item(item: CallHierarchyItem) -> Self {
                Self { item, $($extra: $extra_value),* }
            }
        }
    };
}

pub(super) trait FromTypeHierarchyItem {
    fn from_type_hierarchy_item(item: TypeHierarchyItem) -> Self;
}

macro_rules! impl_from_type_hierarchy_item {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromTypeHierarchyItem for $ty {
            fn from_type_hierarchy_item(item: TypeHierarchyItem) -> Self {
                Self { item, $($extra: $extra_value),* }
            }
        }
    };
}

pub(super) trait FromRangesOptions {
    fn from_ranges_options(uri: &Uri, ranges: Vec<Range>, options: FormattingOptions) -> Self;
}

macro_rules! impl_from_ranges_options {
    ($ty:ty, $($extra:ident : $extra_value:expr),* $(,)?) => {
        impl FromRangesOptions for $ty {
            fn from_ranges_options(uri: &Uri, ranges: Vec<Range>, options: FormattingOptions) -> Self {
                Self {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    ranges,
                    options,
                    $($extra: $extra_value),*
                }
            }
        }
    };
}

pub(super) trait FromUriOnly {
    fn from_uri_only(uri: &Uri) -> Self;
}

macro_rules! impl_from_uri_only {
    ($ty:ty) => {
        impl FromUriOnly for $ty {
            fn from_uri_only(uri: &Uri) -> Self {
                Self { uri: uri.clone() }
            }
        }
    };
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

impl_from_query!(
    WorkspaceSymbolParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position_include_declaration!(
    ReferenceParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_position_new_name!(
    RenameParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_range_options!(
    DocumentRangeFormattingParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_document_reason!(WillSaveTextDocumentParams);
impl_from_positions!(
    SelectionRangeParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_on_type_formatting!(DocumentOnTypeFormattingParams);
impl_from_color_presentation!(
    ColorPresentationParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_execute_command!(
    ExecuteCommandParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_semantic_tokens_delta!(
    SemanticTokensDeltaParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_progress!(ProgressParams);
impl_from_set_trace!(SetTraceParams);
impl_from_settings!(DidChangeConfigurationParams);
impl_from_watched_files!(DidChangeWatchedFilesParams);
impl_from_workspace_folders!(DidChangeWorkspaceFoldersParams);
impl_from_cancel!(CancelParams);
impl_from_call_hierarchy_item!(
    CallHierarchyIncomingCallsParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_call_hierarchy_item!(
    CallHierarchyOutgoingCallsParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_type_hierarchy_item!(
    TypeHierarchySubtypesParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_type_hierarchy_item!(
    TypeHierarchySupertypesParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
    partial_result_params: PartialResultParams::default(),
);
impl_from_ranges_options!(
    DocumentRangesFormattingParams,
    work_done_progress_params: WorkDoneProgressParams::default(),
);
impl_from_uri_only!(TextDocumentContentParams);
