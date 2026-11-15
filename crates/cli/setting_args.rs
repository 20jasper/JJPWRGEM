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

    fn customize(_setting: &Setting, arg: Arg) -> Arg {
        arg
    }
}

#[derive(Clone, Debug)]
pub struct FromSettings<T>(pub T);

fn flag_name(name: &str) -> String {
    name.replace('_', "-")
}

fn camel_case(name: &str) -> String {
    name.split('_')
        .enumerate()
        .map(|(index, word)| match (index, word.split_at_checked(1)) {
            (1.., Some((head, tail))) => head.to_uppercase() + tail,
            _ => word.to_owned(),
        })
        .collect()
}

pub fn flag_spelling(name: &str) -> String {
    format!("--{}", flag_name(name))
}

fn invalid(message: impl Display) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, message)
}

fn setting_arg(setting: &Setting) -> Arg {
    let flag = flag_name(setting.name);
    let arg = Arg::new(setting.name)
        .long(flag.clone())
        .help(setting.description)
        .value_name(flag.to_uppercase().replace('-', "_"));
    match setting.kind {
        SettingKind::Integer { default } => arg
            .default_value(default.to_string())
            .value_parser(value_parser!(usize)),
        SettingKind::Choice { values, default } => arg
            .default_value(default)
            .value_parser(PossibleValuesParser::new(values)),
    }
}

fn given(matches: &ArgMatches, setting: &Setting) -> Result<Option<Value>, clap::Error> {
    if matches.value_source(setting.name) != Some(ValueSource::CommandLine) {
        return Ok(None);
    }
    match setting.kind {
        SettingKind::Integer { .. } => matches
            .try_get_one::<usize>(setting.name)
            .map_err(invalid)
            .map(|value| value.map(|value| Value::from(*value))),
        SettingKind::Choice { .. } => matches
            .try_get_one::<String>(setting.name)
            .map_err(invalid)
            .map(|value| value.map(|value| Value::from(value.as_str()))),
    }
}

fn given_values(
    matches: &ArgMatches,
    table: &[Setting],
) -> Result<Map<String, Value>, clap::Error> {
    table
        .iter()
        .filter_map(|setting| {
            given(matches, setting)
                .map(|value| value.map(|value| (camel_case(setting.name), value)))
                .transpose()
        })
        .collect()
}

impl<T: Settings> Args for FromSettings<T> {
    fn augment_args(cmd: Command) -> Command {
        cmd.args(
            T::TABLE
                .iter()
                .map(|setting| T::customize(setting, setting_arg(setting))),
        )
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl<T: Settings> FromArgMatches for FromSettings<T> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        serde_json::from_value(Value::Object(given_values(matches, T::TABLE)?))
            .map(Self)
            .map_err(invalid)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}
