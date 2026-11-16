use std::{fmt::Display, marker::PhantomData};

use clap::{
    Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches, builder::PossibleValuesParser,
    error::ErrorKind, parser::ValueSource, value_parser,
};
use jjpwrgem_parse::settings::{Setting, SettingKind, Shorthand};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub trait Settings: DeserializeOwned {
    const TABLE: &'static [Setting];
}

pub trait Help {
    fn help(setting: &Setting) -> String;

    fn shorthand_help(setting: &Setting, shorthand: &Shorthand) -> String {
        format!("Shorthand for --{} {}", setting.flag(), shorthand.value)
    }
}

#[derive(Debug, Clone)]
pub struct Plain;

impl Help for Plain {
    fn help(setting: &Setting) -> String {
        setting.description.to_owned()
    }
}

#[derive(Clone, Debug)]
pub struct FromSettings<T, H = Plain>(pub T, PhantomData<H>);

fn invalid_value(message: impl Display) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, message)
}

fn setting_arg(setting: &Setting, help: String) -> Arg {
    let arg = Arg::new(setting.name)
        .long(setting.flag())
        .help(help)
        .visible_aliases(setting.aliases.iter().copied());
    match setting.kind {
        SettingKind::Flag => arg.action(ArgAction::SetTrue),
        SettingKind::Integer { default } => arg
            .value_name(setting.name.to_uppercase())
            .default_value(default.to_string())
            .value_parser(value_parser!(usize)),
        SettingKind::Choice {
            values, default, ..
        } => arg
            .value_name(setting.name.to_uppercase())
            .default_value(default)
            .value_parser(PossibleValuesParser::new(values)),
    }
}

fn shorthand_arg(setting: &Setting, shorthand: &Shorthand, help: String) -> Arg {
    let arg = Arg::new(shorthand.name)
        .long(shorthand.name)
        .help(help)
        .action(ArgAction::SetTrue)
        .conflicts_with(setting.name);
    shorthand.short.into_iter().fold(arg, Arg::short)
}

fn shorthand_value(matches: &ArgMatches, setting: &Setting) -> Option<Value> {
    setting
        .shorthand
        .iter()
        .find(|shorthand| matches.get_flag(shorthand.name))
        .map(|shorthand| Value::from(shorthand.value))
}

fn command_line_value(matches: &ArgMatches, setting: &Setting) -> Option<Value> {
    shorthand_value(matches, setting).or_else(|| main_value(matches, setting))
}

fn main_value(matches: &ArgMatches, setting: &Setting) -> Option<Value> {
    if matches.value_source(setting.name) != Some(ValueSource::CommandLine) {
        return None;
    }
    match setting.kind {
        SettingKind::Flag => matches.get_flag(setting.name).then_some(Value::Bool(true)),
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

impl<T: Settings, H: Help> Args for FromSettings<T, H> {
    fn augment_args(cmd: Command) -> Command {
        cmd.args(T::TABLE.iter().flat_map(|setting| {
            std::iter::once(setting_arg(setting, H::help(setting))).chain(
                setting.shorthand.iter().map(|shorthand| {
                    shorthand_arg(setting, shorthand, H::shorthand_help(setting, shorthand))
                }),
            )
        }))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl<T: Settings, H: Help> FromArgMatches for FromSettings<T, H> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        serde_json::from_value(Value::Object(command_line_values(matches, T::TABLE)))
            .map(|settings| Self(settings, PhantomData))
            .map_err(invalid_value)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}
