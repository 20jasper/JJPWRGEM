use std::collections::VecDeque;

use lsp_server::Notification;

/// Past this, the oldest notification is dropped to make room: a chatty
/// server (`window/logMessage` on every keystroke) mustn't grow it forever.
const CAPACITY: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict<T> {
    Take(T),
    Keep,
    Discard,
}

#[derive(Debug, Default)]
pub(crate) struct Inbox {
    notifications: VecDeque<Notification>,
}

impl Inbox {
    pub(crate) fn push(&mut self, notification: Notification) {
        if self.notifications.len() == CAPACITY {
            self.notifications.pop_front();
        }
        self.notifications.push_back(notification);
    }

    pub(crate) fn take_first<T>(
        &mut self,
        mut judge: impl FnMut(&Notification) -> Verdict<T>,
    ) -> Option<T> {
        let mut index = 0;
        while let Some(notification) = self.notifications.get(index) {
            match judge(notification) {
                Verdict::Take(taken) => {
                    self.notifications.remove(index);
                    return Some(taken);
                }
                Verdict::Keep => index += 1,
                Verdict::Discard => {
                    self.notifications.remove(index);
                }
            }
        }
        None
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&Notification) -> bool) {
        self.notifications.retain(keep);
    }

    #[cfg(test)]
    fn methods(&self) -> Vec<&str> {
        self.notifications
            .iter()
            .map(|notification| notification.method.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn notification(method: &str) -> Notification {
        Notification::new(method.to_owned(), serde_json::Value::Null)
    }

    fn inbox(methods: &[&str]) -> Inbox {
        let mut inbox = Inbox::default();
        for method in methods {
            inbox.push(notification(method));
        }
        inbox
    }

    #[test]
    fn takes_the_oldest_match_and_keeps_the_rest_in_order() {
        let mut inbox = inbox(&["a", "b", "a", "c"]);
        let taken = inbox.take_first(|notification| {
            if notification.method == "a" {
                Verdict::Take(notification.method.clone())
            } else {
                Verdict::Keep
            }
        });
        assert_eq!(taken.as_deref(), Some("a"));
        assert_eq!(inbox.methods(), ["b", "a", "c"]);
    }

    #[test]
    fn discards_what_it_passes_over_but_not_what_follows_the_take() {
        let mut inbox = inbox(&["stale", "keep", "stale", "fresh", "stale"]);
        let taken = inbox.take_first(|notification| match notification.method.as_str() {
            "stale" => Verdict::Discard,
            "fresh" => Verdict::Take(()),
            _ => Verdict::Keep,
        });
        assert_eq!(taken, Some(()));
        assert_eq!(inbox.methods(), ["keep", "stale"]);
    }

    #[test]
    fn a_full_inbox_drops_the_oldest() {
        let mut inbox = Inbox::default();
        for i in 0..=CAPACITY {
            inbox.push(notification(&i.to_string()));
        }
        assert_eq!(inbox.methods().len(), CAPACITY);
        assert_eq!(inbox.methods()[0], "1");
    }

    #[test]
    fn retain_removes_only_what_the_predicate_rejects() {
        let mut inbox = inbox(&["a", "b", "a"]);
        inbox.retain(|notification| notification.method != "a");
        assert_eq!(inbox.methods(), ["b"]);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Take,
        Keep,
        Discard,
    }

    proptest! {
        #[test]
        fn take_first_matches_a_scan(
            kinds in prop::collection::vec(
                prop_oneof![Just(Kind::Take), Just(Kind::Keep), Just(Kind::Discard)],
                0..30,
            ),
        ) {
            let methods: Vec<String> = kinds.iter().enumerate().map(|(i, _)| i.to_string()).collect();
            let mut inbox = Inbox::default();
            for method in &methods {
                inbox.push(notification(method));
            }

            let taken = inbox.take_first(|notification| {
                let index: usize = notification.method.parse().unwrap();
                match kinds[index] {
                    Kind::Take => Verdict::Take(index),
                    Kind::Keep => Verdict::Keep,
                    Kind::Discard => Verdict::Discard,
                }
            });

            let first_take = kinds.iter().position(|kind| *kind == Kind::Take);
            prop_assert_eq!(taken, first_take);
            let judged_up_to = first_take.unwrap_or(kinds.len());
            let expected: Vec<&str> = methods
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    if *i < judged_up_to {
                        kinds[*i] != Kind::Discard
                    } else {
                        Some(*i) != first_take
                    }
                })
                .map(|(_, method)| method.as_str())
                .collect();
            prop_assert_eq!(inbox.methods(), expected);
        }
    }
}
