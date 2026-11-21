use std::{
    error::Error,
    io::{self, BufWriter, StdoutLock, stdout},
    process::ExitCode,
};

use crate::{
    backend::Backend,
    lsp_server::{msg::write_msg_text, stdio::Event},
    transport::{Sink, Writer},
};

struct Stdout(BufWriter<StdoutLock<'static>>);

impl Sink for Stdout {
    fn emit(&mut self, body: &[u8]) -> io::Result<()> {
        write_msg_text(&mut self.0, body)
    }
}

pub fn run_blocking() -> Result<ExitCode, Box<dyn Error + Send + Sync>> {
    let events = crate::lsp_server::stdio::spawn_reader();
    let mut backend = Backend::new(Writer::new(Stdout(BufWriter::new(stdout().lock()))));
    loop {
        let body = match events.recv() {
            Ok(Event::Message(body)) => body,
            Ok(Event::ReadError(error)) => return Err(error.into()),
            Ok(Event::Eof) => return Ok(ExitCode::SUCCESS),
            Err(_) => return Ok(ExitCode::FAILURE),
        };
        if backend.handle(&body)?.is_break() {
            return Ok(ExitCode::from(backend.exit_code().unwrap_or(1)));
        }
    }
}
