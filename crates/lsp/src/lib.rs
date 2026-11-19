mod backend;
mod lsp_server;
mod range;
mod transport;

use std::{error::Error, process::ExitCode};

use backend::Backend;
use lsp_server::{msg::ErrorCode, req_queue::Id, stdio::Event};
use transport::{Rejection, Writer, decode};

pub fn run_blocking() -> Result<ExitCode, Box<dyn Error + Send + Sync>> {
    let events = lsp_server::stdio::spawn_reader();
    let mut backend = Backend::new(Writer::new());
    loop {
        let body = match events.recv() {
            Ok(Event::Message(body)) => body,
            Ok(Event::ReadError(error)) => return Err(error.into()),
            Ok(Event::Eof) => return Ok(ExitCode::SUCCESS),
            Err(_) => return Ok(ExitCode::FAILURE),
        };
        let message = match decode(&body) {
            Ok(message) => message,
            Err(rejection) => {
                backend.reject(rejection)?;
                continue;
            }
        };
        match (message.id, message.method) {
            (Some(id), Some(method)) => backend.request(id, &method, message.params)?,
            (None, Some(method)) => {
                if backend.notification(&method, message.params).is_break() {
                    return Ok(backend.exit_code());
                }
            }
            (Some(id), None) => backend.response(&id),
            (None, None) => backend.reject(Rejection {
                id: Id::Null,
                code: ErrorCode::InvalidRequest,
                message: "invalid request: missing field `method`".into(),
            })?,
        }
    }
}
