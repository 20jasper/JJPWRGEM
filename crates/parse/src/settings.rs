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
pub struct Setting {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
    pub short: Option<char>,
    pub aliases: &'static [&'static str],
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

// Rust-like syntax for defining Settings for conflicts, defaults, descriptions
macro_rules! settings {
    (
        struct $struct_name:ident, table $table_name:ident, id $id_name:ident {
            $(
                #[doc = $field_doc:literal]
                $variant:ident $field_name:ident: $field_type:ty =
                    $kind:ident $({ $($kind_body:tt)* })?
                    $(, short $short:literal)?
                    $(, alias $alias:literal)?,
            )*
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
                kind: $crate::settings::SettingKind::$kind $({ $($kind_body)* })?,
                short: [$(Some($short),)* None][0],
                aliases: &[$($alias),*],
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

pub(crate) use settings;
