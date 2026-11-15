#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
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
}

macro_rules! settings {
    (
        $vis:vis struct $name:ident, table $table:ident {
            $(#[doc = $doc:literal] $field_vis:vis $field:ident: $ty:ty = $kind:expr),* $(,)?
        }
    ) => {
        #[derive(Debug, Default, Clone, PartialEq, Eq)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Deserialize),
            serde(rename_all = "camelCase", default)
        )]
        $vis struct $name {
            $(#[doc = $doc] $field_vis $field: $ty,)*
        }

        $vis const $table: &[$crate::format::Setting] = &[
            $($crate::format::Setting {
                name: stringify!($field),
                description: $doc.trim_ascii(),
                kind: $kind,
            },)*
        ];
    };
}

pub(super) use settings;
