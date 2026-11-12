use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, ExitStatus, Stdio},
    time::Instant,
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use lsp_server::Message;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use super::timings::{Stopwatch, Timings};

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
    outgoing: Sender<(u64, Vec<u8>)>,
    write_errors: Receiver<io::Error>,
    written: Receiver<(u64, Instant)>,
    incoming: Receiver<Arrival>,
    rss_monitor: Option<System>,
    stopwatch: Stopwatch,
}

#[derive(Debug)]
struct Arrival {
    first_byte_at: Instant,
    parsed_at: Instant,
    incoming: Incoming,
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

        let (outgoing, outgoing_rx) = crossbeam_channel::unbounded::<(u64, Vec<u8>)>();
        let (write_error_tx, write_errors) = crossbeam_channel::bounded(1);
        let (written_tx, written) = crossbeam_channel::unbounded();
        std::thread::spawn(move || {
            for (sequence, bytes) in outgoing_rx {
                if let Err(err) = stdin.write_all(&bytes).and_then(|()| stdin.flush()) {
                    drop(write_error_tx.send(err));
                    break;
                }
                if written_tx.send((sequence, Instant::now())).is_err() {
                    break;
                }
            }
        });

        let (incoming_tx, incoming) = crossbeam_channel::unbounded();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let first_byte_at = match reader.fill_buf() {
                    Ok([]) => return,
                    Ok(_) => Instant::now(),
                    Err(err) => {
                        let now = Instant::now();
                        drop(incoming_tx.send(Arrival {
                            first_byte_at: now,
                            parsed_at: now,
                            incoming: Incoming::Malformed(err),
                        }));
                        break;
                    }
                };
                let body = read_body(&mut reader);
                let incoming = match body {
                    Ok(Some(body)) => serde_json::from_slice(&body).map_or_else(
                        |err| {
                            Incoming::Malformed(invalid_data(format!(
                                "malformed LSP payload `{err:?}`: {}",
                                String::from_utf8_lossy(&body)
                            )))
                        },
                        Incoming::Message,
                    ),
                    Ok(None) => return,
                    Err(err) => Incoming::Malformed(err),
                };
                let parsed_at = Instant::now();
                let malformed = matches!(incoming, Incoming::Malformed(_));
                let delivered = incoming_tx.send(Arrival {
                    first_byte_at,
                    parsed_at,
                    incoming,
                });
                if malformed {
                    break;
                }
                if delivered.is_err() {
                    return;
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
            written,
            incoming,
            rss_monitor: System::new().ok(),
            stopwatch: Stopwatch::default(),
        })
    }

    pub(super) const fn timings(&self) -> Timings {
        self.stopwatch.timings()
    }

    /// Queues `message` or returns existing errors
    pub(super) fn send(&mut self, message: &Message, started_at: Instant) -> crate::Result<()> {
        if let Ok(err) = self.write_errors.try_recv() {
            return Err(err.into());
        }
        let mut bytes = Vec::new();
        message
            .write(&mut bytes)
            .expect("writing to a Vec cannot fail");
        let queued_at = Instant::now();
        self.outgoing
            .send((self.stopwatch.next_sequence(), bytes))
            .map_err(|_| {
                crate::Error::from(
                    self.write_errors
                        .try_recv()
                        .unwrap_or_else(|_| io::ErrorKind::BrokenPipe.into()),
                )
            })?;
        self.stopwatch.sent(started_at, queued_at);
        Ok(())
    }

    pub(super) fn recv_until(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<Incoming, RecvTimeoutError> {
        let Arrival {
            first_byte_at,
            parsed_at,
            incoming,
        } = match deadline {
            Some(deadline) => self.incoming.recv_deadline(deadline),
            None => self
                .incoming
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
        }?;
        for (sequence, at) in self.written.try_iter() {
            self.stopwatch.written(sequence, at);
        }
        self.stopwatch.arrived(first_byte_at, parsed_at);
        Ok(incoming)
    }

    pub(super) fn settle(&mut self, call_started_at: Instant) {
        self.stopwatch.settle(call_started_at);
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
