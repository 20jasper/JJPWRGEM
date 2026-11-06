use std::collections::HashMap;

use gen_lsp_types::{
    Diagnostic, DiagnosticServerCancellationData, DocumentDiagnosticReport,
    FullDocumentDiagnosticReport, LspErrorCodes, Notification, PublishDiagnosticsNotification,
    PublishDiagnosticsParams, Uri,
};
use lsp_server::Notification as LspNotification;

use super::inbox::Verdict;
use crate::error::ErrorKind;

pub(crate) const MAX_PULL_ATTEMPTS: usize = 3;

pub(crate) fn should_retry_pull(kind: &ErrorKind) -> bool {
    let ErrorKind::Response { code, data, .. } = kind else {
        return false;
    };
    match LspErrorCodes::from(*code) {
        LspErrorCodes::ContentModified => true,
        LspErrorCodes::ServerCancelled => data
            .clone()
            .and_then(|data| serde_json::from_value::<DiagnosticServerCancellationData>(data).ok())
            .is_some_and(|data| data.retrigger_request),
        _ => false,
    }
}

#[derive(Debug, Default)]
pub(crate) struct DiagnosticCache {
    result_ids: HashMap<Uri, String>,
    items: HashMap<Uri, Vec<Diagnostic>>,
}

impl DiagnosticCache {
    pub(crate) fn previous_result_id(&self, uri: &Uri) -> Option<String> {
        self.result_ids.get(uri).cloned()
    }

    pub(crate) fn apply(&mut self, uri: &Uri, report: DocumentDiagnosticReport) -> Vec<Diagnostic> {
        let (result_id, items) = match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(full) => {
                let FullDocumentDiagnosticReport { result_id, items } =
                    full.full_document_diagnostic_report;
                self.items.insert(uri.clone(), items.clone());
                (result_id, items)
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(unchanged) => (
                Some(unchanged.unchanged_document_diagnostic_report.result_id),
                self.items.get(uri).cloned().unwrap_or_default(),
            ),
        };
        match result_id {
            Some(result_id) => {
                self.result_ids.insert(uri.clone(), result_id);
            }
            None => {
                self.result_ids.remove(uri);
            }
        }
        items
    }

    pub(crate) fn forget(&mut self, uri: &Uri) {
        self.result_ids.remove(uri);
        self.items.remove(uri);
    }
}

pub(crate) fn publication(
    uri: &Uri,
    current_version: i32,
    notification: &LspNotification,
) -> Verdict<Vec<Diagnostic>> {
    if notification.method != PublishDiagnosticsNotification::METHOD.as_str() {
        return Verdict::Keep;
    }
    let Ok(params) =
        serde_json::from_value::<PublishDiagnosticsParams>(notification.params.clone())
    else {
        return Verdict::Keep;
    };
    if params.uri != *uri {
        return Verdict::Keep;
    }
    match params.version {
        Some(version) if version < current_version => Verdict::Discard,
        _ => Verdict::Take(params.diagnostics),
    }
}

pub(crate) fn is_publication_for(notification: &LspNotification, uri: &Uri) -> bool {
    notification.method == PublishDiagnosticsNotification::METHOD.as_str()
        && serde_json::from_value::<PublishDiagnosticsParams>(notification.params.clone())
            .is_ok_and(|params| params.uri == *uri)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    fn publish(uri: &Uri, version: Option<i32>, diagnostics: Vec<Diagnostic>) -> LspNotification {
        LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::to_value(PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics,
                version,
            })
            .unwrap(),
        )
    }

    #[test]
    fn takes_a_publication_for_the_same_uri() {
        let uri = uri("file:///a.json");
        assert_eq!(
            publication(&uri, 1, &publish(&uri, None, vec![])),
            Verdict::Take(vec![])
        );
    }

    #[test]
    fn discards_a_publication_for_an_older_version() {
        let uri = uri("file:///a.json");
        assert_eq!(
            publication(&uri, 3, &publish(&uri, Some(2), vec![])),
            Verdict::Discard
        );
        for version in [3, 4] {
            assert_eq!(
                publication(&uri, 3, &publish(&uri, Some(version), vec![])),
                Verdict::Take(vec![])
            );
        }
    }

    #[test]
    fn keeps_another_uri_s_and_other_methods_and_malformed_ones() {
        let a = uri("file:///a.json");
        let b = uri("file:///b.json");
        let malformed = LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::json!("not an object"),
        );
        let other = LspNotification::new("window/logMessage".to_owned(), serde_json::json!({}));
        for notification in [publish(&b, Some(1), vec![]), malformed, other] {
            assert_eq!(publication(&a, 3, &notification), Verdict::Keep);
        }
    }

    #[test]
    fn recognizes_publications_for_closing_a_document() {
        let a = uri("file:///a.json");
        let b = uri("file:///b.json");
        assert!(is_publication_for(&publish(&a, Some(1), vec![]), &a));
        assert!(!is_publication_for(&publish(&b, Some(1), vec![]), &a));
        assert!(!is_publication_for(
            &LspNotification::new("window/logMessage".to_owned(), serde_json::json!({})),
            &a
        ));
    }

    fn response_error(code: LspErrorCodes, data: Option<serde_json::Value>) -> ErrorKind {
        ErrorKind::Response {
            code: code.into(),
            message: String::new(),
            method: "textDocument/diagnostic",
            id: 1.into(),
            data,
        }
    }

    #[test]
    fn retries_a_pull_the_document_changed_under() {
        assert!(should_retry_pull(&response_error(
            LspErrorCodes::ContentModified,
            None
        )));
    }

    #[test]
    fn retries_a_cancelled_pull_only_when_the_server_asks() {
        let cancelled = |data| response_error(LspErrorCodes::ServerCancelled, data);
        assert!(should_retry_pull(&cancelled(Some(
            json!({ "retriggerRequest": true })
        ))));
        assert!(!should_retry_pull(&cancelled(Some(
            json!({ "retriggerRequest": false })
        ))));
        assert!(!should_retry_pull(&cancelled(Some(json!("junk")))));
        assert!(!should_retry_pull(&cancelled(None)));
    }

    #[test]
    fn does_not_retry_other_failures() {
        assert!(!should_retry_pull(&response_error(
            LspErrorCodes::RequestFailed,
            None
        )));
        assert!(!should_retry_pull(&ErrorKind::AlreadyShutdown));
    }

    fn full(result_id: Option<&str>, message: &str) -> DocumentDiagnosticReport {
        serde_json::from_value(json!({
            "kind": "full",
            "resultId": result_id,
            "items": [{
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 0 }
                },
                "message": message
            }]
        }))
        .unwrap()
    }

    fn unchanged(result_id: &str) -> DocumentDiagnosticReport {
        serde_json::from_value(json!({ "kind": "unchanged", "resultId": result_id })).unwrap()
    }

    fn messages(diagnostics: &[Diagnostic]) -> Vec<String> {
        diagnostics
            .iter()
            .map(|diagnostic| {
                serde_json::to_value(&diagnostic.message)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    #[test]
    fn an_unchanged_report_returns_what_the_last_full_one_did() {
        let a = uri("file:///a.json");
        let mut cache = DiagnosticCache::default();
        assert_eq!(cache.previous_result_id(&a), None);

        cache.apply(&a, full(Some("r1"), "boom"));
        assert_eq!(cache.previous_result_id(&a).as_deref(), Some("r1"));

        let items = cache.apply(&a, unchanged("r2"));
        assert_eq!(messages(&items), vec!["boom".to_owned()]);
        assert_eq!(cache.previous_result_id(&a).as_deref(), Some("r2"));
    }

    #[test]
    fn a_full_report_without_a_result_id_forgets_the_previous_one() {
        let a = uri("file:///a.json");
        let mut cache = DiagnosticCache::default();
        cache.apply(&a, full(Some("r1"), "boom"));
        cache.apply(&a, full(None, "again"));
        assert_eq!(cache.previous_result_id(&a), None);
    }

    #[derive(Debug, Clone)]
    enum Step {
        Full { doc: usize, id: Option<u8>, tag: u8 },
        Unchanged { doc: usize, id: u8 },
        Forget { doc: usize },
    }

    fn steps() -> impl Strategy<Value = Vec<Step>> {
        let doc = 0..3usize;
        prop::collection::vec(
            prop_oneof![
                (doc.clone(), prop::option::of(0..4u8), 0..4u8)
                    .prop_map(|(doc, id, tag)| Step::Full { doc, id, tag }),
                (doc.clone(), 0..4u8).prop_map(|(doc, id)| Step::Unchanged { doc, id }),
                doc.prop_map(|doc| Step::Forget { doc }),
            ],
            0..40,
        )
    }

    proptest! {
        #[test]
        fn the_cache_matches_a_model(steps in steps()) {
            let docs = ["file:///0.json", "file:///1.json", "file:///2.json"].map(uri);
            let mut cache = DiagnosticCache::default();
            let mut model: [(Option<String>, Vec<String>); 3] = Default::default();

            for step in steps {
                match step {
                    Step::Full { doc, id, tag } => {
                        let message = format!("full{tag}");
                        let id = id.map(|id| format!("r{id}"));
                        let items = cache.apply(&docs[doc], full(id.as_deref(), &message));
                        prop_assert_eq!(messages(&items), vec![message.clone()]);
                        model[doc] = (id, vec![message]);
                    }
                    Step::Unchanged { doc, id } => {
                        let id = format!("r{id}");
                        let items = cache.apply(&docs[doc], unchanged(&id));
                        prop_assert_eq!(messages(&items), model[doc].1.clone());
                        model[doc].0 = Some(id);
                    }
                    Step::Forget { doc } => {
                        cache.forget(&docs[doc]);
                        model[doc] = (None, Vec::new());
                    }
                }
                for (doc, (id, _)) in model.iter().enumerate() {
                    prop_assert_eq!(&cache.previous_result_id(&docs[doc]), id);
                }
            }
        }
    }
}
