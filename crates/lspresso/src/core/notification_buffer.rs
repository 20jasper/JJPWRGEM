use std::collections::VecDeque;

use gen_lsp_types::Notification;
use lsp_server::Notification as LspNotification;

#[derive(Debug)]
pub(crate) struct NotificationBuffer {
    capacity: usize,
    queue: VecDeque<LspNotification>,
}

impl Default for NotificationBuffer {
    fn default() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }
}

impl NotificationBuffer {
    const DEFAULT_CAPACITY: usize = 4096;

    pub(crate) const fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.queue.len() > self.capacity {
            self.queue.pop_front();
        }
    }

    pub(crate) fn record(&mut self, notification: &LspNotification) {
        if self.queue.len() >= self.capacity {
            self.queue.pop_front();
        }
        self.queue.push_back(notification.clone());
    }

    pub(crate) fn take<N: Notification>(
        &mut self,
        mut matches: impl FnMut(&N::Params) -> bool,
    ) -> Option<N::Params> {
        let (index, params) = self.queue.iter().enumerate().find_map(|(index, n)| {
            let params = parse::<N>(n).filter(&mut matches)?;
            Some((index, params))
        })?;
        self.queue.remove(index);
        Some(params)
    }

    pub(crate) fn retain<N: Notification>(&mut self, mut keep: impl FnMut(&N::Params) -> bool) {
        self.queue
            .retain(|n| parse::<N>(n).is_none_or(|params| keep(&params)));
    }
}

fn parse<N: Notification>(notification: &LspNotification) -> Option<N::Params> {
    if notification.method != N::METHOD.as_str() {
        return None;
    }
    serde_json::from_value(notification.params.clone()).ok()
}

#[cfg(test)]
mod tests {
    use gen_lsp_types::{
        Diagnostic, LogMessageNotification, LogMessageParams, MessageType,
        PublishDiagnosticsNotification, PublishDiagnosticsParams, Uri,
    };

    use super::*;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    fn diagnostic(message: &str) -> Diagnostic {
        Diagnostic {
            message: message.into(),
            ..Default::default()
        }
    }

    fn publish(uri: &Uri, message: &str) -> LspNotification {
        LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::to_value(PublishDiagnosticsParams {
                uri: uri.clone(),
                diagnostics: vec![diagnostic(message)],
                version: None,
            })
            .unwrap(),
        )
    }

    fn log_params(message: &str) -> LogMessageParams {
        LogMessageParams {
            kind: MessageType::Info,
            message: message.into(),
        }
    }

    fn log(message: &str) -> LspNotification {
        LspNotification::new(
            LogMessageNotification::METHOD.as_str().to_owned(),
            serde_json::to_value(log_params(message)).unwrap(),
        )
    }

    #[test]
    fn takes_the_oldest_match_of_the_method() {
        let (a, b) = (uri("file:///a.json"), uri("file:///b.json"));
        let mut notifications = NotificationBuffer::default();
        notifications.record(&log("noise"));
        notifications.record(&publish(&a, "1"));
        notifications.record(&publish(&b, "2"));
        notifications.record(&publish(&a, "3"));

        let mut take_a = || {
            notifications
                .take::<PublishDiagnosticsNotification>(|p| p.uri == a)
                .map(|p| p.diagnostics)
        };
        assert_eq!(take_a(), Some(vec![diagnostic("1")]));
        assert_eq!(take_a(), Some(vec![diagnostic("3")]));
        assert_eq!(take_a(), None);

        assert_eq!(
            notifications.take::<LogMessageNotification>(|_| true),
            Some(log_params("noise"))
        );
        assert!(
            notifications
                .take::<PublishDiagnosticsNotification>(|p| p.uri == b)
                .is_some()
        );
    }

    #[test]
    fn skips_malformed_params() {
        let mut notifications = NotificationBuffer::default();
        notifications.record(&LspNotification::new(
            PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            serde_json::json!("not an object"),
        ));
        assert!(
            notifications
                .take::<PublishDiagnosticsNotification>(|_| true)
                .is_none()
        );
    }

    #[test]
    fn retain_drops_only_the_failing_notifications_of_the_method() {
        let (a, b) = (uri("file:///a.json"), uri("file:///b.json"));
        let mut notifications = NotificationBuffer::default();
        notifications.record(&publish(&a, "1"));
        notifications.record(&publish(&b, "2"));
        notifications.record(&log("kept"));
        notifications.retain::<PublishDiagnosticsNotification>(|p| p.uri != a);

        assert!(
            notifications
                .take::<PublishDiagnosticsNotification>(|p| p.uri == a)
                .is_none()
        );
        assert!(
            notifications
                .take::<PublishDiagnosticsNotification>(|p| p.uri == b)
                .is_some()
        );
        assert!(
            notifications
                .take::<LogMessageNotification>(|_| true)
                .is_some()
        );
    }

    #[test]
    fn shrinking_the_capacity_drops_the_oldest() {
        let mut notifications = NotificationBuffer::default();
        for i in 0..3 {
            notifications.record(&log(&i.to_string()));
        }
        notifications.set_capacity(1);
        assert_eq!(
            notifications.take::<LogMessageNotification>(|_| true),
            Some(log_params("2"))
        );
        assert!(
            notifications
                .take::<LogMessageNotification>(|_| true)
                .is_none()
        );
    }

    #[test]
    fn a_full_queue_drops_the_oldest() {
        let mut notifications = NotificationBuffer::with_capacity(2);
        for i in 0..3 {
            notifications.record(&log(&i.to_string()));
        }
        assert_eq!(
            notifications.take::<LogMessageNotification>(|_| true),
            Some(log_params("1"))
        );
    }
}
