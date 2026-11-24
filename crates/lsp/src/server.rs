use std::{fmt, io, mem};

use crate::{
    backend::Backend,
    transport::{Sink, Writer},
};

#[derive(Default)]
struct Collected(Vec<String>);

impl Sink for Collected {
    fn emit(&mut self, body: &[u8]) -> io::Result<()> {
        self.0.push(String::from_utf8_lossy(body).into_owned());
        Ok(())
    }
}

pub struct Server {
    backend: Backend<Collected>,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Server").finish_non_exhaustive()
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            backend: Backend::new(Writer::new(Collected::default())),
        }
    }

    pub fn handle(&mut self, message: &str) -> Vec<String> {
        drop(self.backend.handle(message.as_bytes()));
        mem::take(&mut self.backend.sink_mut().0)
    }
}
