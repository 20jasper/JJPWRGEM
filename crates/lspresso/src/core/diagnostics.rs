use std::collections::HashMap;

use gen_lsp_types::{
    Diagnostic, DiagnosticServerCancellationData, DocumentDiagnosticReport,
    FullDocumentDiagnosticReport, LspErrorCodes, Uri,
};

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
    reports: HashMap<Uri, Cached>,
}

#[derive(Debug, Default)]
struct Cached {
    result_id: Option<String>,
    items: Vec<Diagnostic>,
}

impl DiagnosticCache {
    pub(crate) fn previous_result_id(&self, uri: &Uri) -> Option<String> {
        self.reports.get(uri)?.result_id.clone()
    }

    pub(crate) fn apply(&mut self, uri: &Uri, report: DocumentDiagnosticReport) -> Vec<Diagnostic> {
        let cached = self.reports.entry(uri.clone()).or_default();
        match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(full) => {
                let FullDocumentDiagnosticReport { result_id, items } =
                    full.full_document_diagnostic_report;
                *cached = Cached { result_id, items };
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(unchanged) => {
                cached.result_id = Some(unchanged.unchanged_document_diagnostic_report.result_id);
            }
        }
        cached.items.clone()
    }

    pub(crate) fn forget(&mut self, uri: &Uri) {
        self.reports.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
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
}
