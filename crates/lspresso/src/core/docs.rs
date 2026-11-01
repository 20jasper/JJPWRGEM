use std::collections::HashMap;

use gen_lsp_types::Uri;

use crate::error::ErrorKind;

/// Each open document's version and text, as the server was last told.
///
/// `next_version`/`mark_open`/`commit_change` are split into a peek step
/// (no mutation) and a commit step (mutation), so a failed send between the
/// two leaves the tracker exactly where it was -- never ahead of what the
/// server actually received.
#[derive(Debug, Default)]
pub(crate) struct DocumentTracker {
    open_docs: HashMap<Uri, Document>,
}

#[derive(Debug)]
struct Document {
    version: i32,
    text: String,
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

    /// The version the server was last told `uri` is at.
    pub(crate) fn current_version(&self, uri: &Uri) -> crate::Result<i32> {
        self.document(uri).map(|document| document.version)
    }

    pub(crate) fn text(&self, uri: &Uri) -> crate::Result<&str> {
        self.document(uri).map(|document| document.text.as_str())
    }

    fn document(&self, uri: &Uri) -> crate::Result<&Document> {
        self.open_docs
            .get(uri)
            .ok_or_else(|| ErrorKind::NotOpen { uri: uri.clone() }.into())
    }

    /// The version a `didChange` notification for `uri` should carry if
    /// sent right now. Doesn't commit anything.
    pub(crate) fn next_version(&self, uri: &Uri) -> crate::Result<i32> {
        self.current_version(uri)?
            .checked_add(1)
            .ok_or_else(|| ErrorKind::DocumentVersionExhausted { uri: uri.clone() }.into())
    }

    pub(crate) fn mark_open(&mut self, uri: Uri, text: String) {
        self.open_docs.insert(uri, Document { version: 1, text });
    }

    pub(crate) fn commit_change(&mut self, uri: &Uri, version: i32, text: String) {
        if let Some(current) = self.open_docs.get_mut(uri) {
            *current = Document { version, text };
        }
    }

    pub(crate) fn mark_closed(&mut self, uri: &Uri) {
        self.open_docs.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::*, prop_assert_eq};

    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    #[test]
    fn opening_twice_without_closing_is_rejected() {
        let mut tracker = DocumentTracker::default();
        tracker.mark_open(uri("file:///a.json"), String::new());

        assert!(matches!(
            tracker
                .require_not_open(&uri("file:///a.json"))
                .unwrap_err()
                .kind(),
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
        tracker.mark_open(uri.clone(), String::new());

        let peeked = tracker.next_version(&uri).unwrap();
        assert_eq!(peeked, 2);
        // Peeking again without committing must return the same next
        // version -- a failed send between peek and commit must not have
        // advanced anything.
        assert_eq!(tracker.next_version(&uri).unwrap(), 2);

        tracker.commit_change(&uri, peeked, String::new());
        assert_eq!(tracker.next_version(&uri).unwrap(), 3);
    }

    #[test]
    fn closing_then_reopening_resets_the_version() {
        let mut tracker = DocumentTracker::default();
        let uri = uri("file:///a.json");
        tracker.mark_open(uri.clone(), String::new());
        tracker.commit_change(&uri, 5, String::new());

        tracker.mark_closed(&uri);
        assert!(tracker.require_open(&uri).is_err());

        tracker.mark_open(uri.clone(), String::new());
        assert_eq!(tracker.next_version(&uri).unwrap(), 2);
    }

    const DOCS: usize = 3;

    #[derive(Debug, Clone)]
    enum Op {
        Open(usize),
        Change(usize),
        /// Peeks the next version but never commits, like a `didChange`
        /// whose send failed.
        ChangeSendFails(usize),
        Close(usize),
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        let doc = 0..DOCS;
        prop_oneof![
            doc.clone().prop_map(Op::Open),
            doc.clone().prop_map(Op::Change),
            doc.clone().prop_map(Op::ChangeSendFails),
            doc.prop_map(Op::Close),
        ]
    }

    proptest! {
        /// Drives the tracker and a plain `HashMap` model of open versions
        /// with the same operations, across several documents, and checks
        /// they agree on open state and exact version after every step.
        #[test]
        fn tracker_matches_a_model_of_open_document_versions(
            ops in proptest::collection::vec(op_strategy(), 0..60)
        ) {
            let uris: Vec<Uri> = (0..DOCS).map(|i| uri(&format!("file:///{i}.json"))).collect();
            let mut tracker = DocumentTracker::default();
            let mut model: std::collections::HashMap<usize, (i32, String)> = std::collections::HashMap::new();

            for op in ops {
                match op {
                    Op::Open(i) => {
                        prop_assert_eq!(tracker.require_not_open(&uris[i]).is_ok(), !model.contains_key(&i));
                        if tracker.require_not_open(&uris[i]).is_ok() {
                            tracker.mark_open(uris[i].clone(), format!("{i}"));
                            model.insert(i, (1, format!("{i}")));
                        }
                    }
                    Op::Change(i) => {
                        if let Ok(version) = tracker.next_version(&uris[i]) {
                            tracker.commit_change(&uris[i], version, format!("{i}@{version}"));
                            model.insert(i, (version, format!("{i}@{version}")));
                        }
                    }
                    Op::ChangeSendFails(i) => {
                        drop(tracker.next_version(&uris[i]));
                    }
                    Op::Close(i) => {
                        if tracker.require_open(&uris[i]).is_ok() {
                            tracker.mark_closed(&uris[i]);
                            model.remove(&i);
                        }
                    }
                }

                for (i, uri) in uris.iter().enumerate() {
                    prop_assert_eq!(tracker.require_open(uri).is_ok(), model.contains_key(&i));
                    prop_assert_eq!(
                        tracker.next_version(uri).ok(),
                        model.get(&i).map(|(version, _)| version + 1),
                        "doc {} disagrees with the model", i
                    );
                    prop_assert_eq!(tracker.text(uri).ok(), model.get(&i).map(|(_, text)| text.as_str()));
                }
            }
        }
    }
}
