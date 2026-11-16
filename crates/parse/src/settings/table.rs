use super::{Setting, SettingKind};

pub trait Table {
    const TABLE: &'static [Setting];
}

pub trait SettingValue {
    fn text(&self, setting: &Setting) -> String;
}

impl SettingValue for Option<usize> {
    fn text(&self, setting: &Setting) -> String {
        match (self, setting.kind) {
            (Some(value), _) => value.to_string(),
            (None, SettingKind::Integer { default }) => default.to_string(),
            (None, SettingKind::Choice { .. }) => String::new(),
        }
    }
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
                ..$crate::settings::Spec::<$field_type>::row($spec)
            },)*
        ];

        impl $crate::settings::Table for $struct_name {
            const TABLE: &'static [$crate::settings::Setting] = $table_name;
        }

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

            pub fn reset(&mut self, id: $id_name) {
                match id {
                    $($id_name::$variant => self.$field_name = <$field_type as Default>::default(),)*
                }
            }

            pub fn value_text(&self, id: $id_name) -> String {
                match id {
                    $($id_name::$variant => $crate::settings::SettingValue::text(
                        &self.$field_name,
                        id.setting(),
                    ),)*
                }
            }
        }
    };
}

pub(crate) use settings;
