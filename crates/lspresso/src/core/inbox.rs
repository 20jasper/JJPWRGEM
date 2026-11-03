//! Notifications that arrived while the client was waiting for something
//! else, kept in arrival order until a wait asks for them.

use std::collections::VecDeque;

use lsp_server::Notification;

/// Past this, the oldest notification is dropped to make room: a chatty
/// server (`window/logMessage` on every keystroke) mustn't grow it forever.
pub(crate) const CAPACITY: usize = 4096;

/// What a wait wants done with one notification.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict<T> {
    /// The one being waited for.
    Take(T),
    /// Someone else's; leave it for a later wait.
    Keep,
    /// Superseded, e.g. diagnostics for an older version: drop it.
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

    /// Judges buffered notifications oldest first, removing each discarded
    /// one, until one is taken.
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

    #[cfg(test)]
    fn methods(&self) -> Vec<&str> {
        self.notifications
            .iter()
            .map(|n| n.method.as_str())
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
        let taken = inbox.take_first(|n| {
            if n.method == "a" {
                Verdict::Take(n.method.clone())
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
        let taken = inbox.take_first(|n| match n.method.as_str() {
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

    /// Each notification is a letter; the wait takes one letter, discards
    /// another, and keeps the rest.
    #[derive(Debug, Clone)]
    enum Op {
        Push(char),
        TakeFirst { take: char, discard: char },
    }

    fn op() -> impl Strategy<Value = Op> {
        let letter = || prop::sample::select(vec!['a', 'b', 'c']);
        prop_oneof![
            3 => letter().prop_map(Op::Push),
            1 => (letter(), letter()).prop_map(|(take, discard)| Op::TakeFirst { take, discard }),
        ]
    }

    proptest! {
        #[test]
        fn matches_a_vec_model(ops in prop::collection::vec(op(), 0..64)) {
            let mut inbox = Inbox::default();
            let mut model: Vec<char> = Vec::new();
            for op in ops {
                match op {
                    Op::Push(letter) => {
                        inbox.push(notification(&letter.to_string()));
                        model.push(letter);
                    }
                    Op::TakeFirst { take, discard } => {
                        let judge = |letter: char| {
                            if letter == take {
                                Verdict::Take(letter)
                            } else if letter == discard {
                                Verdict::Discard
                            } else {
                                Verdict::Keep
                            }
                        };
                        let taken = inbox.take_first(|n| judge(n.method.chars().next().unwrap()));

                        let position = model.iter().position(|&letter| letter == take);
                        let expected = position.map(|_| take);
                        let end = position.unwrap_or(model.len());
                        let mut kept: Vec<char> = model[..end]
                            .iter()
                            .copied()
                            .filter(|&letter| letter != discard || letter == take)
                            .collect();
                        kept.extend_from_slice(&model[(end + usize::from(position.is_some())).min(model.len())..]);
                        model = kept;

                        prop_assert_eq!(taken, expected);
                    }
                }
                let methods: Vec<char> = inbox.methods().iter().map(|m| m.chars().next().unwrap()).collect();
                prop_assert_eq!(&methods, &model);
            }
        }
    }
}
