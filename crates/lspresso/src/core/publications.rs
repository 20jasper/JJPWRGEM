use std::collections::VecDeque;

use gen_lsp_types::{
    Diagnostic, Notification, PublishDiagnosticsNotification, PublishDiagnosticsParams, Uri,
};
use lsp_server::Notification as LspNotification;

/// Past this, the oldest publication is dropped to make room: a server that
/// publishes on every keystroke mustn't grow it forever.
const CAPACITY: usize = 4096;

#[derive(Debug, Default)]
pub(crate) struct Publications {
    queue: VecDeque<PublishDiagnosticsParams>,
}

impl Publications {
    pub(crate) fn record(&mut self, notification: &LspNotification) {
        if notification.method != PublishDiagnosticsNotification::METHOD.as_str() {
            return;
        }
        let Ok(params) =
            serde_json::from_value::<PublishDiagnosticsParams>(notification.params.clone())
        else {
            return;
        };
        if self.queue.len() == CAPACITY {
            self.queue.pop_front();
        }
        self.queue.push_back(params);
    }

    pub(crate) fn take(&mut self, uri: &Uri, current_version: i32) -> Option<Vec<Diagnostic>> {
        self.queue.retain(|p| {
            p.uri != *uri || p.version.is_none_or(|version| version >= current_version)
        });
        let index = self.queue.iter().position(|p| p.uri == *uri)?;
        self.queue.remove(index).map(|p| p.diagnostics)
    }

    pub(crate) fn clear(&mut self, uri: &Uri) {
        self.queue.retain(|p| p.uri != *uri);
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    fn diagnostic(tag: u8) -> Diagnostic {
        serde_json::from_value(serde_json::json!({
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 0 }
            },
            "message": tag.to_string()
        }))
        .unwrap()
    }

    fn tag(diagnostics: &[Diagnostic]) -> String {
        serde_json::to_value(&diagnostics[0].message)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn publish(uri: &Uri, version: Option<i32>, tag: u8) -> LspNotification {
        LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::to_value(PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics: vec![diagnostic(tag)],
                version,
            })
            .unwrap(),
        )
    }

    #[test]
    fn takes_the_oldest_publication_for_the_uri() {
        let (a, b) = (uri("file:///a.json"), uri("file:///b.json"));
        let mut publications = Publications::default();
        publications.record(&publish(&a, None, 1));
        publications.record(&publish(&b, None, 2));
        publications.record(&publish(&a, None, 3));

        assert_eq!(tag(&publications.take(&a, 1).unwrap()), "1");
        assert_eq!(tag(&publications.take(&a, 1).unwrap()), "3");
        assert_eq!(publications.take(&a, 1), None);
        assert_eq!(tag(&publications.take(&b, 1).unwrap()), "2");
    }

    #[test]
    fn discards_publications_for_an_older_version() {
        let a = uri("file:///a.json");
        let mut publications = Publications::default();
        publications.record(&publish(&a, Some(2), 1));
        publications.record(&publish(&a, Some(3), 2));
        publications.record(&publish(&a, Some(4), 3));

        assert_eq!(tag(&publications.take(&a, 3).unwrap()), "2");
        assert_eq!(tag(&publications.take(&a, 3).unwrap()), "3");
    }

    #[test]
    fn ignores_other_methods_and_malformed_publications() {
        let a = uri("file:///a.json");
        let mut publications = Publications::default();
        publications.record(&LspNotification::new(
            "window/logMessage".to_owned(),
            serde_json::json!({}),
        ));
        publications.record(&LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::json!("not an object"),
        ));
        assert_eq!(publications.take(&a, 1), None);
    }

    #[test]
    fn clear_drops_only_that_uri() {
        let (a, b) = (uri("file:///a.json"), uri("file:///b.json"));
        let mut publications = Publications::default();
        publications.record(&publish(&a, None, 1));
        publications.record(&publish(&b, None, 2));
        publications.clear(&a);
        assert_eq!(publications.take(&a, 1), None);
        assert_eq!(tag(&publications.take(&b, 1).unwrap()), "2");
    }

    #[test]
    fn a_full_queue_drops_the_oldest() {
        let a = uri("file:///a.json");
        let mut publications = Publications::default();
        for i in 0..=CAPACITY {
            publications.record(&publish(&a, Some(i32::try_from(i).unwrap()), 0));
        }
        assert_eq!(publications.queue.len(), CAPACITY);
        assert_eq!(publications.queue[0].version, Some(1));
    }

    #[derive(Debug, Clone)]
    enum Op {
        Publish { doc: usize, version: Option<i32> },
        Take { doc: usize, current: i32 },
        Clear { doc: usize },
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        let doc = 0..3usize;
        prop::collection::vec(
            prop_oneof![
                (doc.clone(), prop::option::of(0..5i32))
                    .prop_map(|(doc, version)| Op::Publish { doc, version }),
                (doc.clone(), 0..5i32).prop_map(|(doc, current)| Op::Take { doc, current }),
                doc.prop_map(|doc| Op::Clear { doc }),
            ],
            0..40,
        )
    }

    proptest! {
        #[test]
        fn matches_a_model(ops in ops()) {
            let docs = ["file:///0.json", "file:///1.json", "file:///2.json"].map(uri);
            let mut publications = Publications::default();
            let mut model: Vec<(usize, Option<i32>, u8)> = Vec::new();

            for (n, op) in ops.into_iter().enumerate() {
                let n = u8::try_from(n).unwrap();
                match op {
                    Op::Publish { doc, version } => {
                        publications.record(&publish(&docs[doc], version, n));
                        model.push((doc, version, n));
                    }
                    Op::Take { doc, current } => {
                        model.retain(|(d, v, _)| *d != doc || v.is_none_or(|v| v >= current));
                        let expected = model
                            .iter()
                            .position(|(d, ..)| *d == doc)
                            .map(|i| model.remove(i).2.to_string());
                        let taken = publications.take(&docs[doc], current);
                        prop_assert_eq!(taken.map(|d| tag(&d)), expected);
                    }
                    Op::Clear { doc } => {
                        publications.clear(&docs[doc]);
                        model.retain(|(d, ..)| *d != doc);
                    }
                }
            }
        }
    }
}
