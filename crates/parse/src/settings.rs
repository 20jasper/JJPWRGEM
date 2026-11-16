//! Settings metadata to more easily generate or describe settings for integrations like editor extensions. See the xtask for an example

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    variant_size_differences,
    reason = "settings are static table rows, never moved around"
)]
pub enum SettingKind {
    Flag,
    Integer {
        default: usize,
    },
    Choice {
        values: &'static [&'static str],
        default: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Setting {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
    pub short: Option<char>,
    pub aliases: &'static [&'static str],
}

pub trait Choice {
    const VALUES: &'static [&'static str];
    const DEFAULT: &'static str;
}

const fn blank(kind: SettingKind) -> Setting {
    Setting {
        name: "",
        description: "",
        kind,
        short: None,
        aliases: &[],
    }
}

pub const fn flag() -> Setting {
    blank(SettingKind::Flag)
}

pub const fn integer(default: usize) -> Setting {
    blank(SettingKind::Integer { default })
}

pub const fn choice<T: Choice>() -> Setting {
    blank(SettingKind::Choice {
        values: T::VALUES,
        default: T::DEFAULT,
    })
}

impl Setting {
    pub const fn short(self, short: char) -> Self {
        Self {
            short: Some(short),
            ..self
        }
    }

    pub const fn aliases(self, aliases: &'static [&'static str]) -> Self {
        Self { aliases, ..self }
    }

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

// Rust-like syntax for defining Settings for conflicts, defaults, descriptions
macro_rules! settings {
    (
        struct $struct_name:ident, table $table_name:ident, id $id_name:ident {
            $(
                #[doc = $field_doc:literal]
                $variant:ident $field_name:ident: $field_type:ty = $spec:expr
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Default, Clone, PartialEq, Eq)]
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Deserialize),
            serde(rename_all = "camelCase", default)
        )]
        pub struct $struct_name {
            $(#[doc = $field_doc] pub $field_name: $field_type,)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $id_name {
            $($variant,)*
        }

        pub const $table_name: &[$crate::settings::Setting] = &[
            $($crate::settings::Setting {
                name: stringify!($field_name),
                description: $field_doc.trim_ascii(),
                ..$spec
            },)*
        ];

        impl $id_name {
            pub const ALL: &[Self] = &[$(Self::$variant),*];

            pub fn setting(self) -> &'static $crate::settings::Setting {
                &$table_name[self as usize]
            }
        }

        impl $struct_name {
            pub fn is_set(&self, id: $id_name) -> bool {
                match id {
                    $($id_name::$variant => self.$field_name != <$field_type as Default>::default(),)*
                }
            }
        }
    };
}

mod choice_macro {
    macro_rules! choice {
        (
            pub enum $name:ident default $default:ident {
                $($variant:ident = $name_lit:literal),+ $(,)?
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
                const DEFAULT: &'static str = Self::$default.name();
            }
        };
    }

    pub(crate) use choice;
}

pub(crate) use choice_macro::choice;
pub(crate) use settings;
