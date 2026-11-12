use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use lsp_server::Message;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug)]
pub(super) enum Incoming {
    Message(Message),
    Malformed(io::Error),
}

#[derive(Debug)]
struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

/// writes are on another thread to avoid
/// blocikg the main thread
#[derive(Debug)]
pub(super) struct Transport {
    process: KillOnDrop,
    outgoing: Sender<Vec<u8>>,
    write_errors: Receiver<io::Error>,
    incoming: Receiver<(Instant, Incoming)>,
    rss_monitor: Option<System>,
    waiting_since: Option<Instant>,
    server_time: Duration,
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn read_body(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let Some(header) = line.strip_suffix("\r\n") else {
            return Err(invalid_data(format!("malformed header: {line:?}")));
        };
        if header.is_empty() {
            break;
        }
        let Some((name, value)) = header.split_once(": ") else {
            return Err(invalid_data(format!("malformed header: {line:?}")));
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            length = Some(
                value
                    .parse::<usize>()
                    .map_err(|err| invalid_data(err.to_string()))?,
            );
        }
    }
    let length = length.ok_or_else(|| invalid_data("no Content-Length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
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
        std::thread::spawn(move || {
            for bytes in outgoing_rx {
                if let Err(err) = stdin.write_all(&bytes).and_then(|()| stdin.flush()) {
                    drop(write_error_tx.send(err));
                    break;
                }
            }
        });

        let (incoming_tx, incoming) = crossbeam_channel::unbounded();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let body = match read_body(&mut reader) {
                    Ok(Some(body)) => body,
                    Ok(None) => return,
                    Err(err) => {
                        drop(incoming_tx.send((Instant::now(), Incoming::Malformed(err))));
                        break;
                    }
                };
                let arrived_at = Instant::now();
                match serde_json::from_slice(&body) {
                    Ok(message) => {
                        if incoming_tx
                            .send((arrived_at, Incoming::Message(message)))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(err) => {
                        let err = invalid_data(format!(
                            "malformed LSP payload `{err:?}`: {}",
                            String::from_utf8_lossy(&body)
                        ));
                        drop(incoming_tx.send((arrived_at, Incoming::Malformed(err))));
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

        Ok(Self {
            process: KillOnDrop(child),
            outgoing,
            write_errors,
            incoming,
            rss_monitor: System::new().ok(),
            waiting_since: None,
            server_time: Duration::ZERO,
        })
    }

    pub(super) const fn server_time(&self) -> Duration {
        self.server_time
    }

    /// Queues `message` or returns existing errors
    pub(super) fn send(&mut self, message: &Message) -> crate::Result<()> {
        if let Ok(err) = self.write_errors.try_recv() {
            return Err(err.into());
        }
        let mut bytes = Vec::new();
        message
            .write(&mut bytes)
            .expect("writing to a Vec cannot fail");
        let queued_at = Instant::now();
        self.outgoing.send(bytes).map_err(|_| {
            crate::Error::from(
                self.write_errors
                    .try_recv()
                    .unwrap_or_else(|_| io::ErrorKind::BrokenPipe.into()),
            )
        })?;
        self.waiting_since = Some(queued_at);
        Ok(())
    }

    pub(super) fn recv_until(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<Incoming, RecvTimeoutError> {
        let (arrived_at, incoming) = match deadline {
            Some(deadline) => self.incoming.recv_deadline(deadline),
            None => self
                .incoming
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
        }?;
        if let Some(waiting_since) = self.waiting_since {
            self.server_time += arrived_at.saturating_duration_since(waiting_since);
            self.waiting_since = Some(waiting_since.max(arrived_at));
        }
        Ok(incoming)
    }

    pub(super) fn try_wait(&mut self) -> crate::Result<Option<ExitStatus>> {
        Ok(self.process.0.try_wait()?)
    }

    pub(super) fn rss_bytes(&mut self) -> Option<u64> {
        let pid = Pid::from_u32(self.process.0.id());
        let monitor = self.rss_monitor.as_mut()?;
        monitor.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        monitor.process(pid).map(sysinfo::Process::memory)
    }
}
