use core::fmt::Display;
use std::{
    env, fs,
    io::Write,
    process::{Command, ExitStatus, Stdio},
};

#[macro_export]
macro_rules! fixture_tuple {
    ($const:ident) => {
        (stringify!($const), $const)
    };
}

pub fn cli() -> Command {
    let exe = env!("CARGO_BIN_EXE_jjp");
    assert!(
        fs::exists(exe).unwrap_or_default(),
        "couldn't find executable, did you forget to build?"
    );
    Command::new(exe)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub args: Vec<String>,
    pub stdin: String,
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "args: {:?}\nstatus: {}\nsuccess: {}\nstdout ---\n{}\nstderr ---\n{}",
            self.args,
            self.status.code().unwrap_or(-1),
            self.status.success(),
            self.stdin,
            self.stderr
        )
    }
}

pub fn exec_cmd(cmd: &mut Command, stdin: Vec<u8>) -> Output {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("test command failed");

    child
        .stdin
        .take()
        .expect("should have stdin")
        .write_all(&stdin)
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait on child");

    let fmt_bytes = |xs: Vec<u8>| {
        String::from_utf8(xs.clone()).unwrap_or_else(|_| format!("raw bytes: {xs:?}"))
    };

    Output {
        args: cmd
            .get_args()
            .map(|x| x.to_str().unwrap().to_string())
            .collect::<Vec<_>>(),
        stdin: fmt_bytes(stdin),
        stdout: fmt_bytes(output.stdout),
        stderr: fmt_bytes(output.stderr),
        status: output.status,
    }
}
