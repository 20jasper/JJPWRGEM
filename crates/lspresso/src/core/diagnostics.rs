use gen_lsp_types::{
    Diagnostic, Notification as LspNotification, PublishDiagnosticsNotification,
    PublishDiagnosticsParams, Uri, WithUri,
};
use lsp_server::Notification;

use super::inbox::Verdict;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
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
}
