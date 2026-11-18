mod backend;
mod lsp_server;
mod range;
mod transport;

use std::{error::Error, process::ExitCode};

use backend::Backend;
use lsp_server::stdio::Event;
use transport::{Reader, Writer, decode};

#[doc(hidden)]
pub fn decode_stream(bytes: &[u8]) {
    let mut reader = Reader::from_input(bytes);
    while reader.read_body().unwrap_or(false) {
        drop(reader.message());
    }
}

pub fn run_blocking() -> Result<ExitCode, Box<dyn Error + Send + Sync>> {
    let events = lsp_server::stdio::spawn_reader();
    let mut backend = Backend::new(Writer::new());
    loop {
        let body = match events.recv() {
            Ok(Event::Message(body)) => body,
            Ok(Event::ReadError(error)) => return Err(error.into()),
            Ok(Event::Eof) | Err(_) => return Ok(ExitCode::SUCCESS),
        };
        let message = match decode(&body) {
            Ok(message) => message,
            Err(rejection) => {
                backend.reject(rejection)?;
                continue;
            }
        };
        if message.method.is_empty() {
            if let Some(id) = message.id {
                backend.response(&id);
            }
            continue;
        }
        match message.id {
            Some(id) => backend.request(id, &message.method, message.params)?,
            None => {
                if backend
                    .notification(&message.method, message.params)
                    .is_break()
                {
                    return Ok(backend.exit_code());
                }
            }
        }
    }
}
