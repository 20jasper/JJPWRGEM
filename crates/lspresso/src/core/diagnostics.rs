use gen_lsp_types::*;

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
pub(crate) struct PulledReport {
    result_id: Option<String>,
    items: Vec<Diagnostic>,
}

impl PulledReport {
    pub(crate) fn result_id(&self) -> Option<String> {
        self.result_id.clone()
    }

    pub(crate) fn items(&self) -> Vec<Diagnostic> {
        self.items.clone()
    }

    pub(crate) fn apply(&mut self, report: DocumentDiagnosticReport) {
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
        let retrigger = |retrigger_request| {
            let data = DiagnosticServerCancellationData { retrigger_request };
            cancelled(Some(serde_json::to_value(data).unwrap()))
        };
        assert!(should_retry_pull(&retrigger(true)));
        assert!(!should_retry_pull(&retrigger(false)));
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

    fn diagnostic(message: &str) -> Diagnostic {
        Diagnostic {
            message: message.into(),
            ..Default::default()
        }
    }

    fn full(result_id: Option<&str>, message: &str) -> DocumentDiagnosticReport {
        RelatedFullDocumentDiagnosticReport {
            full_document_diagnostic_report: FullDocumentDiagnosticReport::new(
                result_id.map(str::to_owned),
                vec![diagnostic(message)],
            ),
            ..Default::default()
        }
        .into()
    }

    fn unchanged(result_id: &str) -> DocumentDiagnosticReport {
        RelatedUnchangedDocumentDiagnosticReport {
            unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                result_id: result_id.to_owned(),
            },
            ..Default::default()
        }
        .into()
    }

    #[test]
    fn an_unchanged_report_returns_what_the_last_full_one_did() {
        let mut report = PulledReport::default();
        assert_eq!(report.result_id(), None);

        report.apply(full(Some("r1"), "boom"));
        assert_eq!(report.result_id().as_deref(), Some("r1"));

        report.apply(unchanged("r2"));
        assert_eq!(report.items(), vec![diagnostic("boom")]);
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
