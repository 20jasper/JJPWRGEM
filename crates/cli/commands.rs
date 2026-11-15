use clap::{Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, error::ErrorKind};
use jjpwrgem_parse::{
    format::{
        ConfigError, FORMAT_SETTINGS, FormatConfig, FormatOption, FormatRequest, FormatSetting,
        FormatSettings, INPUT_SETTINGS, InputSetting, InputSettings, Parser as ParseParser,
        rule_clause,
    },
    settings::Setting,
};

use crate::{
    docs::{indent, strip_front_matter},
    get_docs_snapshot,
    setting_args::{FromSettings, Help, Settings},
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
            input: args.input.0,
            settings: args.settings.0,
        }
    }
}

impl Settings for InputSettings {
    const TABLE: &'static [Setting] = INPUT_SETTINGS;
}

impl Settings for FormatSettings {
    const TABLE: &'static [Setting] = FORMAT_SETTINGS;
}

#[derive(Debug, Clone)]
struct FormatHelp;

impl Help for FormatHelp {
    fn help(setting: &Setting) -> String {
        FormatSetting::ALL
            .iter()
            .copied()
            .find(|id| id.setting().name == setting.name)
            .map(FormatOption::Setting)
            .or_else(|| {
                (InputSetting::Parser.setting().name == setting.name)
                    .then_some(FormatOption::Jsonlines)
            })
            .map_or_else(
                || setting.description.to_owned(),
                |option| with_rules(setting.description, option),
            )
    }
}

pub enum Action {
    Lsp,
    Check(ParseParser),
    Format(FormatRequest),
}

pub fn parse() -> Action {
    let mut command = Cli::command();
    let mut matches = command.get_matches_mut();
    let name = matches.subcommand_name().map(str::to_owned);
    let cli = Cli::from_arg_matches_mut(&mut matches)
        .unwrap_or_else(|error| error.format(&mut command).exit());
    match cli.command {
        Commands::Lsp => Action::Lsp,
        Commands::Check(args) => Action::Check(args.input.0.parser),
        Commands::Format(args) => match FormatRequest::try_from(FormatConfig::from(args)) {
            Ok(request) => Action::Format(request),
            Err(ConfigError(rule)) => {
                usage_error(&command, name.as_deref(), rule.describe(option_flag)).exit()
            }
        },
    }
}

fn option_flag(option: FormatOption) -> String {
    match option {
        FormatOption::Jsonlines => "--parser jsonlines".to_owned(),
        FormatOption::Setting(id) => format!("--{}", id.setting().flag()),
    }
}

fn with_rules(help: &str, option: FormatOption) -> String {
    rule_clause(option, option_flag).map_or_else(
        || help.to_owned(),
        |clause| format!("{help}. {} {clause}", option_flag(option)),
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
