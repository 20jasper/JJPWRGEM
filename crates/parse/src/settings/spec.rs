use std::marker::PhantomData;

use super::{Setting, SettingKind, Shorthand};

pub trait IntegerSetting {}

impl IntegerSetting for Option<usize> {}

#[derive(Debug)]
#[must_use]
pub struct Spec<T>(Setting, PhantomData<fn() -> T>);

pub(super) const fn blank<T>(kind: SettingKind) -> Spec<T> {
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

    pub const fn shorthand(
        self,
        name: &'static str,
        short: Option<char>,
        value: &'static str,
    ) -> Self {
        Self(
            Setting {
                shorthand: Some(Shorthand { name, short, value }),
                ..self.0
            },
            PhantomData,
        )
    }
}
