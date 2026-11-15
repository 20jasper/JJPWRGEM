use clap::{
    Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum, error::ErrorKind,
};
use jjpwrgem_parse::format::{
    ConfigError, DEFAULT_PREFERRED_WIDTH, FormatConfig, FormatOption, FormatRequest, LineEnding,
    Parser as ParseParser, rule_clause,
};

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
    Format(FormatArgs),
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

#[derive(Args)]
pub struct FormatArgs {
    #[arg(short, long, help = with_rules(
        "Removes all insignificant whitespace instead of pretty printing, also known as minifying",
        FormatOption::Uglify,
    ))]
    uglify: bool,

    #[arg(long, help = format!(
        "{} [default: {DEFAULT_PREFERRED_WIDTH}]",
        with_rules(
            "Preferred maximum line width. Note this is not a hard maximum width",
            FormatOption::PreferredWidth,
        ),
    ))]
    preferred_width: Option<usize>,

    /// Line ending to use when formatting output
    #[arg(value_enum, long, visible_alias = "eol", default_value_t)]
    end_of_line: LineEndingArg,

    #[arg(value_enum, long, default_value_t, help = with_rules(
        "Parser to use for input",
        FormatOption::Jsonlines,
    ))]
    parser: ParserArg,
}

impl FormatArgs {
    fn config(&self) -> FormatConfig {
        FormatConfig {
            parser: self.parser.into(),
            uglify: self.uglify,
            preferred_width: self.preferred_width,
            line_ending: self.end_of_line.into(),
        }
    }
}

pub enum Action {
    Lsp,
    Check(ParserArg),
    Format(FormatRequest),
}

pub fn parse() -> Action {
    let mut command = Cli::command();
    let mut matches = command.get_matches_mut();
    let name = matches.subcommand_name().map(str::to_owned);
    let cli = Cli::from_arg_matches_mut(&mut matches).unwrap_or_else(|error| error.exit());
    match cli.command {
        Commands::Lsp => Action::Lsp,
        Commands::Check { parser } => Action::Check(parser),
        Commands::Format(args) => match args.config().validate() {
            Ok(request) => Action::Format(request),
            Err(ConfigError(rule)) => {
                usage_error(&command, name.as_deref(), rule.describe(flag_spelling)).exit()
            }
        },
    }
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

impl From<LineEndingArg> for LineEnding {
    fn from(arg: LineEndingArg) -> Self {
        match arg {
            LineEndingArg::Lf => Self::Lf,
            LineEndingArg::CrLf => Self::CrLf,
        }
    }
}

impl From<ParserArg> for ParseParser {
    fn from(arg: ParserArg) -> Self {
        match arg {
            ParserArg::Json => Self::Json,
            ParserArg::Jsonlines => Self::Jsonlines,
        }
    }
}

fn flag_spelling(option: FormatOption) -> String {
    match option {
        FormatOption::Uglify => "--uglify",
        FormatOption::Jsonlines => "--parser jsonlines",
        FormatOption::PreferredWidth => "--preferred-width",
    }
    .to_owned()
}

fn sentence(option: FormatOption, clause: &str) -> String {
    if let Some((_, value)) = flag_spelling(option).split_once(' ') {
        return format!("{value} {clause}");
    }
    let mut chars = clause.chars();
    chars
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .chain(chars)
        .collect()
}

fn with_rules(help: &str, option: FormatOption) -> String {
    rule_clause(option, flag_spelling).map_or_else(
        || help.to_owned(),
        |clause| format!("{help}. {}", sentence(option, &clause)),
    )
}

fn usage_error(
    command: &Command,
    subcommand: Option<&str>,
    message: impl std::fmt::Display,
) -> clap::Error {
    subcommand
        .and_then(|name| command.find_subcommand(name))
        .unwrap_or(command)
        .clone()
        .error(ErrorKind::ArgumentConflict, message)
}
