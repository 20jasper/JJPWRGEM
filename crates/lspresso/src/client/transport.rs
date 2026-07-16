use std::{
    fmt,
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use lsp_server::Message;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::error::{ErrorKind, Result as LspressoResult};

#[derive(Debug)]
pub(crate) enum Error {
    Io(io::Error),
    Timeout,
    Disconnected(Option<i32>),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

struct KillOnDrop(Child);

impl KillOnDrop {
    fn id(&self) -> u32 {
        self.0.id()
    }

    fn try_exit_code(&mut self) -> Option<i32> {
        self.0.try_wait().ok()?.and_then(|s| s.code())
    }
}

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

struct JoinOnDrop<T>(Option<JoinHandle<T>>);

impl<T> JoinOnDrop<T> {
    fn new(handle: JoinHandle<T>) -> Self {
        Self(Some(handle))
    }
}

impl<T> Drop for JoinOnDrop<T> {
    fn drop(&mut self) {
        if let Some(t) = self.0.take() {
            drop(t.join());
        }
    }
}

impl<T> fmt::Debug for JoinOnDrop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JoinOnDrop")
            .field(&self.0.as_ref().map(|_| ".."))
            .finish()
    }
}

#[derive(Debug)]
struct WriteChannel {
    tx: Sender<Vec<u8>>,
    err_rx: Receiver<io::Error>,
}

impl WriteChannel {
    fn send(&self, buf: Vec<u8>) -> Result<()> {
        if let Ok(e) = self.err_rx.try_recv() {
            return Err(Error::Io(e));
        }
        self.tx.send(buf).map_err(|_| {
            self.err_rx
                .try_recv()
                .map_or(Error::Disconnected(None), Error::Io)
        })
    }
}

#[derive(Debug)]
pub(super) struct Transport {
    msg_rx: Receiver<Message>,
    rss_monitor: System,
    write_channel: WriteChannel,
    process: KillOnDrop,
    _write_thread: JoinOnDrop<()>,
    _read_thread: JoinOnDrop<()>,
    _stderr_thread: Option<JoinOnDrop<()>>,
}

impl Transport {
    pub fn spawn(mut cmd: Command) -> LspressoResult<Self> {
        let trace = tracing::enabled!(tracing::Level::TRACE);
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if trace { Stdio::piped() } else { Stdio::null() })
            .spawn()
            .map_err(|e| {
                let program = cmd.get_program().to_string_lossy().into_owned();
                ErrorKind::Io(io::Error::new(e.kind(), format!("{program}: {e}")))
            })?;

        let stdin = child.stdin.take().expect("stdin was requested as piped");
        let stdout = child.stdout.take().expect("stdout was requested as piped");

        let (msg_tx, msg_rx) = crossbeam_channel::unbounded();
        let (write_tx, write_rx) = crossbeam_channel::unbounded::<Vec<u8>>();
        let (write_err_tx, write_err_rx) = crossbeam_channel::bounded::<io::Error>(1);

        let write_thread = JoinOnDrop::new(std::thread::spawn(move || {
            let mut stdin = stdin;
            for bytes in write_rx {
                if let Err(e) = stdin.write_all(&bytes).and_then(|()| stdin.flush()) {
                    drop(write_err_tx.send(e));
                    break;
                }
            }
        }));

        let reader_thread = JoinOnDrop::new(std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                match Message::read(&mut stdout) {
                    Ok(Some(msg)) => {
                        if msg_tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tracing::warn!(error = %e, "server sent malformed LSP frame");
                        break;
                    }
                }
            }
        }));

        let stderr_thread = child.stderr.take().map(|stderr| {
            JoinOnDrop::new(std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(io::Result::ok) {
                    tracing::trace!(target: "lspresso::server", "{line}");
                }
            }))
        });

        Ok(Self {
            msg_rx,
            rss_monitor: System::new(),
            write_channel: WriteChannel {
                tx: write_tx,
                err_rx: write_err_rx,
            },
            process: KillOnDrop(child),
            _write_thread: write_thread,
            _read_thread: reader_thread,
            _stderr_thread: stderr_thread,
        })
    }

    pub fn send(&mut self, msg: &Message) -> Result<()> {
        let mut buf = Vec::new();
        msg.write(&mut buf).expect("vec writes can't fail");
        self.write_channel.send(buf).map_err(|e| match e {
            Error::Io(_) => e,
            Error::Disconnected(_) => Error::Disconnected(self.process.try_exit_code()),
            Error::Timeout => unreachable!("write channel cannot timeout"),
        })
    }

    pub fn recv_deadline(&mut self, deadline: Instant) -> Result<Message> {
        self.msg_rx
            .recv_deadline(deadline)
            .map_err(|err| match err {
                RecvTimeoutError::Timeout => Error::Timeout,
                RecvTimeoutError::Disconnected => Error::Disconnected(self.process.try_exit_code()),
            })
    }

    /// Polls until the process exits or `deadline` passes. Returns its exit
    /// code, or `None` if it hasn't exited yet, or the OS couldn't report a
    /// code (e.g. killed by signal).
    pub fn wait_for_exit(&mut self, deadline: Instant) -> Option<i32> {
        loop {
            if let Some(code) = self.process.try_exit_code() {
                return Some(code);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            std::thread::sleep(remaining.min(Duration::from_millis(10)));
        }
    }

    pub fn current_rss_bytes(&mut self) -> Option<u64> {
        let pid = Pid::from_u32(self.process.id());
        self.rss_monitor.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        self.rss_monitor.process(pid).map(sysinfo::Process::memory)
    }
}
