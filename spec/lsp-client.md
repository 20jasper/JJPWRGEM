# LSP Client Conformance

Client-side requirements for implementing the Language Server Protocol,
derived from the
[LSP 3.18 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/).

Scoped to what `lspresso` currently implements (client lifecycle: spawn,
`initialize`/`initialized`, `shutdown`/`exit`; document sync: `did_open`,
`did_change`, `did_close`; and the server-initiated requests/notifications
it must tolerate or acknowledge along the way). Grows incrementally
alongside the implementation.

## Request ordering

> lspclient[reqorder-response-matches-id]
> Per the [LSP 3.18 spec, Response Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#responseMessage):
>
> > A Response Message sent as a result of a request.
>
> The response carries the same `id` the request was sent with (per
> [Request Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#requestMessage):
> "Every processed request must send a response back to the sender of the
> request") -- that shared id is what ties a response to its request.

> lspclient[reqorder-notifications-no-response]
> Per the [LSP 3.18 spec, Notification Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#notificationMessage):
>
> > A notification message. A processed notification message must not send
> > a response back. They work like events.

## Initialization

> lspclient[init-first]
> Per the [LSP 3.18 spec, Initialize Request](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#initialize):
>
> > The initialize request is sent as the first request from the client to
> > the server... Until the server has responded to the `initialize` request
> > with an `InitializeResult`, the client must not send any additional
> > requests or notifications to the server... The `initialize` request may
> > only be sent once.

> lspclient[init-initialized-after-response]
> Per the [LSP 3.18 spec, Initialized Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#initialized):
>
> > The initialized notification is sent from the client to the server
> > after the client received the result of the `initialize` request, but
> > before the client is sending any other request or notification to the
> > server... The `initialized` notification may only be sent once.

## Shutdown / Exit

> lspclient[shutdown-before-exit]
> Per the [LSP 3.18 spec, Shutdown Request](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#shutdown):
>
> > Clients must not send any requests or notifications other than `exit`
> > to a server to which they have sent a shutdown request. Clients should
> > also wait with sending the `exit` notification until they have received
> > a response from the `shutdown` request.

> lspclient[shutdown-exit-terminates]
> Per the [LSP 3.18 spec, Exit Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#exit):
>
> > A notification to ask the server to exit its process.
>
> Exit causes the server to actually terminate its process -- a client
> that wants to verify this must observe the process actually exiting,
> not just that the notification was sent.

> lspclient[shutdown-exit-code-reflects-shutdown]
> Per the [LSP 3.18 spec, Exit Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#exit):
>
> > The server should exit with `success` code 0 if the shutdown request
> > has been received before; otherwise with `error` code 1.

## Document synchronization

> lspclient[sync-mandatory]
> Per the [LSP 3.18 spec, Text Document Synchronization](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_synchronization):
>
> > Client support for `textDocument/didOpen`, `textDocument/didChange` and
> > `textDocument/didClose` notifications is mandatory in the protocol and
> > clients can not opt out supporting them.
>
> So a client can't treat text sync as an optional feature gated behind a
> capability -- it must always be able to send these three.

> lspclient[didopen-max-one-open]
> Per the [LSP 3.18 spec, DidOpenTextDocument Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_didOpen):
>
> > An open notification must not be sent more than once without a
> > corresponding close notification send before. This means open and close
> > notification must be balanced and the max open count for a particular
> > textDocument is one.

> lspclient[didopen-close-open-balanced]
> Per the [LSP 3.18 spec, DidOpenTextDocument Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_didOpen):
>
> > An open notification must not be sent more than once without a
> > corresponding close notification send before. This means open and
> > close notification must be balanced and the max open count for a
> > particular textDocument is one.

> lspclient[lifecycle-didchange-only-when-open]
> Per the [LSP 3.18 spec, DidChangeTextDocument Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_didChange):
>
> > Before a client can change a text document it must claim ownership of
> > its content using the `textDocument/didOpen` notification.

## URIs

> lspclient[uri-identifies-document]
> Per the [LSP 3.18 spec, DidOpenTextDocument Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_didOpen)
> and [URI](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#uri):
>
> > The document's content is now managed by the client and the server
> > must not try to read the document's content using the document's Uri.
>
> Every text-document operation in the protocol keys off a `TextDocumentIdentifier.uri`,
> not a filesystem path.

> lspclient[uri-consistent]
> Per the [LSP 3.18 spec, URI](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#uri):
>
> > Clients and servers should be consistent with the form they use
> > themselves to ensure the other party doesn't interpret them as distinct
> > URIs... Clients and servers should not assume that each other are
> > encoding the same way.

## Error codes

> lspclient[errcode-server-not-initialized]
> Per the [LSP 3.18 spec, Response Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#responseMessage)
> (`ErrorCodes.ServerNotInitialized`):
>
> > Error code indicating that a server received a notification or request
> > before the server received the `initialize` request.

> lspclient[errcode-content-modified]
> Per the [LSP 3.18 spec, Response Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#responseMessage)
> (`ErrorCodes.ContentModified`):
>
> > The server detected that the content of a document got modified
> > outside normal conditions... If a client decides that a result is not
> > of any use anymore the client should cancel the request.

> lspclient[errcode-request-cancelled]
> Per the [LSP 3.18 spec, Response Message](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#responseMessage)
> (`ErrorCodes.RequestCancelled`):
>
> > The client has canceled a request and a server has detected the
> > cancel.

## Trace

> lspclient[trace-logtrace-tolerated]
> Per the [LSP 3.18 spec, Log Trace Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#logTrace):
>
> > A notification to log the trace of the server's execution. The
> > amount and content of these notifications depends on the current
> > `trace` configuration.
>
> `$/logTrace` is server-to-client and purely informational -- the
> client must accept it without erroring, the same way it tolerates any
> other notification it doesn't specifically act on.

## Window messages

> lspclient[window-showmessagerequest-responds]
> Per the [LSP 3.18 spec, ShowMessage Request](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#window_showMessageRequest):
>
> > The show message request is sent from a server to a client to ask
> > the client to display a particular message in the user interface...
> > the request allows to pass actions and to wait for an answer from
> > the client.
>
> Unlike `window/showMessage` (a notification), this is a _request_ --
> the client must respond with the selected `MessageActionItem` or
> `null` if none was selected; it can't just acknowledge and move on.

> lspclient[window-telemetry-tolerated]
> Per the [LSP 3.18 spec, Telemetry Notification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#telemetry_event):
>
> > The telemetry notification is sent from the server to the client to
> > ask the client to log a telemetry event. The protocol doesn't
> > specify the payload since no interpretation of the data happens in
> > the protocol.
>
> The client must tolerate an arbitrary, unspecified payload without
> erroring.

> lspclient[progress-create-acknowledged]
> Per the [LSP 3.18 spec, Create Work Done Progress](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#window_workDoneProgress_create):
>
> > The `window/workDoneProgress/create` request is sent from the server
> > to the client to ask the client to create a work done progress.
>
> This is a server-initiated _request_ (result: `void`) -- the client
> must respond to it, not just note the token and move on.

## Refresh requests

> lspclient[refresh-client-recomputes]
> Per the [LSP 3.18 spec, Semantic Tokens Refresh](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#semanticTokens_refreshRequest)
> and [Diagnostics Refresh](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#diagnostic_refresh)
> (the same pattern also appears for code lens, folding range, inlay hint,
> and inline value):
>
> > The `workspace/semanticTokens/refresh` request is sent from the server
> > to the client. Servers can use it to ask clients to refresh the
> > editors for which this server provides semantic tokens. As a result,
> > the client should ask the server to recompute the semantic tokens for
> > these editors... Note that the client still has the freedom to delay
> > the re-calculation of the semantic tokens if, for example, an editor
> > is currently not visible.
>
> > The `workspace/diagnostic/refresh` request is sent from the server to
> > the client. Servers can use it to ask clients to refresh all needed
> > document and workspace diagnostics.
>
> So a `workspace/*/refresh` request (result: `void`) is a hint to
> re-request, not a push of new data itself -- the client re-issues
> whatever request(s) it cares about at its own pace, including never, if
> the affected editor isn't currently relevant.
