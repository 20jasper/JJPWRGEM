use std::{io, ops::ControlFlow};

use gen_lsp_types::ErrorCodes;

use crate::{
    backend::Backend,
    lsp_server::req_queue::Id,
    transport::{Rejection, Sink, decode},
};

impl<S: Sink> Backend<S> {
    pub fn handle(&mut self, body: &[u8]) -> io::Result<ControlFlow<()>> {
        let message = match decode(body) {
            Ok(message) => message,
            Err(rejection) => {
                self.reject(rejection)?;
                return Ok(ControlFlow::Continue(()));
            }
        };
        match (message.id, message.method) {
            (Some(id), Some(method)) => self
                .request(id, &method, message.params)
                .map(ControlFlow::Continue),
            (None, Some(method)) => Ok(self.notification(&method, message.params)),
            (Some(id), None) => {
                self.response(&id);
                Ok(ControlFlow::Continue(()))
            }
            (None, None) => self
                .reject(Rejection {
                    id: Id::Null,
                    code: ErrorCodes::InvalidRequest,
                    message: "invalid request: missing field `method`".into(),
                })
                .map(ControlFlow::Continue),
        }
    }
}
