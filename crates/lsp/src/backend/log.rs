use gen_lsp_types::*;

use crate::{
    backend::{Backend, SOURCE},
    transport::Sink,
};

impl<S: Sink> Backend<S> {
    pub(super) fn error(&mut self, message: impl Into<String>) {
        self.log_message(MessageType::Error, message);
    }

    pub(super) fn warn(&mut self, message: impl Into<String>) {
        self.log_message(MessageType::Warning, message);
    }

    pub(super) fn info(&mut self, message: impl Into<String>) {
        self.log_message(MessageType::Info, message);
    }

    fn log_message(&mut self, kind: MessageType, message: impl Into<String>) {
        self.out.notify::<LogMessageNotification>(LogMessageParams {
            kind,
            message: message.into(),
        });
    }

    pub(super) fn show_warning(&mut self, message: &str) {
        self.out
            .notify::<ShowMessageNotification>(ShowMessageParams {
                kind: MessageType::Warning,
                message: format!("{SOURCE}: {message}"),
            });
    }
}
