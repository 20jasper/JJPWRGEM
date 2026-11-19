use core::fmt::Debug;
use std::{
    env, fs,
    io::Write,
    process::{Command, ExitStatus, Stdio},
};

macro_rules! fixture_tuple {
    ($const:ident) => {
        (stringify!($const), $const)
    };
}

pub(crate) use fixture_tuple;

macro_rules! cli_snapshot {
    ($($args:tt)*) => {
        ::insta::with_settings!({snapshot_path => "../snapshots"}, {
            ::insta::assert_snapshot!($($args)*);
        })
    };
}

pub(crate) use cli_snapshot;

macro_rules! assert_cli_snapshot {
    ($name:expr, $args:expr, $stdin:expr, $success:expr) => {{
        let stdin: Option<&[u8]> = $stdin;
        let output =
            $crate::common::exec_cmd($crate::common::cli().args($args), stdin.map(<[u8]>::to_vec));
        assert_eq!(
            output.status.success(),
            $success,
            "{}",
            output.snapshot_display()
        );
        $crate::common::cli_snapshot!($name, output.snapshot_display());
    }};
}

pub(crate) use assert_cli_snapshot;

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
            r"args: {:?}
status: {}
success: {}
stdin ---
{}
stdout ---
{}
stderr ---
{}",
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
        match child
            .stdin
            .take()
            .expect("should have stdin")
            .write_all(stdin)
        {
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
            result => result.expect("failed to write to stdin"),
        }
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
#[case(crate::common::fixture_tuple!(VALID_FRACTION))]
#[case(crate::common::fixture_tuple!(VALID_NEGATIVE_FRACTION))]
#[case(crate::common::fixture_tuple!(VALID_INTEGER))]
#[case(crate::common::fixture_tuple!(VALID_NEGATIVE_INTEGER))]
#[case(crate::common::fixture_tuple!(LONG_INTEGER))]
#[case(crate::common::fixture_tuple!(LONG_FRACTION))]
#[case(crate::common::fixture_tuple!(EXPONENT_WITH_PLUS_SIGN))]
#[case(crate::common::fixture_tuple!(EXPONENT_WITH_MINUS_SIGN))]
#[case(crate::common::fixture_tuple!(NEGATIVE_FLOAT_WITH_EXPONENT))]
#[case(crate::common::fixture_tuple!(ARRAY_EMPTY))]
#[case(crate::common::fixture_tuple!(ARRAY_SINGLE))]
#[case(crate::common::fixture_tuple!(ARRAY_MANY))]
#[case(crate::common::fixture_tuple!(ARRAY_SUBARRAYS))]
#[case(crate::common::fixture_tuple!(ARRAY_OBJECTS_WITH_INCREASING_KEYS))]
#[case(crate::common::fixture_tuple!(ARRAY_MULTIPLE_EMPTY_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAY_MANY_SINGLE_KEY_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAY_MANY_TWO_KEY_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAY_MANY_FIVE_KEY_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAYS_NESTED_FIVE_LEVELS_WITH_OBJECT))]
#[case(crate::common::fixture_tuple!(STANDALONE_NULL))]
#[case(crate::common::fixture_tuple!(STANDALONE_FALSE))]
#[case(crate::common::fixture_tuple!(STANDALONE_TRUE))]
#[case(crate::common::fixture_tuple!(STANDALONE_STRING))]
#[case(crate::common::fixture_tuple!(NESTED_OBJECT_SINGLE_KEY))]
#[case(crate::common::fixture_tuple!(OBJECT_WITH_LONG_KEYS))]
#[case(crate::common::fixture_tuple!(ARRAY_WITH_NESTED_OBJECTS))]
#[case(crate::common::fixture_tuple!(MIXED_ARRAY_WITH_LONG_STRINGS))]
#[case(crate::common::fixture_tuple!(STANDALONE_STRING_WS))]
#[case(crate::common::fixture_tuple!(DEEPLY_NESTED))]
#[case(crate::common::fixture_tuple!(OBJECT_WITH_LONG_KEY_AND_ARR_VAL))]
#[case(crate::common::fixture_tuple!(OBJECT_WITH_EXPANDED_AND_NON_EXPANDED_ARR))]
#[case(crate::common::fixture_tuple!(DEEPLY_NESTED_OBJECT_WITH_ARR_VALUES))]
#[case(crate::common::fixture_tuple!(PRETTIER_KEY_VALUE))]
#[case(crate::common::fixture_tuple!(PRETTIER_MULTI_LINE))]
#[case(crate::common::fixture_tuple!(PRETTIER_SINGLE_LINE))]
#[case(crate::common::fixture_tuple!(PRETTIER_PASS1))]
#[case(crate::common::fixture_tuple!(PRETTIER_ARRAY))]
#[case(crate::common::fixture_tuple!(STRING_MIXED_CASE_HEX_UNICODE))]
#[case(crate::common::fixture_tuple!(EXPONENT_ZERO_PADDED))]
#[case(crate::common::fixture_tuple!(EXPONENT_ZERO_PADDED_PLUS))]
#[case(crate::common::fixture_tuple!(EXPONENT_ZERO_PADDED_MINUS))]
#[case(crate::common::fixture_tuple!(ARRAY_BLANK_LINES_AND_EXTRA_WHITESPACE))]
#[case(crate::common::fixture_tuple!(OBJECT_KEY_COMPLEX_ESCAPES))]
#[case(crate::common::fixture_tuple!(TSCONFIG))]
#[case(crate::common::fixture_tuple!(VALID_TRAILING_ZEROS_FRACTION))]
#[case(crate::common::fixture_tuple!(STRING_ESCAPED_SOLIDUS))]
#[case(crate::common::fixture_tuple!(STRING_UNICODE_ESCAPE_PRINTABLE))]
#[case(crate::common::fixture_tuple!(STRING_UNICODE_ESCAPE_CONTROL))]
#[case(crate::common::fixture_tuple!(STRING_SURROGATE_PAIR))]
#[case(crate::common::fixture_tuple!(STRING_UNESCAPED_UNICODE))]
#[case(crate::common::fixture_tuple!(OBJECT_KEY_UNICODE_ESCAPE))]
#[case(crate::common::fixture_tuple!(NEGATIVE_ZERO))]
#[case(crate::common::fixture_tuple!(FLOAT_WITH_TRAILING_ZERO))]
#[case(crate::common::fixture_tuple!(ARRAY_THREE_NEWLINES_BETWEEN_ITEMS))]
#[case(crate::common::fixture_tuple!(OBJECT_BLANK_LINES_BETWEEN_KEYS))]
#[case(crate::common::fixture_tuple!(OBJECT_ONLY_NEWLINE))]
#[case(crate::common::fixture_tuple!(OBJECT_LEADING_NEWLINE))]
#[case(crate::common::fixture_tuple!(OBJECT_WITH_80_CHAR_STRING_ARRAY_VAL))]
#[case(crate::common::fixture_tuple!(OBJECT_EMPTY_STRING_KEY))]
#[case(crate::common::fixture_tuple!(OBJECT_DUPLICATE_KEYS))]
#[case(crate::common::fixture_tuple!(ARRAY_SINGLE_OBJECT))]
#[case(crate::common::fixture_tuple!(OBJECT_INLINE_ROOT_80))]
#[case(crate::common::fixture_tuple!(OBJECT_INLINE_ROOT_83))]
#[case(crate::common::fixture_tuple!(OBJECT_INLINE_SHORT_ARRAY_VALUE))]
#[case(crate::common::fixture_tuple!(OBJECT_INLINE_NESTED_OBJECT_VALUE))]
#[case(crate::common::fixture_tuple!(OBJECT_INLINE_UNICODE_VALUE))]
#[case(crate::common::fixture_tuple!(ARRAY_INLINE_THREE_SHORT_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAY_EXPANDED_SIX_SHORT_OBJECTS))]
#[case(crate::common::fixture_tuple!(ARRAY_STRING_LONGER_THAN_PRINT_WIDTH))]
#[case(crate::common::fixture_tuple!(ARRAY_OVER_80_CHARS))]
#[case(crate::common::fixture_tuple!(ZERO))]
#[case(crate::common::fixture_tuple!(ZERO_FLOAT))]
#[case(crate::common::fixture_tuple!(EXPONENT_ZERO))]
#[case(crate::common::fixture_tuple!(EXPONENT_UPPERCASE_E))]
#[case(crate::common::fixture_tuple!(EXPONENT_UPPERCASE_E_PLUS))]
#[case(crate::common::fixture_tuple!(EXPONENT_LEADING_ZEROS))]
#[case(crate::common::fixture_tuple!(STRING_BACKSPACE_ESCAPE))]
#[case(crate::common::fixture_tuple!(STRING_FORM_FEED_ESCAPE))]
#[case(crate::common::fixture_tuple!(STRING_UNICODE_SPACE_ESCAPE))]
#[case(crate::common::fixture_tuple!(STRING_ALL_BASIC_ESCAPES))]
#[case(crate::common::fixture_tuple!(OBJECT_NUMERIC_STRING_KEY))]
#[case(crate::common::fixture_tuple!(OBJECT_SINGLE_KEY_ROOT))]
#[case(crate::common::fixture_tuple!(ARRAY_OBJECT_ELEMENT_INLINE_77))]
#[case(crate::common::fixture_tuple!(ARRAY_OBJECT_ELEMENT_INLINE_80))]
#[case(crate::common::fixture_tuple!(ARRAY_OBJECT_ELEMENT_INLINE_83))]
#[case(crate::common::fixture_tuple!(ARRAY_OBJECT_CRAB_EMOJI_INLINE))]
#[case(crate::common::fixture_tuple!(ARRAY_NUMBERS_FILL_MIXED_LENGTHS))]
#[case(crate::common::fixture_tuple!(ARRAY_NUMBERS_WITH_EXPONENTS_FILL))]
#[case(crate::common::fixture_tuple!(ARRAY_STRINGS_OVER_80))]
#[case(crate::common::fixture_tuple!(ARRAY_BOOLS_NULLS_OVER_80))]
#[case(crate::common::fixture_tuple!(ARRAY_MIXED_PRIMITIVES_OVER_80))]
#[case(crate::common::fixture_tuple!(ARRAY_NUMBERS_WITH_ONE_STRING_OVER_80))]
#[case(crate::common::fixture_tuple!(OBJECT_WITH_NESTED_FILL_ARRAY))]
#[case(crate::common::fixture_tuple!(ARRAY_NUMERIC_MATRIX_SHORT))]
#[case(crate::common::fixture_tuple!(ARRAY_NUMERIC_MATRIX_HETEROGENEOUS))]
pub fn format_template(#[case] (name, input): (&str, &str)) {}
