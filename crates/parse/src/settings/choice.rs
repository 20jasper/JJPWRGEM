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

#[doc(hidden)]
#[macro_export]
macro_rules! __choice_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident default $default:ident {
            $(#[doc = $variant_doc:literal] $variant:ident = $name_lit:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $(#[$meta])*
        pub enum $name {
            $($variant,)+
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

        impl $crate::settings::SettingValue for $name {
            fn text(&self, _: &$crate::settings::Setting) -> String {
                self.name().to_owned()
            }
        }

        impl $crate::settings::Choice for $name {
            const VALUES: &'static [&'static str] = &[$($name_lit),+];
            const DESCRIPTIONS: &'static [&'static str] = &[$($variant_doc.trim_ascii()),+];
            const DEFAULT: &'static str = Self::$default.name();
        }
    };
}
