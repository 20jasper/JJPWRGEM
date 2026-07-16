//! # lspresso
//!
//! language server test client
//!
//! ![an orange axolotl in a blue coffee cup falling over. coffee is spilling out and the axolotl looks distressed](./lspresso.webp)
//!
//! Under active development. So far, the client lifecycle is implemented:
//! spawning a server process, the `initialize`/`initialized` handshake,
//! `shutdown`/`exit`, RSS sampling, and wire-level tracing via `tracing`.
//! Text document sync, diagnostics, and every other language feature land
//! incrementally in follow-up changes, each checked against the [LSP client
//! conformance spec] this crate is held to.
//!
//! [Language Server Protocol]: https://microsoft.github.io/language-server-protocol/
//! [LSP client conformance spec]: https://github.com/20jasper/jjpwrgem/blob/main/spec/lsp-client.md

mod client;
mod error;

pub use client::Client;
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types::{self, LanguageKind};

/// Parses `s` as a [`gen_lsp_types::Uri`], panicking with a clear message if
/// it isn't a syntactically valid RFC 3986 URI. For literals in tests and
/// benchmarks, where a malformed URI is a bug in the caller to surface
/// immediately, not a runtime condition to propagate as a `Result`.
pub fn parse_uri(s: &str) -> gen_lsp_types::Uri {
    s.parse()
        .unwrap_or_else(|e| panic!("invalid URI {s:?}: {e}"))
}

/// Builds a `file://` [`gen_lsp_types::Uri`] for `path`, percent-encoding
/// characters (spaces, non-ASCII, etc.) a raw path string would need
/// escaped to form a valid URI -- `fluent_uri` (which `Uri` is built on)
/// has no `from_file_path` of its own, unlike `url::Url`, so lspresso
/// provides one rather than leaving every caller to hand-encode paths.
/// Handles Windows drive-letter prefixes (`C:\...` becomes
/// `file:///C:/...`) the same way as Unix paths.
///
/// # Panics
///
/// Panics if `path` isn't absolute -- a relative path has no fixed
/// `file://` form.
pub fn path_to_uri(path: &std::path::Path) -> gen_lsp_types::Uri {
    use std::path::Component;

    use fluent_uri::pct_enc::{EString, encoder::Path as PathEncoder};

    assert!(
        path.is_absolute(),
        "path_to_uri requires an absolute path, got {path:?}"
    );

    let mut encoded = String::new();
    for component in path.components() {
        let segment = match component {
            Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
            Component::RootDir => continue,
            Component::CurDir => ".".to_owned(),
            Component::ParentDir => "..".to_owned(),
            Component::Normal(s) => s.to_string_lossy().into_owned(),
        };
        let mut e = EString::<PathEncoder>::new();
        e.encode_str::<PathEncoder>(&segment);
        encoded.push('/');
        encoded.push_str(e.as_estr().as_str());
    }

    parse_uri(&format!("file://{encoded}"))
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
pub mod server_helpers;
