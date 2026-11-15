use std::fmt::Display;

use clap::{
    Arg, ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum,
    builder::PossibleValuesParser, error::ErrorKind, parser::ValueSource, value_parser,
};
use jjpwrgem_parse::format::{
    ConfigError, FormatConfig, FormatOption, FormatRequest, Parser as ParseParser, Setting,
    SettingKind, rule_clause,
};
use serde_json::Value;

use crate::{
    docs::{indent, strip_front_matter},
    format_settings::FORMAT_SETTINGS,
    get_docs_snapshot,
};

const FLAG_ALIASES: &[(&str, &str)] = &[("end-of-line", "eol")];

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
    settings: FormatSettings,

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
            ..self.settings.0.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormatSettings(FormatConfig);

fn invalid(message: impl Display) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, message)
}

fn setting_option(setting: &Setting) -> Option<FormatOption> {
    FormatOption::ALL
        .into_iter()
        .find(|option| option.setting() == Some(setting.name))
}

fn setting_arg(setting: &Setting) -> Arg {
    let help = setting_option(setting).map_or_else(
        || setting.description.to_owned(),
        |option| with_rules(setting.description, option),
    );
    let arg = Arg::new(setting.name)
        .long(setting.flag)
        .help(help)
        .value_name(setting.flag.to_uppercase().replace('-', "_"));
    let arg = match setting.kind {
        SettingKind::Integer { default } => arg
            .default_value(default.to_string())
            .value_parser(value_parser!(usize)),
        SettingKind::Choice { values, default } => arg
            .default_value(default)
            .value_parser(PossibleValuesParser::new(values)),
    };
    FLAG_ALIASES
        .iter()
        .filter(|(flag, _)| *flag == setting.flag)
        .fold(arg, |arg, (_, alias)| arg.visible_alias(*alias))
}

impl Args for FormatSettings {
    fn augment_args(cmd: Command) -> Command {
        cmd.args(FORMAT_SETTINGS.iter().map(setting_arg))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for FormatSettings {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut config = serde_json::Map::new();
        for setting in FORMAT_SETTINGS {
            if matches.value_source(setting.name) != Some(ValueSource::CommandLine) {
                continue;
            }
            let value = match setting.kind {
                SettingKind::Integer { .. } => matches
                    .try_get_one::<usize>(setting.name)
                    .map_err(invalid)?
                    .map(|value| Value::from(*value)),
                SettingKind::Choice { .. } => matches
                    .try_get_one::<String>(setting.name)
                    .map_err(invalid)?
                    .map(|value| Value::from(value.as_str())),
            };
            if let Some(value) = value {
                config.insert(setting.name.to_owned(), value);
            }
        }
        serde_json::from_value(Value::Object(config))
            .map(Self)
            .map_err(invalid)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
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

fn flag_spelling(option: FormatOption) -> String {
    FORMAT_SETTINGS
        .iter()
        .find(|setting| option.setting() == Some(setting.name))
        .map_or_else(
            || match option {
                FormatOption::Uglify => "--uglify".to_owned(),
                FormatOption::Jsonlines => "--parser jsonlines".to_owned(),
                other @ FormatOption::PreferredWidth => other.name().to_owned(),
            },
            |setting| format!("--{}", setting.flag),
        )
}

fn with_rules(help: &str, option: FormatOption) -> String {
    rule_clause(option, flag_spelling).map_or_else(
        || help.to_owned(),
        |clause| format!("{help}. {} {clause}", flag_spelling(option)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_options_with_settings_are_in_the_table() {
        for option in FormatOption::ALL {
            if let Some(name) = option.setting() {
                assert!(FORMAT_SETTINGS.iter().any(|setting| setting.name == name));
            }
        }
    }
}
