use std::collections::HashMap;

use gen_lsp_types::Uri;

use crate::error::ErrorKind;

#[derive(Debug, Default)]
pub(crate) struct DocumentTracker {
    open_docs: HashMap<Uri, i32>,
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

    pub(crate) fn next_version(&self, uri: &Uri) -> crate::Result<i32> {
        self.open_docs
            .get(uri)
            .map(|version| version + 1)
            .ok_or_else(|| ErrorKind::NotOpen { uri: uri.clone() }.into())
    }

    pub(crate) fn mark_open(&mut self, uri: Uri, version: i32) {
        self.open_docs.insert(uri, version);
    }

    pub(crate) fn commit_change(&mut self, uri: &Uri, version: i32) {
        if let Some(current) = self.open_docs.get_mut(uri) {
            *current = version;
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
    fn opening_twice_without_closing_is_rejected() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), 1);

        assert!(matches!(
            tracker.require_not_open(&uri).unwrap_err().kind(),
            ErrorKind::AlreadyOpen { .. }
        ));
    }

    #[test]
    fn changing_or_closing_an_unopened_document_is_rejected() {
        let tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");

        assert!(matches!(
            tracker.require_open(&uri).unwrap_err().kind(),
            ErrorKind::NotOpen { .. }
        ));
        assert!(matches!(
            tracker.next_version(&uri).unwrap_err().kind(),
            ErrorKind::NotOpen { .. }
        ));
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
