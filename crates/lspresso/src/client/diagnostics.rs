use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use gen_lsp_types::{
    Diagnostic, DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticRequest,
    PartialResultParams, PublishDiagnosticsNotification, PublishDiagnosticsParams, Request, Uri,
    WorkDoneProgressParams,
};

use super::{Client, from_value, text_document, to_value, with_uri::for_uri};
use crate::error::{Error, ErrorKind, Result, elapsed_ms};

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

impl Client {
    // lspclient[impl diag-replace-previous]
    fn diagnostic_items(&mut self, uri: Uri, report: DocumentDiagnosticReport) -> Vec<Diagnostic> {
        match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(r) => {
                let items = r.full_document_diagnostic_report.items;
                self.diagnostic_cache.insert(uri, items.clone());
                items
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(_) => {
                self.diagnostic_cache.get(&uri).cloned().unwrap_or_default()
            }
        }
    }

    fn diagnostics_once(
        &mut self,
        uri: &Uri,
        deadline: Instant,
        predicate: impl Fn(&[Diagnostic]) -> bool,
    ) -> Result<Vec<Diagnostic>> {
        if self.supports_pull_diagnostics() {
            let report = self.diagnostic_before(uri.clone(), deadline)?;
            Ok(self.diagnostic_items(uri.clone(), report))
        } else {
            Ok(self
                .wait_for_within::<PublishDiagnosticsNotification, _>(
                    remaining(deadline),
                    for_uri(uri, |p: &PublishDiagnosticsParams| {
                        predicate(&p.diagnostics)
                    }),
                )?
                .diagnostics)
        }
    }

    fn poll_pull_diagnostics<T>(
        &mut self,
        uri: &Uri,
        deadline: Instant,
        mut decide: impl FnMut(&[Diagnostic]) -> ControlFlow<T>,
    ) -> Result<Option<T>> {
        loop {
            if remaining(deadline).is_zero() {
                return Ok(None);
            }
            let items = match self.diagnostic_before(uri.clone(), deadline) {
                Ok(report) => self.diagnostic_items(uri.clone(), report),
                Err(e) if e.is_response_deadline() => return Ok(None),
                Err(e) => return Err(e),
            };
            if let ControlFlow::Break(value) = decide(&items) {
                return Ok(Some(value));
            }
            std::thread::sleep(remaining(deadline).min(self.settle));
        }
    }

    // lspclient[impl resultid-pull-diagnostics-threaded]
    fn diagnostic_before(
        &mut self,
        uri: Uri,
        deadline: Instant,
    ) -> Result<DocumentDiagnosticReport> {
        let previous_result_id = self.diagnostic_result_ids.get(&uri).cloned();
        let params = to_value(
            DocumentDiagnosticParams {
                text_document: text_document(uri.clone()),
                identifier: None,
                previous_result_id,
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            },
            DocumentDiagnosticRequest::METHOD.as_str(),
        )?;

        let raw = self.request_value_before(
            DocumentDiagnosticRequest::METHOD.as_str(),
            params,
            deadline,
        )?;
        let report = from_value::<DocumentDiagnosticReport>(
            raw,
            DocumentDiagnosticRequest::METHOD.as_str(),
        )?;
        self.track_diagnostic_result_id(uri, &report);
        Ok(report)
    }

    fn track_diagnostic_result_id(&mut self, uri: Uri, report: &DocumentDiagnosticReport) {
        let result_id = match report {
            DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(r) => {
                r.full_document_diagnostic_report.result_id.clone()
            }
            DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(r) => {
                Some(r.unchanged_document_diagnostic_report.result_id.clone())
            }
        };
        match result_id {
            Some(id) => drop(self.diagnostic_result_ids.insert(uri, id)),
            None => drop(self.diagnostic_result_ids.remove(&uri)),
        }
    }

    // lspclient[impl diag-replace-previous]
    pub fn wait_for_diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        self.require_initialized()?;
        self.require_open(uri)?;
        let start = Instant::now();
        let deadline = start + self.timeout;
        if self.supports_pull_diagnostics() {
            return self.diagnostics_once(uri, deadline, |_| true);
        }
        let mut last = self
            .diagnostics_once(uri, deadline, |_| true)
            .map_err(|e| {
                e.into_diag_timeout(uri.clone(), elapsed_ms(&start), |uri, after_ms| {
                    ErrorKind::DiagnosticsSettleTimeout { uri, after_ms }
                })
            })?;
        while let Some(params) = self.settle_once(uri, deadline)? {
            last = params.diagnostics;
        }
        Ok(last)
    }

    // lspclient[impl diag-empty-clears]
    pub fn wait_for_diagnostics_cleared(&mut self, uri: &Uri) -> Result<()> {
        self.require_initialized()?;
        self.require_open(uri)?;
        let start = Instant::now();
        let deadline = start + self.timeout;
        if self.supports_pull_diagnostics() {
            let cleared = self.poll_pull_diagnostics(uri, deadline, |items| {
                if items.is_empty() {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            })?;
            return cleared.ok_or_else(|| Error::diagnostics_cleared_timeout(uri.clone(), &start));
        }
        self.wait_for_cleared_once(uri, deadline, &start)?;
        self.notifications.drain_diagnostics(uri);

        let mut seen_nonempty = false;
        loop {
            match self.settle_once(uri, deadline)? {
                Some(params) if params.diagnostics.is_empty() => {
                    if seen_nonempty {
                        return Ok(());
                    }
                }
                Some(_) => seen_nonempty = true,
                None => {
                    return if seen_nonempty {
                        Err(Error::diagnostics_cleared_timeout(uri.clone(), &start))
                    } else {
                        Ok(())
                    };
                }
            }
        }
    }

    fn wait_for_cleared_once(
        &mut self,
        uri: &Uri,
        deadline: Instant,
        start: &Instant,
    ) -> Result<()> {
        self.wait_for_within::<PublishDiagnosticsNotification, _>(
            remaining(deadline),
            for_uri(uri, |p: &PublishDiagnosticsParams| p.diagnostics.is_empty()),
        )
        .map_err(|e| {
            e.into_diag_timeout(uri.clone(), elapsed_ms(start), |uri, after_ms| {
                ErrorKind::DiagnosticsClearedTimeout { uri, after_ms }
            })
        })?;
        Ok(())
    }

    fn settle_once(
        &mut self,
        uri: &Uri,
        deadline: Instant,
    ) -> Result<Option<PublishDiagnosticsParams>> {
        let rem = remaining(deadline);
        if rem.is_zero() {
            return Ok(None);
        }
        match self.wait_for_within::<PublishDiagnosticsNotification, _>(
            rem.min(self.settle),
            for_uri(uri, |_: &PublishDiagnosticsParams| true),
        ) {
            Ok(params) => Ok(Some(params)),
            Err(e) if e.is_notification_deadline() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn next_diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        self.require_initialized()?;
        self.require_open(uri)?;
        self.diagnostics_once(uri, Instant::now() + self.timeout, |_| true)
    }

    pub fn check_no_diagnostics(&mut self, uri: &Uri) -> Result<()> {
        self.require_initialized()?;
        self.require_open(uri)?;
        self.notifications.drain_diagnostics(uri);
        let deadline = Instant::now() + self.timeout;
        if self.supports_pull_diagnostics() {
            // Deliberately not poll_pull_diagnostics: that primitive swallows
            // a response-deadline timeout into "nothing observed" (correct for
            // waiting on a state transition, like wait_for_diagnostics_cleared).
            // Here a hung pull request must propagate as a real error instead
            // of reading as "0 diagnostics, check passed".
            let report = self.diagnostic_before(uri.clone(), deadline)?;
            let count = self.diagnostic_items(uri.clone(), report).len();
            return if count == 0 {
                Ok(())
            } else {
                Err(Error::unexpected_diagnostics(uri.clone(), count))
            };
        }
        match self.wait_for_within::<PublishDiagnosticsNotification, _>(
            remaining(deadline),
            for_uri(uri, |p: &PublishDiagnosticsParams| {
                !p.diagnostics.is_empty()
            }),
        ) {
            Ok(params) => Err(Error::unexpected_diagnostics(
                uri.clone(),
                params.diagnostics.len(),
            )),
            Err(e) if e.is_notification_deadline() => Ok(()),
            Err(e) => Err(e),
        }
    }
}
