// Verifies the client's full notification sequence for this flow --
// initialized, then didOpen/didChange/didClose in order with the expected
// content -- gating the shutdown response on it like the other verify-via-
// shutdown fixtures. Anything outside that exact sequence is an error, not
// silently ignored.
use std::cell::{Cell, RefCell};

use lsp_server::Message;
use lspresso::{
    gen_lsp_types::{
        DidChangeTextDocumentNotification, DidChangeTextDocumentParams,
        DidCloseTextDocumentNotification, DidOpenTextDocumentNotification,
        DidOpenTextDocumentParams, InitializedNotification, Notification as _,
        TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument,
    },
    test_server as helpers,
};

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    WaitInitialized,
    WaitOpen,
    WaitChange,
    WaitClose,
    Done,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Stage::WaitInitialized => "waiting for initialized",
            Stage::WaitOpen => "waiting for didOpen",
            Stage::WaitChange => "waiting for didChange",
            Stage::WaitClose => "waiting for didClose",
            Stage::Done => "done",
        }
    }
}

fn main() {
    let (conn, _io) = lsp_server::Connection::stdio();
    helpers::handle_initialize(&conn);

    let stage = Cell::new(Stage::WaitInitialized);
    let error: RefCell<Option<String>> = RefCell::new(None);

    helpers::run_event_loop_with_shutdown_check(
        &conn,
        |msg, _conn| {
            let Message::Notification(note) = msg else {
                return;
            };
            if error.borrow().is_some() {
                return;
            }
            let method = note.method.clone();
            let matched = match (stage.get(), method.as_str()) {
                (Stage::WaitInitialized, m) if m == InitializedNotification::METHOD.as_str() => {
                    true
                }
                (Stage::WaitOpen, m) if m == DidOpenTextDocumentNotification::METHOD.as_str() => {
                    let params: DidOpenTextDocumentParams =
                        serde_json::from_value(note.params).unwrap();
                    params.text_document.text == "v1"
                }
                (Stage::WaitChange, m)
                    if m == DidChangeTextDocumentNotification::METHOD.as_str() =>
                {
                    let params: DidChangeTextDocumentParams =
                        serde_json::from_value(note.params).unwrap();
                    matches!(
                        params.content_changes.first(),
                        Some(TextDocumentContentChangeWholeDocument(w)) if w.text == "v2"
                    )
                }
                (Stage::WaitClose, m) if m == DidCloseTextDocumentNotification::METHOD.as_str() => {
                    true
                }
                _ => false,
            };
            if !matched {
                *error.borrow_mut() = Some(format!(
                    "unexpected or malformed {method} while {}",
                    stage.get().name()
                ));
                return;
            }
            stage.set(match stage.get() {
                Stage::WaitInitialized => Stage::WaitOpen,
                Stage::WaitOpen => Stage::WaitChange,
                Stage::WaitChange => Stage::WaitClose,
                Stage::WaitClose | Stage::Done => Stage::Done,
            });
        },
        || match error.borrow_mut().take() {
            Some(e) => Err(e),
            None if stage.get() == Stage::Done => Ok(()),
            None => Err(format!("sync incomplete: still {}", stage.get().name())),
        },
    );
}
