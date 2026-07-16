//! # lspresso
//!
//! language server test client
//!
//! ![an orange axolotl in a blue coffee cup falling over. coffee is spilling out and the axolotl looks distressed](./lspresso.webp)

mod client;
mod error;

pub use client::Client;
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types::{self, LanguageKind};

/// # Panics
///
/// Panics if `path` isn't absolute.
pub fn path_to_uri(path: &std::path::Path) -> gen_lsp_types::Uri {
    use std::path::Component;

    use fluent_uri::pct_enc::{EString, encoder::Path as PathEncoder};

    assert!(
        path.is_absolute(),
        "path_to_uri requires an absolute path, got {path:?}"
    );

    let mut joined = String::new();
    for component in path.components() {
        let segment: std::borrow::Cow<'_, str> = match component {
            Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned().into(),
            Component::RootDir => continue,
            Component::CurDir => ".".into(),
            Component::ParentDir => "..".into(),
            Component::Normal(s) => s.to_string_lossy(),
        };
        joined.push('/');
        joined.push_str(&segment);
    }

    let mut encoded = EString::<PathEncoder>::new();
    encoded.encode_str::<PathEncoder>(&joined);
    let uri = format!("file://{}", encoded.as_estr().as_str());
    uri.parse()
        .unwrap_or_else(|e| panic!("invalid URI {uri:?}: {e}"))
}

#[cfg(test)]
mod path_to_uri_tests {
    use super::path_to_uri;

    #[test]
    #[cfg(unix)]
    fn encodes_spaces_in_unix_path() {
        let uri = path_to_uri(std::path::Path::new("/tmp/my file.json"));
        assert_eq!(uri.as_str(), "file:///tmp/my%20file.json");
    }

    #[test]
    #[cfg(unix)]
    fn preserves_plain_unix_path() {
        let uri = path_to_uri(std::path::Path::new("/tmp/foo/bar.json"));
        assert_eq!(uri.as_str(), "file:///tmp/foo/bar.json");
    }

    #[test]
    #[cfg(windows)]
    fn encodes_windows_drive_and_spaces() {
        let uri = path_to_uri(std::path::Path::new(r"C:\Users\x\my file.json"));
        assert_eq!(uri.as_str(), "file:///C:/Users/x/my%20file.json");
    }

    #[test]
    #[should_panic(expected = "requires an absolute path")]
    fn panics_on_relative_path() {
        path_to_uri(std::path::Path::new("relative/path.json"));
    }
}

#[cfg(feature = "test-binaries")]
pub mod test_server;
