use clap::{Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, error::ErrorKind};
use jjpwrgem_parse::{
    format::{
        ConfigError, FormatConfig, FormatOption, FormatRequest, FormatSettings, InputSettings,
        Parser as ParseParser, rule_clause,
    },
    settings::{Setting, Shorthand},
};

use crate::{
    docs::{indent, strip_front_matter},
    get_docs_snapshot,
    setting_args::{FromSettings, Help},
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
    Check(CheckArgs),
    /// Start a language server over stdio
    Lsp,
}

#[derive(Args, Debug)]
pub struct FormatArgs {
    #[command(flatten)]
    settings: FromSettings<FormatSettings, FormatHelp>,

    #[command(flatten)]
    input: FromSettings<InputSettings, FormatHelp>,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    input: FromSettings<InputSettings>,
}

impl From<FormatArgs> for FormatConfig {
    fn from(args: FormatArgs) -> Self {
        Self {
            input: args.input.into_inner(),
            settings: args.settings.into_inner(),
        }
    }
}

#[derive(Debug, Clone)]
struct FormatHelp;

impl Help for FormatHelp {
    fn help(setting: &Setting) -> String {
        with_rules(setting.description, &FormatOption::for_setting(setting))
    }

    fn shorthand_help(setting: &Setting, shorthand: &Shorthand) -> String {
        let help = format!("Shorthand for --{} {}", setting.flag(), shorthand.value);
        with_rules(&help, &FormatOption::for_setting(setting))
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
        Commands::Check(args) => Action::Check(args.input.into_inner().parser),
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

fn option_flag(option: FormatOption) -> String {
    match option.value() {
        Some(value) => format!("--{} {value}", option.setting().flag()),
        None => format!("--{}", option.setting().flag()),
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
