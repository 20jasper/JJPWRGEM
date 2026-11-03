use std::{
    fmt,
    io::{self, BufReader, Write},
    process::{Child, Command, ExitStatus, Stdio},
    thread::JoinHandle,
    time::Instant,
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use lsp_server::Message;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug)]
pub(super) enum Incoming {
    Message(Message),
    /// The server wrote something that isn't a valid LSP frame. Nothing
    /// after it can be trusted to line up, so reading stops here.
    Malformed(io::Error),
    Disconnected,
}

/// Kills and reaps the server on drop.
struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

impl fmt::Debug for KillOnDrop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("KillOnDrop").field(&self.0.id()).finish()
    }
}

struct JoinOnDrop(Option<JoinHandle<()>>);

impl Drop for JoinOnDrop {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            drop(handle.join());
        }
    }
}

impl fmt::Debug for JoinOnDrop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JoinOnDrop")
            .field(&self.0.as_ref().map(|_| ".."))
            .finish()
    }
}

/// Writes happen on their own thread so a server that stops reading stdin
/// can't block the caller: a large `didOpen` would otherwise wedge in
/// `write_all` once the pipe buffer fills, with no timeout able to fire.
///
/// Field order matters for drop: the process is killed first, then the
/// sender is dropped so the writer can exit, and only then are the threads
/// joined.
#[derive(Debug)]
pub(super) struct Transport {
    process: KillOnDrop,
    outgoing: Sender<Vec<u8>>,
    write_errors: Receiver<io::Error>,
    incoming: Receiver<Incoming>,
    rss_monitor: System,
    _writer: JoinOnDrop,
    _reader: JoinOnDrop,
}

impl Transport {
    pub(super) fn spawn(mut command: Command) -> crate::Result<Self> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .map_err(|source| crate::error::ErrorKind::Spawn {
                program: command.get_program().to_string_lossy().into_owned(),
                source,
            })?;
        let mut stdin = child.stdin.take().expect("stdin is piped");
        let stdout = child.stdout.take().expect("stdout is piped");

        let (outgoing, outgoing_rx) = crossbeam_channel::unbounded::<Vec<u8>>();
        let (write_error_tx, write_errors) = crossbeam_channel::bounded(1);
        let writer = JoinOnDrop(Some(std::thread::spawn(move || {
            for bytes in outgoing_rx {
                if let Err(err) = stdin.write_all(&bytes).and_then(|()| stdin.flush()) {
                    drop(write_error_tx.send(err));
                    break;
                }
            }
        })));

        let (incoming_tx, incoming) = crossbeam_channel::unbounded();
        let reader = JoinOnDrop(Some(std::thread::spawn(move || {
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
        })));

        Ok(Self {
            process: KillOnDrop(child),
            outgoing,
            write_errors,
            incoming,
            rss_monitor: System::new(),
            _writer: writer,
            _reader: reader,
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

    pub(super) fn try_wait(&mut self) -> crate::Result<Option<ExitStatus>> {
        Ok(self.process.0.try_wait()?)
    }

    pub(super) fn rss_bytes(&mut self) -> Option<u64> {
        let pid = Pid::from_u32(self.process.0.id());
        self.rss_monitor.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        self.rss_monitor.process(pid).map(sysinfo::Process::memory)
    }
}
