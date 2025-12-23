use core::fmt::Debug;
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

impl Output {
    pub fn snapshot_display(&self) -> String {
        format!(
            r#"args: {:?}
status: {}
success: {}
stdin ---
{}
stdout ---
{}
stderr ---
{}"#,
            self.args,
            self.status.code().unwrap_or(-1),
            self.status.success(),
            self.stdin,
            self.stdout,
            self.stderr
        )
    }

    pub fn docs_display_stdin(&self) -> String {
        format!(
            "$ echo -en {:?} | jjp {}\n{}{}",
            self.stdin,
            self.args.join(" "),
            self.stdout,
            self.stderr
        )
    }
}

pub fn exec_cmd(cmd: &mut Command, stdin: Option<Vec<u8>>) -> Output {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("test command failed");

    if let Some(stdin) = &stdin {
        child
            .stdin
            .take()
            .expect("should have stdin")
            .write_all(stdin)
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait on child");

    let fmt_bytes = |xs: Option<Vec<u8>>| {
        if let Some(xs) = xs {
            String::from_utf8(xs.clone()).unwrap_or_else(|_| format!("raw bytes: {xs:?}"))
        } else {
            "<no stdin passed>".into()
        }
    };

    Output {
        args: cmd
            .get_args()
            .map(|x| x.to_str().unwrap().to_string())
            .collect::<Vec<_>>(),
        stdin: fmt_bytes(stdin),
        stdout: fmt_bytes(output.stdout.into()),
        stderr: fmt_bytes(output.stderr.into()),
        status: output.status,
    }
}

#[rstest_reuse::template]
#[rstest::rstest]
#[case(crate::fixture_tuple!(VALID_FRACTION))]
#[case(crate::fixture_tuple!(VALID_NEGATIVE_FRACTION))]
#[case(crate::fixture_tuple!(VALID_INTEGER))]
#[case(crate::fixture_tuple!(VALID_NEGATIVE_INTEGER))]
#[case(crate::fixture_tuple!(LONG_INTEGER))]
#[case(crate::fixture_tuple!(LONG_FRACTION))]
#[case(crate::fixture_tuple!(EXPONENT_WITH_PLUS_SIGN))]
#[case(crate::fixture_tuple!(EXPONENT_WITH_MINUS_SIGN))]
#[case(crate::fixture_tuple!(NEGATIVE_FLOAT_WITH_EXPONENT))]
#[case(crate::fixture_tuple!(ARRAY_EMPTY))]
#[case(crate::fixture_tuple!(ARRAY_SINGLE))]
#[case(crate::fixture_tuple!(ARRAY_MANY))]
#[case(crate::fixture_tuple!(ARRAY_SUBARRAYS))]
#[case(crate::fixture_tuple!(ARRAY_OBJECTS_WITH_INCREASING_KEYS))]
#[case(crate::fixture_tuple!(ARRAY_MULTIPLE_EMPTY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_SINGLE_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_TWO_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAY_MANY_FIVE_KEY_OBJECTS))]
#[case(crate::fixture_tuple!(ARRAYS_NESTED_FIVE_LEVELS_WITH_OBJECT))]
#[case(crate::fixture_tuple!(STANDALONE_NULL))]
#[case(crate::fixture_tuple!(STANDALONE_FALSE))]
#[case(crate::fixture_tuple!(STANDALONE_TRUE))]
#[case(crate::fixture_tuple!(STANDALONE_STRING))]
#[case(crate::fixture_tuple!(NESTED_OBJECT_SINGLE_KEY))]
#[case(crate::fixture_tuple!(OBJECT_WITH_LONG_KEYS))]
#[case(crate::fixture_tuple!(ARRAY_WITH_NESTED_OBJECTS))]
#[case(crate::fixture_tuple!(MIXED_ARRAY_WITH_LONG_STRINGS))]
#[case(crate::fixture_tuple!(STANDALONE_STRING_WS))]
#[case(crate::fixture_tuple!(DEEPLY_NESTED))]
#[case(crate::fixture_tuple!(OBJECT_WITH_LONG_KEY_AND_ARR_VAL))]
#[case(crate::fixture_tuple!(OBJECT_WITH_EXPANDED_AND_NON_EXPANDED_ARR))]
#[case(crate::fixture_tuple!(DEEPLY_NESTED_OBJECT_WITH_ARR_VALUES))]
#[case(crate::fixture_tuple!(TSCONFIG))]
pub fn format_template(#[case] (name, input): (&str, &str)) {}
