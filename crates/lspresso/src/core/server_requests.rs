use gen_lsp_types::{
    ApplyWorkspaceEditRequest, ApplyWorkspaceEditResult, ConfigurationParams, ConfigurationRequest,
    LspRequestMethod, Request as LspRequest, ShowDocumentRequest, ShowDocumentResult,
};
use lsp_server::{ErrorCode, Request, Response};

fn ok<R: LspRequest>(request: &Request, result: R::Result) -> Response {
    let result = serde_json::to_value(result).expect("serializing a result as JSON cannot fail");
    Response::new_ok(request.id.clone(), result)
}

fn err(request: &Request, code: ErrorCode, message: String) -> Response {
    Response::new_err(request.id.clone(), code as i32, message)
}

// always give an answer so doesn't hang
pub fn answer(request: &Request) -> Response {
    match LspRequestMethod::from(request.method.as_str()) {
        LspRequestMethod::WorkspaceConfiguration => {
            match serde_json::from_value::<ConfigurationParams>(request.params.clone()) {
                // gives `null` for right now. No config is saved
                Ok(params) => ok::<ConfigurationRequest>(
                    request,
                    vec![serde_json::Value::Null; params.items.len()],
                ),
                Err(source) => err(request, ErrorCode::InvalidParams, source.to_string()),
            }
        }
        LspRequestMethod::WorkspaceApplyEdit => ok::<ApplyWorkspaceEditRequest>(
            request,
            ApplyWorkspaceEditResult {
                applied: false,
                failure_reason: Some("lspresso does not apply workspace edits".to_owned()),
                failed_change: None,
            },
        ),
        LspRequestMethod::WindowShowDocument => {
            ok::<ShowDocumentRequest>(request, ShowDocumentResult { success: false })
        }
        // `null`. no workspace folders, and no message action chosen
        LspRequestMethod::WorkspaceWorkspaceFolders
        | LspRequestMethod::WindowShowMessageRequest => {
            Response::new_ok(request.id.clone(), serde_json::Value::Null)
        }
        LspRequestMethod::ClientRegisterCapability
        | LspRequestMethod::ClientUnregisterCapability
        | LspRequestMethod::WindowWorkDoneProgressCreate
        | LspRequestMethod::WorkspaceCodeLensRefresh
        | LspRequestMethod::WorkspaceDiagnosticRefresh
        | LspRequestMethod::WorkspaceFoldingRangeRefresh
        | LspRequestMethod::WorkspaceInlayHintRefresh
        | LspRequestMethod::WorkspaceInlineValueRefresh
        | LspRequestMethod::WorkspaceSemanticTokensRefresh
        | LspRequestMethod::WorkspaceTextDocumentContentRefresh => {
            Response::new_ok(request.id.clone(), serde_json::Value::Null)
        }
        _ => err(
            request,
            ErrorCode::MethodNotFound,
            format!("lspresso does not handle {}", request.method),
        ),
    }
}

#[cfg(test)]
mod tests {
    use lsp_server::RequestId;

    use super::*;

    fn answered(method: &str, params: serde_json::Value) -> Response {
        let response = answer(&Request::new(RequestId::from(7), method.to_owned(), params));
        assert_eq!(
            response.id,
            RequestId::from(7),
            "answers carry the request's id"
        );
        response
    }

    fn result(response: Response) -> serde_json::Value {
        response.response_result.expect("an ok response")
    }

    fn error_code(response: Response) -> i32 {
        response
            .response_result
            .expect_err("an error response")
            .code
    }

    #[test]
    fn configuration_gets_a_null_per_item() {
        let response = answered(
            "workspace/configuration",
            serde_json::json!({ "items": [{ "section": "a" }, { "section": "b" }] }),
        );
        assert_eq!(result(response), serde_json::json!([null, null]));
    }

    #[test]
    fn malformed_configuration_params_are_rejected() {
        let response = answered("workspace/configuration", serde_json::json!({ "items": 3 }));
        assert_eq!(error_code(response), ErrorCode::InvalidParams as i32);
    }

    #[test]
    fn workspace_edits_are_declined() {
        let response = answered("workspace/applyEdit", serde_json::json!({ "edit": {} }));
        let result = result(response);
        assert_eq!(result["applied"], false);
        assert!(result["failureReason"].is_string());
    }

    #[test]
    fn showing_a_document_is_declined() {
        let response = answered(
            "window/showDocument",
            serde_json::json!({ "uri": "file:///a" }),
        );
        assert_eq!(result(response), serde_json::json!({ "success": false }));
    }

    #[test]
    fn acknowledgements_answer_null() {
        for method in [
            "client/registerCapability",
            "client/unregisterCapability",
            "window/workDoneProgress/create",
            "workspace/diagnostic/refresh",
            "workspace/semanticTokens/refresh",
            "workspace/workspaceFolders",
            "window/showMessageRequest",
        ] {
            let response = answered(method, serde_json::Value::Null);
            assert_eq!(result(response), serde_json::Value::Null, "{method}");
        }
    }

    #[test]
    fn unknown_and_client_to_server_methods_are_not_found() {
        for method in ["custom/thing", "textDocument/hover"] {
            let response = answered(method, serde_json::Value::Null);
            assert_eq!(
                error_code(response),
                ErrorCode::MethodNotFound as i32,
                "{method}"
            );
        }
    }
}
