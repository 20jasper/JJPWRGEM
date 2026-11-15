use std::fmt::Display;

use clap::{
    Arg, ArgMatches, Args, Command, FromArgMatches, builder::PossibleValuesParser,
    error::ErrorKind, parser::ValueSource, value_parser,
};
use jjpwrgem_parse::settings::{Setting, SettingKind};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub trait Settings: DeserializeOwned {
    const TABLE: &'static [Setting];
    const ALIASES: &'static [(&'static str, &'static str)] = &[];

    fn customize(_setting: &Setting, arg: Arg) -> Arg {
        arg
    }
}

#[derive(Clone, Debug)]
pub struct FromSettings<T>(pub T);

fn invalid_value(message: impl Display) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, message)
}

fn setting_arg(setting: &Setting) -> Arg {
    let arg = Arg::new(setting.name)
        .long(setting.flag())
        .help(setting.description)
        .value_name(setting.name.to_uppercase());
    match setting.kind {
        SettingKind::Integer { default } => arg
            .default_value(default.to_string())
            .value_parser(value_parser!(usize)),
        SettingKind::Choice { values, default } => arg
            .default_value(default)
            .value_parser(PossibleValuesParser::new(values)),
    }
}

fn command_line_value(matches: &ArgMatches, setting: &Setting) -> Option<Value> {
    if matches.value_source(setting.name) != Some(ValueSource::CommandLine) {
        return None;
    }
    match setting.kind {
        SettingKind::Integer { .. } => matches
            .get_one::<usize>(setting.name)
            .map(|value| Value::from(*value)),
        SettingKind::Choice { .. } => matches
            .get_one::<String>(setting.name)
            .map(|value| Value::from(value.as_str())),
    }
}

fn command_line_values(matches: &ArgMatches, table: &[Setting]) -> Map<String, Value> {
    table
        .iter()
        .filter_map(|setting| Some((setting.key(), command_line_value(matches, setting)?)))
        .collect()
}

impl<T: Settings> Args for FromSettings<T> {
    fn augment_args(cmd: Command) -> Command {
        cmd.args(T::TABLE.iter().map(|setting| {
            let arg = T::ALIASES
                .iter()
                .filter(|(name, _)| *name == setting.name)
                .fold(setting_arg(setting), |arg, (_, alias)| {
                    arg.visible_alias(*alias)
                });
            T::customize(setting, arg)
        }))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl<T: Settings> FromArgMatches for FromSettings<T> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        serde_json::from_value(Value::Object(command_line_values(matches, T::TABLE)))
            .map(Self)
            .map_err(invalid_value)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}
