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

use clap::Parser;
pub use error::{Error, Result};
use jjpwrgem_parse::{
    diagnostics::Diagnostic,
    format::{FormatRequest, JsonMode, JsonlinesOptions, PrettifyOptions},
    format_str, validate_str,
};
use jjpwrgem_ui::{Color, Style};

use crate::{
    commands::{Commands, ParserArg},
    output::Output,
};

fn main() -> ExitCode {
    let cli = commands::Cli::parse();

    let style = Style::Pretty(Color::Plain);

    if matches!(cli.command, Commands::Lsp) {
        lsp::run_blocking();
        return ExitCode::SUCCESS;
    }

    let Some(input) = read_stdin(style) else {
        return ExitCode::FAILURE;
    };
    let Some(json) = decode_utf8(&input, style) else {
        return ExitCode::FAILURE;
    };

    let mut output = match &cli.command {
        Commands::Format {
            uglify,
            preferred_width,
            end_of_line,
            parser,
        } => match parser {
            ParserArg::Jsonlines => {
                if *uglify {
                    Output::failure(
                        "error: the argument '--parser jsonlines' cannot be used with '--uglify'\n\nUsage: jjp format --parser jsonlines\n\nFor more information, try '--help'.",
                    )
                } else if preferred_width.is_some() {
                    Output::failure(
                        "error: the argument '--parser jsonlines' cannot be used with '--preferred-width'\n\nUsage: jjp format --parser jsonlines\n\nFor more information, try '--help'.",
                    )
                } else {
                    let request = FormatRequest::Jsonlines(
                        JsonlinesOptions::builder()
                            .line_ending(end_of_line.into_parse())
                            .build(),
                    );
                    format_input(json, style, &request)
                }
            }
            ParserArg::Json => {
                let mode = if *uglify {
                    JsonMode::Uglify
                } else {
                    JsonMode::Prettify(
                        PrettifyOptions::builder()
                            .line_ending(end_of_line.into_parse())
                            .maybe_preferred_width(*preferred_width)
                            .build(),
                    )
                };
                format_input(json, style, &FormatRequest::Json(mode))
            }
        },
        Commands::Check { parser } => match parser {
            ParserArg::Jsonlines => check_jsonlines(json, style),
            ParserArg::Json => check(json, style),
        },
        Commands::Lsp => unreachable!("LSP command handled above"),
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
