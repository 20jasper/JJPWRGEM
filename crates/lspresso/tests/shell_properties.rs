//! Drives a real `Client` against `lspresso_scripted_server` with generated
//! scripts of server behavior, and checks every call against a model of
//! what it should return -- under a watchdog, so a hang fails instead of
//! wedging the test run.

use std::{
    process::Command,
    sync::mpsc,
    time::{Duration, Instant},
};

use lspresso::{Client, ErrorKind};
use proptest::prelude::*;

const REQUEST_TIMEOUT: Duration = Duration::from_millis(250);
/// How far past its own timeout a call may return before it counts as stuck.
const SLACK: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy)]
enum Action {
    Respond { delay_ms: u64 },
    Late { delay_ms: u64 },
    Drip,
    Error,
    Ignore,
    Stall,
    Garbage,
    Exit,
}

#[derive(Debug, Clone, Copy)]
struct Step {
    flood: u8,
    server_request: bool,
    stray_response: bool,
    action: Action,
}

impl Step {
    fn to_json(self) -> serde_json::Value {
        let action = match self.action {
            Action::Respond { delay_ms } => format!("respond:{delay_ms}"),
            Action::Late { delay_ms } => format!("late:{delay_ms}"),
            Action::Drip => "drip".to_owned(),
            Action::Error => "error".to_owned(),
            Action::Ignore => "ignore".to_owned(),
            Action::Stall => "stall".to_owned(),
            Action::Garbage => "garbage".to_owned(),
            Action::Exit => "exit".to_owned(),
        };
        serde_json::json!({
            "flood": self.flood,
            "server_request": self.server_request,
            "stray_response": self.stray_response,
            "action": action,
        })
    }
}

fn step_strategy() -> impl Strategy<Value = Step> {
    let action = prop_oneof![
        6 => (0u64..40).prop_map(|delay_ms| Action::Respond { delay_ms }),
        1 => (300u64..500).prop_map(|delay_ms| Action::Late { delay_ms }),
        1 => Just(Action::Drip),
        1 => Just(Action::Error),
        1 => Just(Action::Ignore),
        1 => Just(Action::Stall),
        1 => Just(Action::Garbage),
        1 => Just(Action::Exit),
    ];
    (0u8..30, any::<bool>(), any::<bool>(), action).prop_map(
        |(flood, server_request, stray_response, action)| Step {
            flood,
            server_request,
            stray_response,
            action,
        },
    )
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Outcome {
    Ok,
    ResponseError,
    Timeout,
    Malformed,
    /// The stream is gone: a disconnect, or the write failing outright.
    Dead,
    NotInitialized,
}

fn classify<T>(result: &lspresso::Result<T>) -> Outcome {
    match result {
        Ok(_) => Outcome::Ok,
        Err(err) => match err.kind() {
            ErrorKind::Response { .. } => Outcome::ResponseError,
            ErrorKind::ResponseTimeout { .. } => Outcome::Timeout,
            ErrorKind::MalformedMessage { .. } => Outcome::Malformed,
            ErrorKind::Disconnected { .. } | ErrorKind::Io(_) => Outcome::Dead,
            ErrorKind::NotInitialized => Outcome::NotInitialized,
            other => panic!("unexpected error kind: {other:?}"),
        },
    }
}

/// What the server-side state of the session means for the next request.
#[derive(Default)]
struct Model {
    garbled: bool,
    exited: bool,
    stalled: bool,
    /// Responses for ids the client never sent, that it read.
    strays: usize,
}

impl Model {
    fn request(&mut self, step: Option<Step>) -> Outcome {
        if self.exited {
            return Outcome::Dead;
        }
        if self.stalled {
            return if self.garbled {
                Outcome::Dead
            } else {
                Outcome::Timeout
            };
        }
        if self.garbled {
            // The client stopped parsing, but the server still reads and
            // acts on this request -- including exiting -- unless it first
            // waits on a reply the client will never read the request for.
            match step {
                Some(step) if step.server_request => self.stalled = true,
                Some(Step {
                    action: Action::Exit,
                    ..
                }) => self.exited = true,
                Some(Step {
                    action: Action::Stall,
                    ..
                }) => self.stalled = true,
                _ => {}
            }
            return Outcome::Dead;
        }
        // Written ahead of the action, so read before its outcome lands.
        if step.is_some_and(|step| step.stray_response) {
            self.strays += 1;
        }
        match step.map(|step| step.action) {
            Some(Action::Respond { .. }) => Outcome::Ok,
            Some(Action::Error) => Outcome::ResponseError,
            Some(Action::Late { .. } | Action::Drip | Action::Ignore) | None => Outcome::Timeout,
            Some(Action::Stall) => {
                self.stalled = true;
                Outcome::Timeout
            }
            Some(Action::Garbage) => {
                self.garbled = true;
                Outcome::Malformed
            }
            Some(Action::Exit) => {
                self.exited = true;
                Outcome::Dead
            }
        }
    }
}

fn timed<T>(label: &str, budget: Duration, call: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = call();
    assert!(
        start.elapsed() <= budget + SLACK,
        "{label} took {:?}, over its {budget:?} budget",
        start.elapsed()
    );
    out
}

/// Runs one scripted session, asserting each call matches the model.
fn session(script: &[Step], requests: usize) {
    let json = serde_json::Value::Array(script.iter().map(|step| step.to_json()).collect());
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", json.to_string());
    let mut client = Client::spawn(command).unwrap();
    client.set_request_timeout(REQUEST_TIMEOUT);
    let mut steps = script.iter().copied();
    let mut model = Model::default();

    let init = timed("initialize", REQUEST_TIMEOUT, || {
        client.initialize(gen_lsp_types::ClientCapabilities::default())
    });
    let expected = model.request(steps.next());
    assert_eq!(classify(&init), expected, "initialize: {init:?}");
    let initialized = expected == Outcome::Ok;

    let big_settings = serde_json::json!("1,".repeat(100_000));
    for i in 0..requests {
        if initialized {
            // Large enough to fill a pipe a stalled server isn't draining.
            let sent = timed("notify", Duration::ZERO, || {
                client.notify::<gen_lsp_types::DidChangeConfigurationNotification>(
                    gen_lsp_types::DidChangeConfigurationParams {
                        settings: big_settings.clone(),
                    },
                )
            });
            // Once the server is gone, the failed write may surface here.
            if !model.exited {
                sent.unwrap();
            }
        }
        let result = timed("request", REQUEST_TIMEOUT, || {
            client.request::<gen_lsp_types::WorkspaceSymbolRequest>(
                gen_lsp_types::WorkspaceSymbolParams {
                    query: String::new(),
                    ..Default::default()
                },
            )
        });
        let expected = if initialized {
            model.request(steps.next())
        } else {
            Outcome::NotInitialized
        };
        assert_eq!(classify(&result), expected, "request #{i}: {result:?}");
    }

    let shutdown = timed("shutdown", REQUEST_TIMEOUT, || client.shutdown());
    let expected = if initialized {
        model.request(steps.next())
    } else {
        Outcome::NotInitialized
    };
    assert_eq!(classify(&shutdown), expected, "shutdown: {shutdown:?}");

    let strays = client
        .background_errors()
        .iter()
        .filter(|err| matches!(err.kind(), ErrorKind::UnmatchedResponseId { .. }))
        .count();
    assert_eq!(
        strays,
        model.strays,
        "unmatched responses: {:?}",
        client.background_errors()
    );

    let should_exit = expected == Outcome::Ok || model.exited;
    let budget = if should_exit {
        Duration::from_secs(2)
    } else {
        Duration::from_millis(200)
    };
    let exit = timed("wait_for_exit", budget, || client.wait_for_exit(budget));
    if should_exit {
        assert!(exit.unwrap().success(), "server exited unsuccessfully");
    } else {
        let err = exit.unwrap_err();
        assert!(
            matches!(err.kind(), ErrorKind::ExitTimeout { .. }),
            "wait_for_exit: {err:?}"
        );
    }
}

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|cases| cases.parse().ok())
        .unwrap_or(24)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn every_call_matches_the_model_and_returns_within_its_timeout(
        script in proptest::collection::vec(step_strategy(), 0..7),
        requests in 0usize..4,
    ) {
        let (done_tx, done_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            session(&script, requests);
            done_tx.send(()).unwrap();
        });
        match done_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(()) => worker.join().unwrap(),
            // A panic drops `done_tx` early; surface the real assertion.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                std::panic::resume_unwind(worker.join().unwrap_err())
            }
            Err(mpsc::RecvTimeoutError::Timeout) => panic!("session hung"),
        }
    }
}
