//! Settings metadata to more easily generate or describe settings for integrations like editor extensions. See the xtask for an example

use std::marker::PhantomData;

mod choice;
mod table;

pub(crate) use choice::choice_enum;
pub use choice::{Choice, choice};
pub use table::Table;
pub(crate) use table::settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    Integer {
        default: usize,
    },
    Choice {
        values: &'static [&'static str],
        descriptions: &'static [&'static str],
        default: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shorthand {
    pub name: &'static str,
    pub short: Option<char>,
    pub value: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Setting {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
    pub aliases: &'static [&'static str],
    pub shorthand: Option<Shorthand>,
}

pub trait IntegerSetting {}

impl IntegerSetting for usize {}

impl IntegerSetting for Option<usize> {}

#[derive(Debug)]
#[must_use]
pub struct Spec<T>(Setting, PhantomData<fn() -> T>);

const fn blank<T>(kind: SettingKind) -> Spec<T> {
    Spec(
        Setting {
            name: "",
            description: "",
            kind,
            aliases: &[],
            shorthand: None,
        },
        PhantomData,
    )
}

pub const fn integer<T: IntegerSetting>(default: usize) -> Spec<T> {
    blank(SettingKind::Integer { default })
}

impl<T> Spec<T> {
    pub const fn row(Self(setting, _): Self) -> Setting {
        setting
    }

    pub const fn aliases(self, aliases: &'static [&'static str]) -> Self {
        Self(Setting { aliases, ..self.0 }, PhantomData)
    }

    pub const fn shorthand(self, name: &'static str, short: char, value: &'static str) -> Self {
        Self(
            Setting {
                shorthand: Some(Shorthand {
                    name,
                    short: Some(short),
                    value,
                }),
                ..self.0
            },
            PhantomData,
        )
    }
}

impl Setting {
    pub fn key(&self) -> String {
        snake_to_camel(self.name)
    }

    pub fn flag(&self) -> String {
        snake_to_kebab(self.name)
    }
}

fn snake_to_kebab(name: &str) -> String {
    name.replace('_', "-")
}

fn snake_to_camel(name: &str) -> String {
    name.split('_')
        .enumerate()
        .map(|(index, word)| match (index, word.split_at_checked(1)) {
            (1.., Some((head, tail))) => head.to_uppercase() + tail,
            _ => word.to_owned(),
        })
        .collect()
}
