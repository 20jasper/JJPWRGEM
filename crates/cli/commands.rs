use clap::{
    Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum, error::ErrorKind,
};
use jjpwrgem_parse::format::{
    ConfigError, DEFAULT_PREFERRED_WIDTH, FormatConfig, FormatMode, FormatOption, FormatRequest,
    FormatSetting, FormatSettings, InputSettings, LineEnding, Parser as ParseParser, rule_clause,
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

#[derive(Args, Debug)]
pub struct FormatArgs {
    #[arg(value_enum, long, default_value_t, help = with_rules(
        "Pretty print, or remove all insignificant whitespace (uglify, also known as minify)",
        &FormatOption::for_setting(FormatSetting::Mode.name()),
    ))]
    mode: ModeArg,

    #[arg(short, long, conflicts_with = "mode", help = with_rules(
        "Shorthand for --mode uglify",
        &FormatOption::for_setting(FormatSetting::Mode.name()),
    ))]
    uglify: bool,

    #[arg(long, help = format!(
        "{} [default: {DEFAULT_PREFERRED_WIDTH}]",
        with_rules(
            "Preferred maximum line width. Note this is not a hard maximum width",
            &FormatOption::for_setting(FormatSetting::PreferredWidth.name()),
        ),
    ))]
    preferred_width: Option<usize>,

    /// Line ending to use when formatting output
    #[arg(value_enum, long, visible_alias = "eol", default_value_t)]
    end_of_line: LineEndingArg,

    #[arg(value_enum, long, default_value_t, help = with_rules(
        "Parser to use for input",
        &FormatOption::for_setting("parser"),
    ))]
    parser: ParserArg,
}

impl From<FormatArgs> for FormatConfig {
    fn from(args: FormatArgs) -> Self {
        let mode = if args.uglify {
            FormatMode::Uglify
        } else {
            args.mode.into()
        };
        Self {
            input: InputSettings {
                parser: args.parser.into(),
            },
            settings: FormatSettings {
                mode,
                preferred_width: args.preferred_width,
                end_of_line: args.end_of_line.into(),
            },
        }
    }
}

pub enum Action {
    Lsp,
    Check(ParseParser),
    Format(FormatRequest),
}

pub fn parse() -> Action {
    let mut command = Cli::command();
    let matches = command.get_matches_mut();
    let cli =
        Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.format(&mut command).exit());
    match cli.command {
        Commands::Lsp => Action::Lsp,
        Commands::Check { parser } => Action::Check(parser.into()),
        Commands::Format(args) => match FormatRequest::try_from(FormatConfig::from(args)) {
            Ok(request) => Action::Format(request),
            Err(ConfigError(rule)) => usage_error(
                &command,
                matches.subcommand_name(),
                rule.describe(option_flag),
            )
            .exit(),
        },
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ModeArg {
    #[default]
    #[value(name = "pretty")]
    Pretty,
    #[value(name = "uglify")]
    Uglify,
}

impl From<ModeArg> for FormatMode {
    fn from(arg: ModeArg) -> Self {
        match arg {
            ModeArg::Pretty => Self::Pretty,
            ModeArg::Uglify => Self::Uglify,
        }
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

fn option_flag(option: FormatOption) -> String {
    let flag = option.setting_name().replace('_', "-");
    match option.value() {
        Some(value) => format!("--{flag} {value}"),
        None => format!("--{flag}"),
    }
}

fn with_rules(help: &str, options: &[FormatOption]) -> String {
    match (options.first(), rule_clause(options, option_flag)) {
        (Some(&subject), Some(clause)) => format!("{help}. {} {clause}", option_flag(subject)),
        _ => help.to_owned(),
    }
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
