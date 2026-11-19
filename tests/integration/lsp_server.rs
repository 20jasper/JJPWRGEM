use lspresso::{ErrorKind, Protocol, capabilities, gen_lsp_types::*};

use crate::lsp_e2e::{TestResult, client};

const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;

macro_rules! untyped_request {
    ($name:ident, $method:expr) => {
        struct $name;

        impl Request for $name {
            type Params = serde_json::Value;
            type Result = serde_json::Value;
            const METHOD: LspRequestMethod<'static> = $method;
            const MESSAGE_DIRECTION: MessageDirection = MessageDirection::ClientToServer;
        }
    };
}

untyped_request!(
    UnknownRequest,
    LspRequestMethod::Custom("workspace/somethingNew")
);
untyped_request!(EmptyMethodRequest, LspRequestMethod::Custom(""));
untyped_request!(
    UntypedDocumentSymbol,
    LspRequestMethod::TextDocumentDocumentSymbol
);

struct CustomNotification;

impl Notification for CustomNotification {
    type Params = serde_json::Value;
    const METHOD: LspNotificationMethod<'static> = LspNotificationMethod::Custom("$/somethingNew");
    const MESSAGE_DIRECTION: MessageDirection = MessageDirection::ClientToServer;
}

fn response_code(error: &lspresso::Error) -> Option<i32> {
    match error.kind() {
        ErrorKind::Response { code, .. } => Some(*code),
        _ => None,
    }
}

#[test]
fn exit_without_shutdown_exits_with_code_one() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    client.notify::<ExitNotification>(())?;
    assert_eq!(client.wait_for_exit()?.code(), Some(1));
    Ok(())
}

#[test]
fn an_unknown_request_gets_method_not_found_and_an_unknown_notification_is_ignored() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    client.notify::<CustomNotification>(serde_json::json!({ "a": 1 }))?;
    let error = client
        .request::<UnknownRequest>(serde_json::Value::Null)
        .unwrap_err();
    assert_eq!(response_code(&error), Some(METHOD_NOT_FOUND));
    client.shutdown()?;
    Ok(())
}

#[test]
fn a_request_with_an_empty_method_gets_method_not_found_and_the_server_keeps_working() -> TestResult
{
    let mut client = client(capabilities::vscode::latest)?;
    let error = client
        .request::<EmptyMethodRequest>(serde_json::Value::Null)
        .unwrap_err();
    assert_eq!(response_code(&error), Some(METHOD_NOT_FOUND));
    client.shutdown()?;
    Ok(())
}

#[test]
fn invalid_params_get_invalid_params_and_the_server_keeps_working() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let error = client
        .request::<UntypedDocumentSymbol>(serde_json::json!({}))
        .unwrap_err();
    assert_eq!(response_code(&error), Some(INVALID_PARAMS));
    client.shutdown()?;
    Ok(())
}
