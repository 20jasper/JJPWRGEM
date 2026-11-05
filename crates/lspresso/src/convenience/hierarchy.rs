use gen_lsp_types::*;

use super::at;
use crate::{Client, Protocol, Result};

impl Client {
    /// Finds the call hierarchy items at `position` in `uri`.
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

    /// Finds the calls to `item`.
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

    /// Finds the calls `item` makes.
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

    /// Finds the type hierarchy items at `position` in `uri`.
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

    /// Finds the subtypes of `item`.
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

    /// Finds the supertypes of `item`.
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
