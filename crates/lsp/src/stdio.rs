// vendored from lsp-server v0.10.0
use std::{
    io::{self, stdin},
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::vendored::read_msg_text;

pub enum Event {
    Message(Vec<u8>),
    Eof,
    ReadError(io::Error),
}

pub fn spawn_reader() -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || {
            let mut stdin = stdin().lock();
            let mut line = String::new();
            loop {
                let mut body = Vec::new();
                let event = match read_msg_text(&mut stdin, &mut line, &mut body) {
                    Ok(true) => Event::Message(body),
                    Ok(false) => Event::Eof,
                    Err(error) => Event::ReadError(error),
                };
                let last = !matches!(event, Event::Message(_));
                if sender.send(event).is_err() || last {
                    break;
                }
            }
        })
        .unwrap();
    receiver
}
