use std::collections::HashMap;

use gen_lsp_types::*;

use super::diagnostics::PulledReport;
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

    pub(crate) fn apply_pulled_report(
        &mut self,
        uri: &Uri,
        report: DocumentDiagnosticReport,
    ) -> crate::Result<Vec<Diagnostic>> {
        let document = self
            .open_docs
            .get_mut(uri)
            .ok_or_else(|| ErrorKind::NotOpen { uri: uri.clone() })?;
        Ok(document.pulled.apply(report))
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

    pub(crate) fn mark_open(&mut self, uri: Uri, version: i32, text: String) {
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

    pub(crate) fn commit_change(&mut self, uri: &Uri, version: i32, text: String) {
        if let Some(document) = self.open_docs.get_mut(uri) {
            document.version = version;
            document.text = text;
        }
    }

    pub(crate) fn mark_closed(&mut self, uri: &Uri) {
        if let Some(document) = self.open_docs.remove(uri) {
            self.closed_versions.insert(uri.clone(), document.version);
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
}
