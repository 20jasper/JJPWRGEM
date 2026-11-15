use clap::{
    Arg, Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum,
    error::ErrorKind,
};
use jjpwrgem_parse::{
    format::{
        ConfigError, FORMAT_SETTINGS, FormatConfig, FormatOption, FormatRequest, FormatSettings,
        Parser as ParseParser, rule_clause,
    },
    settings::Setting,
};

use crate::{
    docs::{indent, strip_front_matter},
    get_docs_snapshot,
    setting_args::{FromSettings, Settings},
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

    #[command(flatten)]
    settings: FromSettings<FormatSettings>,

    #[arg(value_enum, long, default_value_t, help = with_rules(
        "Parser to use for input",
        FormatOption::Jsonlines,
    ))]
    parser: ParserArg,
}

impl FormatArgs {
    fn config(self) -> FormatConfig {
        FormatConfig {
            parser: self.parser.into(),
            uglify: self.uglify,
            settings: self.settings.0,
        }
    }
}

impl Settings for FormatSettings {
    const TABLE: &'static [Setting] = FORMAT_SETTINGS;
    const ALIASES: &'static [(&'static str, &'static str)] = &[("end_of_line", "eol")];

    fn customize(setting: &Setting, arg: Arg) -> Arg {
        match FormatOption::for_setting(setting.name) {
            Some(option) => arg.help(with_rules(setting.description, option)),
            None => arg,
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
                usage_error(&command, name.as_deref(), rule.describe(option_flag)).exit()
            }
        },
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ParserArg {
    #[default]
    #[value(name = "json")]
    Json,
    #[value(name = "jsonlines")]
    Jsonlines,
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
    match option {
        FormatOption::Uglify => "--uglify".to_owned(),
        FormatOption::Jsonlines => "--parser jsonlines".to_owned(),
        FormatOption::PreferredWidth => option
            .setting()
            .map_or_default(|setting| format!("--{}", setting.flag())),
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
