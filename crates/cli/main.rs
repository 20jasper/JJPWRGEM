use clap::Parser;
use jjpwrgem_parse::{
    ast::parse_str,
    error::diagnostics::{self, Diagnostic, Source},
    format,
};
use jjpwrgem_ui::{Color, Style};
use std::io::Read;
use std::process::ExitCode;

use crate::commands::Commands;
use crate::output::Output;

fn main() -> ExitCode {
    let cli = commands::Cli::parse();

    let style = Style::Pretty(Color::Plain);

    let mut buf = vec![];
    std::io::stdin()
        .read_to_end(&mut buf)
        .expect("Failed to read from stdin");

    let json = match String::from_utf8(buf) {
        Err(_) => {
            anstream::eprintln!(
                "{}",
                jjpwrgem_ui::render(diagnostics::invalid_encoding(Source::Stdin("")), style,)
            );
            return ExitCode::FAILURE;
        }
        Ok(s) => s,
    };
    let output = match &cli.command {
        Commands::Format { uglify } => format(json, style, *uglify),
        Commands::Check => check(json, style),
    };

    print_output(&output);

    output.exit_code
}

pub fn format(json: String, style: Style, uglify: bool) -> Output {
    let cmd = if !uglify {
        format::prettify_str
    } else {
        format::uglify_str
    };

    match cmd(&json) {
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

mod commands {
    use clap::{Parser, Subcommand};

    fn strip_front_matter(raw: &str) -> &str {
        const FRONT_MATTER_SEP: &str = "\n---\n";
        raw.split_once(FRONT_MATTER_SEP)
            .expect("snapshots should always have a separator")
            .1
    }
    fn indent(s: &str) -> String {
        s.lines()
            .map(|x| format!("\t{x}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
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

    use get_docs_snapshot;

    #[derive(Parser)]
    #[command(
        version = concat!(
            env!("CARGO_PKG_VERSION"), "\n", 
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/axolotl.txt")), "\n",
            "an axolotl riding a skateboard"
        ),
        about,
        disable_help_subcommand = true,
        help_expected = true,
        after_help = format!(
            "jjpwrgem is a tool for formatting and validating json inputs\n\nExamples:\n{}\n\n{}\n\nRun jjp <COMMAND> --help for information about specific commands",
            indent(strip_front_matter(get_docs_snapshot!("format_pretty"))),
            indent(strip_front_matter(get_docs_snapshot!("check_failure"))), 
        )
    )]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        /// Make your json look really good
        #[command(after_help = format!(
            "Examples:\n{}\n\n{}",
            indent(strip_front_matter(get_docs_snapshot!("format_pretty"))),
            indent(strip_front_matter(get_docs_snapshot!("format_uglify"))),
        ))]
        Format {
            /// Removes all insignificant whitespace instead of pretty printing, also known as minifying
            #[arg(short, long)]
            uglify: bool,
        },
        #[command(after_help = format!(
            "Examples:\n{}\n\n{}",
            indent(strip_front_matter(get_docs_snapshot!("check_success"))),
            indent(strip_front_matter(get_docs_snapshot!("check_failure"))),
        ))]
        /// Validates json syntax
        Check,
    }
}

mod output {
    use core::fmt::Debug;
    use jjpwrgem_parse::error::diagnostics::Diagnostic;
    use jjpwrgem_ui::Style;
    use std::process::ExitCode;

    pub struct Output {
        pub stdout: Option<String>,
        pub stderr: Option<String>,
        pub exit_code: ExitCode,
    }

    impl Output {
        pub fn success(stdout: impl Into<String>) -> Self {
            Output {
                stdout: Some(stdout.into()),
                stderr: None,
                exit_code: ExitCode::SUCCESS,
            }
        }

        pub fn failure_diagnostic(diagnostic: Diagnostic, style: Style) -> Self {
            Output {
                stdout: None,
                stderr: Some(jjpwrgem_ui::render(diagnostic, style)),
                exit_code: ExitCode::FAILURE,
            }
        }
    }

    impl Debug for Output {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let stdout = self.stdout.as_deref().unwrap_or("<empty>");
            let stderr = self.stderr.as_deref().unwrap_or("<empty>");
            write!(f, "stdout --- \n{stdout}\nstderr --- \n{stderr}",)
        }
    }
}
