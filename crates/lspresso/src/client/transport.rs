use std::{
    io::{self, BufReader, Write},
    process::{Child, Command, ExitStatus, Stdio},
    thread::JoinHandle,
    time::Instant,
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use lsp_server::Message;

#[derive(Debug)]
pub(super) enum Incoming {
    Message(Message),
    /// The server wrote something that isn't a valid LSP frame. Nothing
    /// after it can be trusted to line up, so reading stops here.
    Malformed(io::Error),
    Disconnected,
}

/// Writes happen on their own thread so a server that stops reading stdin
/// can't block the caller: a large `didOpen` would otherwise wedge in
/// `write_all` once the pipe buffer fills, with no timeout able to fire.
#[derive(Debug)]
pub(super) struct Transport {
    child: Child,
    outgoing: Sender<Vec<u8>>,
    write_errors: Receiver<io::Error>,
    incoming: Receiver<Incoming>,
    _writer: JoinHandle<()>,
    _reader: JoinHandle<()>,
    _stderr: JoinHandle<()>,
}

impl Transport {
    pub(super) fn spawn(mut command: Command) -> crate::Result<Self> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|source| crate::error::ErrorKind::Spawn {
                program: command.get_program().to_string_lossy().into_owned(),
                source,
            })?;
        let mut stdin = child.stdin.take().expect("stdin is piped");
        let stdout = child.stdout.take().expect("stdout is piped");
        let mut stderr = child.stderr.take().expect("stderr is piped");

        let (outgoing, outgoing_rx) = crossbeam_channel::unbounded::<Vec<u8>>();
        let (write_error_tx, write_errors) = crossbeam_channel::bounded(1);
        let writer = std::thread::spawn(move || {
            for bytes in outgoing_rx {
                if let Err(err) = stdin.write_all(&bytes).and_then(|()| stdin.flush()) {
                    drop(write_error_tx.send(err));
                    break;
                }
            }
        });

        let (incoming_tx, incoming) = crossbeam_channel::unbounded();
        let reader = std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match Message::read(&mut reader) {
                    Ok(Some(message)) => {
                        if incoming_tx.send(Incoming::Message(message)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => {
                        drop(incoming_tx.send(Incoming::Disconnected));
                        return;
                    }
                    Err(err) => {
                        drop(incoming_tx.send(Incoming::Malformed(err)));
                        break;
                    }
                }
            }
            // Closing the channel lets waits fail fast, but stdout itself
            // stays drained: dropping it would kill the server with EPIPE on
            // its next write, just for having sent one bad frame.
            drop(incoming_tx);
            drop(io::copy(&mut reader, &mut io::sink()));
        });

        let stderr_thread = std::thread::spawn(move || {
            drop(io::copy(&mut stderr, &mut io::sink()));
        });

        Ok(Self {
            child,
            outgoing,
            write_errors,
            incoming,
            _writer: writer,
            _reader: reader,
            _stderr: stderr_thread,
        })
    }

    /// Queues `message` for the writer thread. A write that failed earlier
    /// is reported here, on the next send, since the failing one already
    /// returned.
    pub(super) fn send(&mut self, message: &Message) -> crate::Result<()> {
        if let Ok(err) = self.write_errors.try_recv() {
            return Err(err.into());
        }
        let mut bytes = Vec::new();
        message
            .write(&mut bytes)
            .expect("writing to a Vec cannot fail");
        self.outgoing.send(bytes).map_err(|_| {
            self.write_errors
                .try_recv()
                .unwrap_or_else(|_| io::ErrorKind::BrokenPipe.into())
                .into()
        })
    }

    /// The next incoming message, or a timeout once `deadline` passes; with
    /// no deadline, waits for a message or the stream closing.
    pub(super) fn recv_until(
        &self,
        deadline: Option<Instant>,
    ) -> Result<Incoming, RecvTimeoutError> {
        match deadline {
            Some(deadline) => self.incoming.recv_deadline(deadline),
            None => self
                .incoming
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
        }
    }

    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn try_wait(&mut self) -> crate::Result<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        // Killing first also unblocks a writer stuck on a full pipe.
        drop(self.child.kill());
        drop(self.child.wait());
    }
}
