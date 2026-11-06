use gen_lsp_types::{
    Diagnostic, DiagnosticServerCancellationData, DocumentDiagnosticReport,
    FullDocumentDiagnosticReport, LspErrorCodes,
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

/// The last pull report for one document: its `resultId` for the next
/// request, and its items for an `unchanged` answer to stand in for.
#[derive(Debug, Default)]
pub(crate) struct PulledReport {
    result_id: Option<String>,
    items: Vec<Diagnostic>,
}

impl PulledReport {
    pub(crate) fn result_id(&self) -> Option<String> {
        self.result_id.clone()
    }

    pub(crate) fn apply(&mut self, report: DocumentDiagnosticReport) -> Vec<Diagnostic> {
        match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(full) => {
                let FullDocumentDiagnosticReport { result_id, items } =
                    full.full_document_diagnostic_report;
                *self = Self { result_id, items };
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(unchanged) => {
                self.result_id = Some(unchanged.unchanged_document_diagnostic_report.result_id);
            }
        }
        self.items.clone()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

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
        let mut report = PulledReport::default();
        assert_eq!(report.result_id(), None);

        report.apply(full(Some("r1"), "boom"));
        assert_eq!(report.result_id().as_deref(), Some("r1"));

        let items = report.apply(unchanged("r2"));
        assert_eq!(messages(&items), vec!["boom".to_owned()]);
        assert_eq!(report.result_id().as_deref(), Some("r2"));
    }

    #[test]
    fn a_full_report_without_a_result_id_forgets_the_previous_one() {
        let mut report = PulledReport::default();
        report.apply(full(Some("r1"), "boom"));
        report.apply(full(None, "again"));
        assert_eq!(report.result_id(), None);
    }
}
