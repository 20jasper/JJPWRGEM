use clap::{Parser, Subcommand, ValueEnum};
use jjpwrgem_parse::format::LineEnding;

use crate::{
    docs::{indent, strip_front_matter},
    get_docs_snapshot,
};

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
        /// Removes all insignificant whitespace instead of pretty printing,
        /// also known as minifying. Cannot be combined with --preferred-width or --parser jsonlines
        #[arg(short, long, conflicts_with = "preferred_width")]
        uglify: bool,

        /// Preferred maximum line width. Note this is not a hard maximum width [default: 80]
        #[arg(long, conflicts_with = "uglify")]
        preferred_width: Option<usize>,

        /// Line ending to use when formatting output
        #[arg(value_enum, long, visible_alias = "eol", default_value_t)]
        end_of_line: LineEndingArg,

        /// Parser to use for input. jsonlines cannot be combined with --uglify
        #[arg(value_enum, long, default_value_t)]
        parser: ParserArg,
    },
    #[command(after_help = format!(
        "Examples:\n{}\n\n{}",
        indent(strip_front_matter(get_docs_snapshot!("check_success"))),
        indent(strip_front_matter(get_docs_snapshot!("check_failure"))),
    ))]
    /// Validates json syntax
    Check {
        /// Parser to use for input
        #[arg(value_enum, long, default_value_t)]
        parser: ParserArg,
    },
    /// Start a language server over stdio
    Lsp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Default)]
pub enum LineEndingArg {
    #[default]
    #[value(name = "lf")]
    Lf,
    #[value(name = "crlf")]
    CrLf,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ParserArg {
    #[default]
    #[value(name = "json")]
    Json,
    #[value(name = "jsonlines")]
    Jsonlines,
}

impl LineEndingArg {
    pub const fn into_parse(self) -> LineEnding {
        match self {
            Self::Lf => LineEnding::Lf,
            Self::CrLf => LineEnding::CrLf,
        }
    }
}
