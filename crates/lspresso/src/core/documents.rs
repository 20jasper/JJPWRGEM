use std::collections::HashMap;

use gen_lsp_types::*;
use serde::Deserialize;
use serde_json::Value;

use super::{diagnostics::PulledReport, text};
use crate::error::ErrorKind;

#[derive(Debug, Default)]
pub(crate) struct DocumentTracker {
    open_docs: HashMap<Uri, OpenDocument>,
    closed_versions: HashMap<Uri, i32>,
}

#[derive(Debug)]
struct OpenDocument {
    version: i32,
    text: String,
    pulled: PulledReport,
}

impl DocumentTracker {
    pub(crate) fn require_not_open(&self, uri: &Uri) -> crate::Result<()> {
        if self.open_docs.contains_key(uri) {
            Err(ErrorKind::AlreadyOpen { uri: uri.clone() }.into())
        } else {
            Ok(())
        }
    }

    pub(crate) fn require_open(&self, uri: &Uri) -> crate::Result<()> {
        if self.open_docs.contains_key(uri) {
            Ok(())
        } else {
            Err(ErrorKind::NotOpen { uri: uri.clone() }.into())
        }
    }

    pub(crate) fn current_version(&self, uri: &Uri) -> crate::Result<i32> {
        Ok(self.open(uri)?.version)
    }

    pub(crate) fn text(&self, uri: &Uri) -> crate::Result<&str> {
        Ok(self.open(uri)?.text.as_str())
    }

    fn open(&self, uri: &Uri) -> crate::Result<&OpenDocument> {
        self.open_docs
            .get(uri)
            .ok_or_else(|| ErrorKind::NotOpen { uri: uri.clone() }.into())
    }

    pub(crate) fn previous_result_id(&self, uri: &Uri) -> crate::Result<Option<String>> {
        Ok(self.open(uri)?.pulled.result_id())
    }

    pub(crate) fn pulled_items(&self, uri: &Uri) -> crate::Result<Vec<Diagnostic>> {
        Ok(self.open(uri)?.pulled.items())
    }

    pub(crate) fn next_version_and_text(&self, uri: &Uri) -> crate::Result<(i32, &str)> {
        let document = self.open(uri)?;
        Ok((document.version + 1, document.text.as_str()))
    }

    pub(crate) fn next_open_version(&self, uri: &Uri) -> i32 {
        self.closed_versions
            .get(uri)
            .map_or(1, |version| version + 1)
    }

    fn mark_open(&mut self, uri: Uri, version: i32, text: String) {
        self.closed_versions.remove(&uri);
        let pulled = PulledReport::default();
        self.open_docs.insert(
            uri,
            OpenDocument {
                version,
                text,
                pulled,
            },
        );
    }

    fn commit_change(&mut self, uri: &Uri, version: i32, text: String) {
        if let Some(document) = self.open_docs.get_mut(uri) {
            document.version = version;
            document.text = text;
        }
    }

    fn mark_closed(&mut self, uri: &Uri) {
        if let Some(document) = self.open_docs.remove(uri) {
            self.closed_versions.insert(uri.clone(), document.version);
        }
    }
}

/// State change to apply once the message is sent.
#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub(crate) enum Commit {
    Open {
        uri: Uri,
        version: i32,
        text: String,
    },
    Change {
        uri: Uri,
        version: i32,
        text: String,
    },
    Close {
        uri: Uri,
    },
    Nothing,
}

impl Commit {
    /// The URI whose pushed diagnostics go stale.
    pub(crate) const fn invalidates(&self) -> Option<&Uri> {
        match self {
            Self::Open { uri, .. } | Self::Change { uri, .. } | Self::Close { uri } => Some(uri),
            Self::Nothing => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scoped {
    text_document: TextDocumentIdentifier,
}

/// `None` if `params` has the wrong shape. Those go out untracked, so tests
/// can send malformed messages on purpose.
fn decode<'a, T: Deserialize<'a>>(params: &'a Value) -> Option<T> {
    T::deserialize(params).ok()
}

impl DocumentTracker {
    /// Validates an outgoing notification. Changes nothing until [`Self::commit`].
    pub(crate) fn prepare(
        &self,
        method: &str,
        params: &Value,
        encoding: &PositionEncodingKind,
    ) -> crate::Result<Commit> {
        use LspNotificationMethod as M;
        match M::from(method) {
            M::TextDocumentDidOpen => {
                let Some(DidOpenTextDocumentParams { text_document }) = decode(params) else {
                    return Ok(Commit::Nothing);
                };
                self.require_not_open(&text_document.uri)?;
                let TextDocumentItem {
                    uri, version, text, ..
                } = text_document;
                Ok(Commit::Open { uri, version, text })
            }
            M::TextDocumentDidChange => {
                let Some(DidChangeTextDocumentParams {
                    text_document,
                    content_changes,
                }) = decode(params)
                else {
                    return Ok(Commit::Nothing);
                };
                let uri = text_document.text_document_identifier.uri;
                let text = text::apply_changes(self.text(&uri)?, &content_changes, encoding)?;
                Ok(Commit::Change {
                    uri,
                    version: text_document.version,
                    text,
                })
            }
            M::TextDocumentDidClose => match decode::<Scoped>(params) {
                Some(Scoped { text_document }) => {
                    self.require_open(&text_document.uri)?;
                    Ok(Commit::Close {
                        uri: text_document.uri,
                    })
                }
                None => Ok(Commit::Nothing),
            },
            M::TextDocumentDidSave | M::TextDocumentWillSave => {
                self.require_open_scoped(params).map(|()| Commit::Nothing)
            }
            _ => Ok(Commit::Nothing),
        }
    }

    /// Validates an outgoing request. Returns the URI to pass to
    /// [`Self::record_pulled`] for pull diagnostics.
    pub(crate) fn check_request(&self, method: &str, params: &Value) -> crate::Result<Option<Uri>> {
        match LspRequestMethod::from(method) {
            LspRequestMethod::TextDocumentWillSaveWaitUntil => {
                self.require_open_scoped(params).map(|()| None)
            }
            LspRequestMethod::TextDocumentDiagnostic => {
                Ok(decode::<Scoped>(params).map(|scoped| scoped.text_document.uri))
            }
            _ => Ok(None),
        }
    }

    /// Stores a pull response so an "unchanged" report can reuse the last items.
    pub(crate) fn record_pulled(&mut self, uri: &Uri, result: &Value) {
        let Some(report) = decode::<DocumentDiagnosticReport>(result) else {
            return;
        };
        if let Some(document) = self.open_docs.get_mut(uri) {
            document.pulled.apply(report);
        }
    }

    fn require_open_scoped(&self, params: &Value) -> crate::Result<()> {
        decode::<Scoped>(params).map_or(Ok(()), |s| self.require_open(&s.text_document.uri))
    }

    pub(crate) fn commit(&mut self, commit: Commit) {
        match commit {
            Commit::Open { uri, version, text } => self.mark_open(uri, version, text),
            Commit::Change { uri, version, text } => self.commit_change(&uri, version, text),
            Commit::Close { uri } => self.mark_closed(&uri),
            Commit::Nothing => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    #[test]
    fn reopening_continues_past_the_last_version_of_the_closed_session() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1, String::new());
        tracker.commit_change(&uri, 4, String::new());
        tracker.mark_closed(&uri);

        assert_eq!(tracker.next_open_version(&uri), 5);
        assert_eq!(tracker.next_open_version(&self::uri("file:///b.json")), 1);
    }

    #[test]
    fn next_version_continues_from_the_version_opened_with() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 7, "text".to_owned());

        assert_eq!(tracker.next_version_and_text(&uri).unwrap().0, 8);
    }

    #[test]
    fn next_version_does_not_commit_until_commit_change_runs() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1, String::new());

        let peeked = tracker.next_version_and_text(&uri).unwrap().0;
        assert_eq!(peeked, 2);
        assert_eq!(tracker.next_version_and_text(&uri).unwrap(), (2, ""));

        tracker.commit_change(&uri, peeked, "changed".to_owned());
        assert_eq!(tracker.next_version_and_text(&uri).unwrap(), (3, "changed"));
        assert_eq!(tracker.text(&uri).unwrap(), "changed");
    }

    #[test]
    fn closing_then_reopening_resets_the_version_and_text() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1, "first".to_owned());
        tracker.commit_change(&uri, 5, "changed".to_owned());

        tracker.mark_closed(&uri);
        assert!(tracker.require_open(&uri).is_err());
        assert!(tracker.text(&uri).is_err());

        tracker.mark_open(uri.clone(), 1, "reopened".to_owned());
        assert_eq!(tracker.next_version_and_text(&uri).unwrap().0, 2);
        assert_eq!(tracker.text(&uri).unwrap(), "reopened");
    }

    fn open_params(uri: &str, text: &str) -> Value {
        serde_json::json!({
            "textDocument": { "uri": uri, "languageId": "json", "version": 1, "text": text }
        })
    }

    #[test]
    fn prepare_leaves_the_tracker_alone_until_commit() {
        let mut tracker = DocumentTracker::default();
        let utf16 = PositionEncodingKind::UTF16;
        let params = open_params("file:///a.json", "{}");

        let commit = tracker
            .prepare("textDocument/didOpen", &params, &utf16)
            .unwrap();
        assert_eq!(tracker.text(&uri("file:///a.json")).ok(), None);

        tracker.commit(commit);
        assert_eq!(tracker.text(&uri("file:///a.json")).unwrap(), "{}");
        assert!(matches!(
            tracker.prepare("textDocument/didOpen", &params, &utf16),
            Err(err) if matches!(err.kind(), ErrorKind::AlreadyOpen { .. })
        ));
    }

    #[test]
    fn prepare_applies_changes_and_close_forgets_the_document() {
        let mut tracker = DocumentTracker::default();
        let utf16 = PositionEncodingKind::UTF16;
        let a = uri("file:///a.json");
        tracker.mark_open(a.clone(), 1, "ab".to_owned());

        let change = serde_json::json!({
            "textDocument": { "uri": "file:///a.json", "version": 2 },
            "contentChanges": [{ "text": "xyz" }]
        });
        let commit = tracker
            .prepare("textDocument/didChange", &change, &utf16)
            .unwrap();
        tracker.commit(commit);
        assert_eq!(tracker.text(&a).unwrap(), "xyz");

        let close = serde_json::json!({ "textDocument": { "uri": "file:///a.json" } });
        let commit = tracker
            .prepare("textDocument/didClose", &close, &utf16)
            .unwrap();
        tracker.commit(commit);
        assert!(tracker.text(&a).is_err());
    }

    #[test]
    fn save_and_will_save_wait_until_need_an_open_document() {
        let tracker = DocumentTracker::default();
        let utf16 = PositionEncodingKind::UTF16;
        let scoped = serde_json::json!({ "textDocument": { "uri": "file:///a.json" } });

        assert!(
            tracker
                .prepare("textDocument/didSave", &scoped, &utf16)
                .is_err()
        );
        assert!(
            tracker
                .check_request("textDocument/willSaveWaitUntil", &scoped)
                .is_err()
        );
        assert!(
            tracker
                .check_request("textDocument/formatting", &scoped)
                .is_ok()
        );
    }

    #[test]
    fn params_of_the_wrong_shape_go_out_untracked() {
        let tracker = DocumentTracker::default();
        let junk = serde_json::json!(5);

        for method in [
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ] {
            let commit = tracker
                .prepare(method, &junk, &PositionEncodingKind::UTF16)
                .unwrap();
            assert_eq!(commit, Commit::Nothing);
        }
    }
}
