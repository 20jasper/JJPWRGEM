use std::{process::Command, time::Duration};

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind, Protocol};
use serde_json::json;

use crate::common::*;

#[test]
fn initialize_and_shutdown_round_trip() {
    let mut client = ScriptedServer::new().respond_to_all().spawn();

    let result = client.initialize(ClientCapabilities::default()).unwrap();
    assert_eq!(result.capabilities, ServerCapabilities::default());

    client.shutdown().unwrap();

    let status = client.wait_for_exit().unwrap();
    assert!(status.success());
}

#[test]
fn exits_immediately_is_reported_as_disconnected() {
    let mut client = ScriptedServer::new().steps([step::exit()]).spawn();

    assert_err_kind!(
        client.initialize(ClientCapabilities::default()),
        ErrorKind::Disconnected { .. }
    );

    assert_err_kind!(
        client.initialize(ClientCapabilities::default()),
        ErrorKind::Disconnected { .. }
    );
}

#[test]
fn never_responds_times_out() {
    let mut client = uninitialized().with_timeout(Duration::from_millis(100));

    assert_err_kind!(
        client.initialize(ClientCapabilities::default()),
        ErrorKind::ResponseTimeout { .. }
    );
}

#[test]
fn wait_for_exit_times_out_instead_of_blocking_forever_on_a_hung_process() {
    let mut client = uninitialized().with_exit_timeout(Duration::from_millis(100));

    assert_err_kind!(client.wait_for_exit(), ErrorKind::ExitTimeout { .. });
}

#[test]
fn spawning_a_missing_program_names_it_in_the_error() {
    let err = Client::spawn(Command::new("lspresso-no-such-server")).unwrap_err();

    assert!(
        matches!(err.kind(), ErrorKind::Spawn { program, .. } if program == "lspresso-no-such-server")
    );
    assert!(err.to_string().contains("lspresso-no-such-server"));
}

#[test]
#[cfg(unix)]
fn wait_for_exit_times_out_when_stdout_closes_but_the_process_keeps_running() {
    let mut command = Command::new("sh");
    command.args(["-c", "exec >&-; sleep 30"]);
    let mut client = Client::spawn(command)
        .unwrap()
        .with_exit_timeout(Duration::from_millis(200));

    assert_err_kind!(client.wait_for_exit(), ErrorKind::ExitTimeout { .. });
}

#[test]
fn rss_is_reported_only_while_the_server_runs() {
    let mut client = initialized();
    let rss = client.rss_bytes().expect("the server is running");
    assert!(rss > 256 * 1024, "implausibly small RSS: {rss} bytes");

    client.shutdown().unwrap();
    client.wait_for_exit().unwrap();
    assert_eq!(client.rss_bytes(), None);
}

#[test]
fn the_escape_hatch_rejects_calls_before_initialize() {
    let mut client = uninitialized();

    assert_not_initialized([
        workspace_symbol(&mut client).unwrap_err(),
        client
            .notify::<DidChangeConfigurationNotification>(DidChangeConfigurationParams {
                settings: serde_json::Value::Null,
            })
            .unwrap_err(),
    ]);
}

#[test]
fn a_malformed_initialize_result_leaves_the_client_uninitialized() {
    let mut client = ScriptedServer::new()
        .steps([
            step::respond_with(json!({ "capabilities": 5 })),
            step::respond(),
            step::respond(),
        ])
        .spawn();

    assert_err_kind!(
        client.initialize(ClientCapabilities::default()),
        ErrorKind::MalformedResponse { .. }
    );
    assert!(client.server_capabilities().is_none());
    assert_not_initialized([workspace_symbol(&mut client).unwrap_err()]);

    client.initialize(ClientCapabilities::default()).unwrap();
    workspace_symbol(&mut client).unwrap();
}

#[test]
fn an_endless_timeout_does_not_overflow() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .spawn()
        .with_timeout(Duration::MAX);

    client.initialize(ClientCapabilities::default()).unwrap();
    workspace_symbol(&mut client).unwrap();
}

#[test]
fn a_server_crash_while_a_request_is_pending_is_a_disconnect() {
    let mut client = ScriptedServer::new()
        .steps([step::exit()])
        .spawn_initialized();

    assert_err_kind!(
        workspace_symbol(&mut client),
        ErrorKind::Disconnected { .. }
    );
}

#[test]
fn a_late_response_is_dropped_silently_and_later_requests_still_work() {
    let mut client = ScriptedServer::new()
        .steps([step::respond_after(300), step::respond(), step::respond()])
        .spawn_initialized()
        .with_timeout(Duration::from_millis(100));

    assert_err_kind!(
        workspace_symbol(&mut client),
        ErrorKind::ResponseTimeout { .. }
    );
    std::thread::sleep(Duration::from_millis(500));

    workspace_symbol(&mut client).unwrap();
    assert!(client.background_errors().is_empty());
}

#[test]
fn a_failed_shutdown_can_be_retried_and_ends_the_session() {
    let mut client = ScriptedServer::new()
        .steps([
            step::fail(LspErrorCodes::RequestFailed, ()),
            step::respond(),
        ])
        .spawn_initialized();

    assert_err_kind!(client.shutdown(), ErrorKind::Response { .. });

    client.shutdown().unwrap();
    assert_not_initialized([workspace_symbol(&mut client).unwrap_err()]);
    assert_err_kind!(client.shutdown(), ErrorKind::AlreadyShutdown);
}

#[test]
fn server_requests_are_answered_while_a_request_is_pending() {
    let recording = Recording::new();
    let configuration = lsp_server::Request::new(
        "from-server".to_owned().into(),
        ConfigurationRequest::METHOD.as_str().to_owned(),
        ConfigurationParams {
            items: vec![ConfigurationItem::default(); 2],
        },
    );
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .steps([step::send(configuration), step::respond()])
        .recording_to(&recording)
        .spawn_initialized();

    workspace_symbol(&mut client).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit().unwrap();

    let responses = recording.responses();
    let [response] = &responses[..] else {
        panic!("expected one response, got {responses:?}");
    };
    assert_eq!(response.id, "from-server".to_owned().into());
    assert_eq!(
        response.response_result.as_ref().ok(),
        Some(&json!([null, null])),
        "one null per item asked for"
    );
}

#[test]
fn a_response_nobody_asked_for_is_a_background_error() {
    let stray = lsp_server::Response::new_ok(999.into(), ());
    let mut client = ScriptedServer::new()
        .steps([step::send(stray), step::respond()])
        .spawn_initialized();

    workspace_symbol(&mut client).unwrap();

    let errors = client.background_errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        matches!(errors[0].kind(), ErrorKind::UnmatchedResponseId { .. }),
        "{errors:?}"
    );
}

#[test]
fn a_malformed_frame_is_reported_as_a_malformed_message() {
    let mut client = ScriptedServer::new()
        .steps([step::write_raw("Content-Length: 3\r\n\r\n{{{")])
        .spawn_initialized();

    assert_err_kind!(
        workspace_symbol(&mut client),
        ErrorKind::MalformedMessage { .. }
    );
}
