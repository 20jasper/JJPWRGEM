mod commands;
mod error;
mod output;
pub use error::{Error, Result};

use clap::Parser;
use jjpwrgem_parse::{
    ast::parse_str,
    error::diagnostics::{self, Diagnostic, Source},
    format,
};
use jjpwrgem_ui::{Color, Style};
use std::io::{IsTerminal, Read};
use std::process::ExitCode;

use crate::commands::Commands;
use crate::output::Output;

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
        Err(_) => {
            anstream::eprintln!(
                "{}",
                style.render_diagnostic(diagnostics::invalid_encoding(Source::Stdin("")))
            );
            return ExitCode::FAILURE;
        }
        Ok(s) => s,
    };
    let output = match &cli.command {
        Commands::Format {
            uglify,
            preferred_width,
        } => format(json, style, *uglify, *preferred_width),
        Commands::Check => check(json, style),
    };

    print_output(&output);

    output.exit_code
}

pub fn format(json: String, style: Style, uglify: bool, preferred_width: usize) -> Output {
    let result = if uglify {
        format::uglify_str(&json)
    } else {
        format::prettify_str(&json, preferred_width)
    };

    match result {
        Ok(pretty) => Output::success(pretty),
        Err(error) => Output::failure_diagnostic(Diagnostic::from(&error), style),
    }
}

pub fn check(json: String, style: Style) -> Output {
    match parse_str(&json) {
        Ok(_) => Output::success(""),
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
                "/tests/integration/commands/docs/snapshots/",
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
