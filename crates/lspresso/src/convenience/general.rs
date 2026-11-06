use gen_lsp_types::*;

use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`$/cancelRequest`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#cancelRequest).
    ///
    /// Use [`Protocol::protocol_cancel_request`] for full params.
    pub fn cancel_request(&mut self, id: Id) -> Result<()> {
        self.protocol_cancel_request(CancelParams { id })
    }

    /// Sends [`$/progress`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#progress).
    ///
    /// Use [`Protocol::protocol_progress`] for full params.
    pub fn progress(&mut self, token: ProgressToken, value: LspAny) -> Result<()> {
        self.protocol_progress(ProgressParams { token, value })
    }

    /// Sends [`$/setTrace`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#setTrace).
    ///
    /// Use [`Protocol::protocol_set_trace`] for full params.
    pub fn set_trace(&mut self, value: TraceValue) -> Result<()> {
        self.protocol_set_trace(SetTraceParams { value })
    }

    /// Sends [`window/workDoneProgress/cancel`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#window_workDoneProgress_cancel).
    ///
    /// Use [`Protocol::protocol_window_work_done_progress_cancel`] for full params.
    pub fn window_work_done_progress_cancel(
        &mut self,
        params: WorkDoneProgressCancelParams,
    ) -> Result<()> {
        self.protocol_window_work_done_progress_cancel(params)
    }
}
