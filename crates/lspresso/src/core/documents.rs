use std::collections::HashMap;

use gen_lsp_types::*;

use super::diagnostics::PulledReport;
use crate::error::ErrorKind;

#[derive(Debug, Default)]
pub(crate) struct DocumentTracker {
    open_docs: HashMap<Uri, OpenDocument>,
}

#[derive(Debug)]
struct OpenDocument {
    version: i32,
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

    pub(crate) fn next_version(&self, uri: &Uri) -> crate::Result<i32> {
        Ok(self.current_version(uri)? + 1)
    }

    pub(crate) fn mark_open(&mut self, uri: Uri, version: i32) {
        let pulled = PulledReport::default();
        self.open_docs.insert(uri, OpenDocument { version, pulled });
    }

    pub(crate) fn commit_change(&mut self, uri: &Uri, version: i32) {
        if let Some(document) = self.open_docs.get_mut(uri) {
            document.version = version;
        }
    }

    pub(crate) fn mark_closed(&mut self, uri: &Uri) {
        self.open_docs.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    #[test]
    fn next_version_continues_from_the_version_opened_with() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 7);

        assert_eq!(tracker.next_version(&uri).unwrap(), 8);
    }

    #[test]
    fn next_version_does_not_commit_until_commit_change_runs() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1);

        let peeked = tracker.next_version(&uri).unwrap();
        assert_eq!(peeked, 2);
        assert_eq!(tracker.next_version(&uri).unwrap(), 2);

        tracker.commit_change(&uri, peeked);
        assert_eq!(tracker.next_version(&uri).unwrap(), 3);
    }

    #[test]
    fn closing_then_reopening_resets_the_version() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1);
        tracker.commit_change(&uri, 5);

        tracker.mark_closed(&uri);
        assert!(tracker.require_open(&uri).is_err());

        tracker.mark_open(uri.clone(), 1);
        assert_eq!(tracker.next_version(&uri).unwrap(), 2);
    }
}
