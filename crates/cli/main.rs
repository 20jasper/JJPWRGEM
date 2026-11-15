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

use std::{
    io::{IsTerminal, Read, Write},
    process::ExitCode,
};

use clap::{CommandFactory, FromArgMatches};
pub use error::{Error, Result};
use jjpwrgem_parse::{
    diagnostics::Diagnostic,
    format::{DEFAULT_PREFERRED_WIDTH, FormatConfig, FormatRequest},
    format_str, validate_str,
};
use jjpwrgem_ui::{Color, Style};

use crate::{
    commands::{Commands, ParserArg},
    output::Output,
};

fn main() -> ExitCode {
    let mut command = commands::Cli::command();
    let cli = commands::Cli::from_arg_matches_mut(&mut command.get_matches_mut())
        .unwrap_or_else(|error| error.exit());

    let style = Style::Pretty(Color::Plain);

    if matches!(cli.command, Commands::Lsp) {
        lsp::run_blocking();
        return ExitCode::SUCCESS;
    }

    let request = match &cli.command {
        Commands::Format {
            uglify,
            preferred_width,
            end_of_line,
            parser,
        } => {
            let config = FormatConfig {
                parser: (*parser).into(),
                uglify: *uglify,
                preferred_width: preferred_width.unwrap_or(DEFAULT_PREFERRED_WIDTH),
                line_ending: (*end_of_line).into(),
            };
            match config.validate() {
                Ok(request) => Some(request),
                Err(error) => commands::rule_error(&command, error).exit(),
            }
        }
        _ => None,
    };

    let Some(input) = read_stdin(style) else {
        return ExitCode::FAILURE;
    };
    let Some(json) = decode_utf8(&input, style) else {
        return ExitCode::FAILURE;
    };

    let mut output = match (&cli.command, request) {
        (Commands::Format { .. }, Some(request)) => format_input(json, style, &request),
        (Commands::Check { parser }, _) => match parser {
            ParserArg::Jsonlines => check_jsonlines(json, style),
            ParserArg::Json => check(json, style),
        },
        _ => unreachable!("LSP handled above and format always has a request"),
    };

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
    #[macro_export]
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
