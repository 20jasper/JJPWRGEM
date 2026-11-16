use super::{
    SettingKind,
    spec::{Spec, blank},
};

pub trait Choice {
    const VALUES: &'static [&'static str];
    const DESCRIPTIONS: &'static [&'static str];
    const DEFAULT: &'static str;
}

pub const fn choice<T: Choice>() -> Spec<T> {
    blank(SettingKind::Choice {
        values: T::VALUES,
        descriptions: T::DESCRIPTIONS,
        default: T::DEFAULT,
    })
}

macro_rules! choice_enum {
    (
        pub enum $name:ident default $default:ident {
            $(#[doc = $variant_doc:literal] $variant:ident = $name_lit:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
        pub enum $name {
            $(
                #[cfg_attr(feature = "serde", serde(rename = $name_lit))]
                $variant,
            )+
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$default
            }
        }

        impl $name {
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name_lit,)+
                }
            }
        }

        impl $crate::settings::Choice for $name {
            const VALUES: &'static [&'static str] = &[$($name_lit),+];
            const DESCRIPTIONS: &'static [&'static str] = &[$($variant_doc.trim_ascii()),+];
            const DEFAULT: &'static str = Self::$default.name();
        }
    };
}

pub(crate) use choice_enum;
