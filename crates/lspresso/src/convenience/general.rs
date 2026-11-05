use gen_lsp_types::*;

use crate::{Client, Protocol, Result};

impl Client {
    /// Asks the server to cancel the request with `id`.
    ///
    /// Use [`Protocol::protocol_cancel_request`] for full params.
    pub fn cancel_request(&mut self, id: Id) -> Result<()> {
        self.protocol_cancel_request(CancelParams { id })
    }

    /// Reports `value` as progress for `token`.
    ///
    /// Use [`Protocol::protocol_progress`] for full params.
    pub fn progress(&mut self, token: ProgressToken, value: LspAny) -> Result<()> {
        self.protocol_progress(ProgressParams { token, value })
    }

    /// Sets the server's trace level to `value`.
    ///
    /// Use [`Protocol::protocol_set_trace`] for full params.
    pub fn set_trace(&mut self, value: TraceValue) -> Result<()> {
        self.protocol_set_trace(SetTraceParams { value })
    }

    /// Cancels work done progress the server started.
    pub fn window_work_done_progress_cancel(
        &mut self,
        params: WorkDoneProgressCancelParams,
    ) -> Result<()> {
        self.protocol_window_work_done_progress_cancel(params)
    }
}
