#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI should write to stdout/err"
)]
mod commands;
mod error;
mod output;

use std::{
    io::{IsTerminal, Read},
    process::ExitCode,
};

use clap::Parser;
pub use error::{Error, Result};
use jjpwrgem_parse::{
    error::diagnostics::Diagnostic,
    format::{self, LineEnding},
    validate_str,
};
use jjpwrgem_ui::{Color, Style};

use crate::{commands::Commands, output::Output};

fn main() -> ExitCode {
    let cli = commands::Cli::parse();

    let style = Style::Pretty(Color::Plain);

    let buf = {
        let mut stdin = std::io::stdin();

        if stdin.is_terminal() {
            anstream::eprintln!(
                "{}",
                style.render_message(Error::NonEmptyStdinRequired.into())
            );
            return ExitCode::FAILURE;
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
            return ExitCode::FAILURE;
        }

        buf
    };
    let json = match String::from_utf8(buf) {
        Err(e) => {
            let err = jjpwrgem_parse::Error::from_utf8_error_slice(e.utf8_error(), e.as_bytes());
            anstream::eprintln!("{}", style.render_diagnostic(Diagnostic::from(&err)));
            return ExitCode::FAILURE;
        }
        Ok(s) => s,
    };
    let output = match &cli.command {
        Commands::Format {
            uglify,
            preferred_width,
            end_of_line,
        } => format(
            &json,
            style,
            *uglify,
            *preferred_width,
            end_of_line.into_parse(),
        ),
        Commands::Check => check(&json, style),
    };

    print_output(&output);

    output.exit_code
}

pub fn format(
    json: &str,
    style: Style,
    uglify: bool,
    preferred_width: usize,
    line_ending: LineEnding,
) -> Output {
    let result = if uglify {
        format::uglify_str(json)
    } else {
        format::prettify_str(json, preferred_width, line_ending)
    };

    match result {
        Ok(pretty) => Output::success(pretty),
        Err(error) => Output::failure_diagnostic(Diagnostic::from(&error), style),
    }
}

pub fn check(json: &str, style: Style) -> Output {
    match validate_str(json) {
        Ok(()) => Output::success(""),
        Err(error) => Output::failure_diagnostic(Diagnostic::from(&error), style),
    }
}

fn print_output(output: &Output) {
    if let Some(stdout) = &output.stdout {
        println!("{stdout}");
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
