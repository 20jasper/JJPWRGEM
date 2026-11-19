#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI should write to stdout/err"
)]
#![allow(
    variant_size_differences,
    reason = "commands are different sizes and not perf sensitive"
)]
mod commands;
mod error;
mod output;
mod setting_args;

use std::{
    io::{IsTerminal, Read, Write},
    process::ExitCode,
};

pub use error::{Error, Result};
use jjpwrgem_parse::{
    diagnostics::Diagnostic,
    format::{FormatRequest, Parser},
    format_str, validate_str,
};
use jjpwrgem_ui::{Color, Style};

use crate::{commands::Action, output::Output};

fn main() -> ExitCode {
    let style = Style::Pretty(Color::Plain);

    match commands::parse() {
        Action::Lsp => match lsp::run_blocking() {
            Ok(code) => code,
            Err(error) => {
                anstream::eprintln!("jjp lsp: {error}");
                ExitCode::FAILURE
            }
        },
        Action::Check(Parser::Jsonlines) => run_on_stdin(style, check_jsonlines),
        Action::Check(Parser::Json) => run_on_stdin(style, check),
        Action::Format(request) => {
            run_on_stdin(style, |json, style| format_input(json, style, &request))
        }
    }
}

fn run_on_stdin(style: Style, run: impl FnOnce(&str, Style) -> Output) -> ExitCode {
    let Some(input) = read_stdin(style) else {
        return ExitCode::FAILURE;
    };
    let Some(json) = decode_utf8(&input, style) else {
        return ExitCode::FAILURE;
    };

    let mut output = run(json, style);

    print_output(&mut output);
    output.exit_code
}

fn read_stdin(style: Style) -> Option<Vec<u8>> {
    let buf = {
        let mut stdin = std::io::stdin();

        if stdin.is_terminal() {
            anstream::eprintln!(
                "{}",
                style.render_message(Error::NonEmptyStdinRequired.into())
            );
            return None;
        }

        let mut buf = vec![];
        stdin
            .read_to_end(&mut buf)
            .expect("Failed to read from stdin");

        if buf.is_empty() {
            anstream::eprintln!(
                "{}",
                style.render_message(Error::NonEmptyStdinRequired.into())
            );
            return None;
        }

        buf
    };
    Some(buf)
}

fn decode_utf8(bytes: &[u8], style: Style) -> Option<&str> {
    // simd validation is faster, std finds where it went wrong
    if let Ok(json) = simdutf8::basic::from_utf8(bytes) {
        return Some(json);
    }
    let e = std::str::from_utf8(bytes).expect_err("std accepted input simdutf8 rejected");
    let err = jjpwrgem_parse::Error::from_utf8_error_slice(e, bytes);
    anstream::eprintln!("{}", style.render_diagnostic(Diagnostic::from(&err)));
    None
}

fn format_input(json: &str, style: Style, request: &FormatRequest) -> Output {
    match format_str(json, request) {
        Ok(result) => Output::success(result),
        Err(error) => Output::failure_diagnostic(Diagnostic::from(&error), style),
    }
}

pub fn check(json: &str, style: Style) -> Output {
    match validate_str(json) {
        Ok(()) => Output::success(""),
        Err(error) => Output::failure_diagnostic(Diagnostic::from(&error), style),
    }
}

fn check_jsonlines(json: &str, style: Style) -> Output {
    match jjpwrgem_parse::jsonlines::parse(json) {
        Ok(_) => Output::success(""),
        Err(e) => Output::failure_diagnostic(Diagnostic::from(&e), style),
    }
}

fn print_output(output: &mut Output) {
    if let Some(stdout) = &mut output.stdout {
        // one write ending in a newline, so stdout's line buffering finds it at the last byte
        // instead of scanning the whole output
        stdout.push('\n');
        std::io::stdout()
            .lock()
            .write_all(stdout.as_bytes())
            .expect("failed printing to stdout");
    }
    if let Some(stderr) = &output.stderr {
        anstream::eprintln!("{stderr}");
    }
}
mod docs {
    macro_rules! get_docs_snapshot {
        ($name:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/integration/snapshots/",
                $name,
                ".snap"
            ))
        };
    }

    pub(crate) use get_docs_snapshot;

    pub fn strip_front_matter(raw: &str) -> &str {
        const FRONT_MATTER_SEP: &str = "\n---\n";
        raw.split_once(FRONT_MATTER_SEP)
            .expect("snapshots should always have a separator")
            .1
    }
    pub fn indent(s: &str) -> String {
        s.lines()
            .map(|x| format!("\t{x}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
