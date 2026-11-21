mod backend;
#[cfg_attr(
    target_arch = "wasm32",
    expect(dead_code, reason = "stdio framing is native only")
)]
mod lsp_server;
mod range;
mod server;
mod transport;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    error::Error,
    io::{BufWriter, StdoutLock, stdout},
    process::ExitCode,
};
use std::{io, ops::ControlFlow};

use backend::Backend;
use gen_lsp_types::ErrorCodes;
use lsp_server::req_queue::Id;
#[cfg(not(target_arch = "wasm32"))]
use lsp_server::{msg::write_msg_text, stdio::Event};
pub use server::Server;
#[cfg(not(target_arch = "wasm32"))]
use transport::Writer;
use transport::{Rejection, Sink, decode};

#[cfg(not(target_arch = "wasm32"))]
struct Stdout(BufWriter<StdoutLock<'static>>);

#[cfg(not(target_arch = "wasm32"))]
impl Sink for Stdout {
    fn emit(&mut self, body: &[u8]) -> io::Result<()> {
        write_msg_text(&mut self.0, body)
    }
}

fn dispatch<S: Sink>(backend: &mut Backend<S>, body: &[u8]) -> io::Result<ControlFlow<()>> {
    let message = match decode(body) {
        Ok(message) => message,
        Err(rejection) => {
            backend.reject(rejection)?;
            return Ok(ControlFlow::Continue(()));
        }
    };
    match (message.id, message.method) {
        (Some(id), Some(method)) => backend
            .request(id, &method, message.params)
            .map(ControlFlow::Continue),
        (None, Some(method)) => Ok(backend.notification(&method, message.params)),
        (Some(id), None) => {
            backend.response(&id);
            Ok(ControlFlow::Continue(()))
        }
        (None, None) => backend
            .reject(Rejection {
                id: Id::Null,
                code: ErrorCodes::InvalidRequest,
                message: "invalid request: missing field `method`".into(),
            })
            .map(ControlFlow::Continue),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_blocking() -> Result<ExitCode, Box<dyn Error + Send + Sync>> {
    let events = lsp_server::stdio::spawn_reader();
    let mut backend = Backend::new(Writer::new(Stdout(BufWriter::new(stdout().lock()))));
    loop {
        let body = match events.recv() {
            Ok(Event::Message(body)) => body,
            Ok(Event::ReadError(error)) => return Err(error.into()),
            Ok(Event::Eof) => return Ok(ExitCode::SUCCESS),
            Err(_) => return Ok(ExitCode::FAILURE),
        };
        if dispatch(&mut backend, &body)?.is_break() {
            return Ok(backend.exit_code());
        }
    }
}
