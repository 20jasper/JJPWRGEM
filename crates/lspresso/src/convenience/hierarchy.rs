use gen_lsp_types::*;

use super::at;
use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`textDocument/prepareCallHierarchy`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_prepareCallHierarchy).
    ///
    /// Use [`Protocol::protocol_prepare_call_hierarchy`] for full params.
    pub fn prepare_call_hierarchy(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        self.protocol_prepare_call_hierarchy(CallHierarchyPrepareParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`callHierarchy/incomingCalls`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#callHierarchy_incomingCalls).
    ///
    /// Use [`Protocol::protocol_call_hierarchy_incoming_calls`] for full params.
    pub fn call_hierarchy_incoming_calls(
        &mut self,
        item: CallHierarchyItem,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        self.protocol_call_hierarchy_incoming_calls(CallHierarchyIncomingCallsParams {
            item,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`callHierarchy/outgoingCalls`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#callHierarchy_outgoingCalls).
    ///
    /// Use [`Protocol::protocol_call_hierarchy_outgoing_calls`] for full params.
    pub fn call_hierarchy_outgoing_calls(
        &mut self,
        item: CallHierarchyItem,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.protocol_call_hierarchy_outgoing_calls(CallHierarchyOutgoingCallsParams {
            item,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`textDocument/prepareTypeHierarchy`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_prepareTypeHierarchy).
    ///
    /// Use [`Protocol::protocol_prepare_type_hierarchy`] for full params.
    pub fn prepare_type_hierarchy(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.protocol_prepare_type_hierarchy(TypeHierarchyPrepareParams {
            text_document_position_params: at(uri, position),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }

    /// Sends [`typeHierarchy/subtypes`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#typeHierarchy_subtypes).
    ///
    /// Use [`Protocol::protocol_type_hierarchy_subtypes`] for full params.
    pub fn type_hierarchy_subtypes(
        &mut self,
        item: TypeHierarchyItem,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.protocol_type_hierarchy_subtypes(TypeHierarchySubtypesParams {
            item,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`typeHierarchy/supertypes`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#typeHierarchy_supertypes).
    ///
    /// Use [`Protocol::protocol_type_hierarchy_supertypes`] for full params.
    pub fn type_hierarchy_supertypes(
        &mut self,
        item: TypeHierarchyItem,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        self.protocol_type_hierarchy_supertypes(TypeHierarchySupertypesParams {
            item,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
}
