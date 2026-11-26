//! Settings metadata to more easily generate or describe settings for integrations like editor extensions. See the xtask for an example

mod choice;
mod spec;
mod table;

pub use choice::{Choice, choice};
pub use spec::{IntegerSetting, Spec, integer};
pub use table::{SettingValue, Table};

#[doc(inline)]
pub use crate::{__choice_enum as choice_enum, __settings as settings};

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
