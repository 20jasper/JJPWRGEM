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
    NullIdResponse,
    Malformed(io::Error),
}

fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let invalid = |message: &str| io::Error::new(io::ErrorKind::InvalidData, message.to_owned());
    let mut length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let Some(line) = line.strip_suffix("\r\n") else {
            return Err(invalid("malformed header"));
        };
        if line.is_empty() {
            break;
        }
        let (name, value) = line
            .split_once(": ")
            .ok_or_else(|| invalid("malformed header"))?;
        if name.eq_ignore_ascii_case("Content-Length") {
            length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| invalid("bad Content-Length"))?,
            );
        }
    }
    let mut body = vec![0; length.ok_or_else(|| invalid("no Content-Length"))?];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

fn parse_incoming(body: &[u8]) -> Incoming {
    match serde_json::from_slice::<Message>(body) {
        Ok(message) => Incoming::Message(message),
        Err(_) if is_null_id_response(body) => Incoming::NullIdResponse,
        Err(err) => Incoming::Malformed(io::Error::new(io::ErrorKind::InvalidData, err)),
    }
}

fn is_null_id_response(body: &[u8]) -> bool {
    let Ok(serde_json::Value::Object(fields)) = serde_json::from_slice(body) else {
        return false;
    };
    fields.get("id").is_some_and(serde_json::Value::is_null)
        && (fields.contains_key("result") || fields.contains_key("error"))
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
                drop(reader.fill_buf());
                let first_byte_at = Instant::now();
                let incoming = match read_frame(&mut reader) {
                    Ok(Some(body)) => parse_incoming(&body),
                    Ok(None) => return,
                    Err(err) => Incoming::Malformed(err),
                };
                let malformed = matches!(incoming, Incoming::Malformed(_));
                let arrival = Arrival {
                    first_byte_at,
                    parsed_at: Instant::now(),
                    incoming,
                };
                if incoming_tx.send(arrival).is_err() {
                    return;
                }
                if malformed {
                    break;
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
        let sequence = self.stopwatch.sent(started_at, queued_at);
        self.outgoing.send((sequence, bytes)).map_err(|_| {
            self.write_errors
                .try_recv()
                .unwrap_or_else(|_| io::ErrorKind::BrokenPipe.into())
                .into()
        })
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
