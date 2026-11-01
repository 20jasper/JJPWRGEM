use std::collections::HashMap;

use gen_lsp_types::{
    Diagnostic, DocumentDiagnosticReport, DocumentDiagnosticRequest,
    Notification as LspNotification, PublishDiagnosticsNotification, PublishDiagnosticsParams,
    Request as LspRequest, Uri, WithUri,
};
use lsp_server::Notification;

use super::inbox::Verdict;
use crate::error::ErrorKind;

/// Takes a `publishDiagnostics` for `uri`, discarding one the server marks
/// as for a version older than `current_version`: it's been superseded.
/// Keeps everything else, a malformed publication included.
pub(crate) fn publication(
    uri: &Uri,
    current_version: i32,
    notification: &Notification,
) -> Verdict<Vec<Diagnostic>> {
    if notification.method != PublishDiagnosticsNotification::METHOD.as_str() {
        return Verdict::Keep;
    }
    let Ok(params) =
        serde_json::from_value::<PublishDiagnosticsParams>(notification.params.clone())
    else {
        return Verdict::Keep;
    };
    if params.uri() != uri {
        return Verdict::Keep;
    }
    match params.version {
        Some(version) if version < current_version => Verdict::Discard,
        _ => Verdict::Take(params.diagnostics),
    }
}

/// The last pull report per document, so the next pull can send its
/// `previousResultId` and an "unchanged" answer can be resolved.
#[derive(Debug, Default)]
pub(crate) struct PullCache {
    reports: HashMap<Uri, (String, Vec<Diagnostic>)>,
}

impl PullCache {
    pub(crate) fn previous_result_id(&self, uri: &Uri) -> Option<String> {
        self.reports
            .get(uri)
            .map(|(result_id, _)| result_id.clone())
    }

    pub(crate) fn on_report(
        &mut self,
        uri: &Uri,
        report: DocumentDiagnosticReport,
    ) -> crate::Result<Vec<Diagnostic>> {
        match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(full) => {
                let full = full.full_document_diagnostic_report;
                match full.result_id {
                    Some(result_id) => {
                        self.reports
                            .insert(uri.clone(), (result_id, full.items.clone()));
                    }
                    None => {
                        self.reports.remove(uri);
                    }
                }
                Ok(full.items)
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(unchanged) => {
                let result_id = unchanged.unchanged_document_diagnostic_report.result_id;
                match self.reports.get(uri) {
                    Some((cached, items)) if *cached == result_id => Ok(items.clone()),
                    _ => Err(ErrorKind::MalformedResponse {
                        method: DocumentDiagnosticRequest::METHOD.as_str(),
                        message: format!(
                            "unchanged report for result id {result_id:?}, which wasn't the last one sent"
                        ),
                    }
                    .into()),
                }
            }
        }
    }

    pub(crate) fn forget(&mut self, uri: &Uri) {
        self.reports.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    fn diagnostic(message: &str) -> Diagnostic {
        serde_json::from_value(serde_json::json!({
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
            "message": message,
        }))
        .unwrap()
    }

    fn publish(uri: &Uri, version: Option<i32>, diagnostics: Vec<Diagnostic>) -> Notification {
        Notification::new(
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
        let malformed = Notification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::json!("not an object"),
        );
        let other = Notification::new("window/logMessage".to_owned(), serde_json::json!({}));
        for notification in [publish(&b, Some(1), vec![]), malformed, other] {
            assert_eq!(publication(&a, 3, &notification), Verdict::Keep);
        }
    }

    fn full(result_id: Option<&str>, items: &[Diagnostic]) -> DocumentDiagnosticReport {
        serde_json::from_value(serde_json::json!({
            "kind": "full",
            "resultId": result_id,
            "items": items,
        }))
        .unwrap()
    }

    fn unchanged(result_id: &str) -> DocumentDiagnosticReport {
        serde_json::from_value(serde_json::json!({ "kind": "unchanged", "resultId": result_id }))
            .unwrap()
    }

    #[test]
    fn an_unchanged_report_reuses_the_last_full_one() {
        let uri = uri("file:///a.json");
        let mut cache = PullCache::default();

        let items = cache
            .on_report(&uri, full(Some("1"), &[diagnostic("x")]))
            .unwrap();
        assert_eq!(items, [diagnostic("x")]);
        assert_eq!(cache.previous_result_id(&uri).as_deref(), Some("1"));

        assert_eq!(
            cache.on_report(&uri, unchanged("1")).unwrap(),
            [diagnostic("x")]
        );
    }

    #[test]
    fn an_unchanged_report_for_an_unknown_result_id_is_malformed() {
        let uri = uri("file:///a.json");
        let mut cache = PullCache::default();
        assert!(cache.on_report(&uri, unchanged("1")).is_err());

        cache.on_report(&uri, full(Some("1"), &[])).unwrap();
        assert!(cache.on_report(&uri, unchanged("2")).is_err());
    }

    #[test]
    fn a_full_report_without_a_result_id_or_forgetting_clears_the_cache() {
        let uri = uri("file:///a.json");
        let mut cache = PullCache::default();
        cache.on_report(&uri, full(Some("1"), &[])).unwrap();
        cache.on_report(&uri, full(None, &[])).unwrap();
        assert_eq!(cache.previous_result_id(&uri), None);

        cache.on_report(&uri, full(Some("2"), &[])).unwrap();
        cache.forget(&uri);
        assert_eq!(cache.previous_result_id(&uri), None);
    }
}
