# lspresso build plan

`lspresso` is being built up from the full reference implementation (branch
`lsp-bench`) in minimal, independently-reviewable slices, each squashed to one
commit and layered on top of the last on branch `lspresso-slice-1-lifecycle`
(the branch name is stale -- it now carries multiple slices; rename when
convenient). Each slice's `Client` struct holds only the fields that slice's
own methods use -- no fields added speculatively for a future slice.

Workflow per slice: implement -> add fixture binaries + integration tests in
`tests/integration/` -> add spec rules to `spec/lsp-client.md` + `lspclient[impl
...]`/`lspclient[verify ...]` tags -> `cargo build`/`test`/`clippy`/`fmt` ->
`mise run diet` -> commit. Compare against the reference (`git show
lsp-bench:crates/lspresso/src/...`) for fidelity -- match it unless there's a
concrete reason to diverge (call out the reason when it happens).

## Completed

- **Slice 0 -- shell**: crate skeleton, Cargo.toml publish metadata, cargo-rdme
  README generation, `test-binaries` feature. Commit: `74831066`.
- **Slice 1 -- lifecycle**: `spawn`/`spawn_initialized`/`initialize`/`shutdown`/
  `wait_for_exit`, message IO core (`send`/`recv_until`/`await_response`/
  `handle_background`), `Lifecycle`/`PendingRequests` wrapper structs, `trace`
  module, generic `request<R>`/`notify<N>` escape hatch, tolerance for
  server-initiated notifications/requests lspresso doesn't act on. Commit:
  `4991e8f8` (squashed across earlier WIP commits).
- **Slice 2 -- document sync (full-sync only)**: `did_open`/`did_change`/
  `did_close` (raw `gen_lsp_types` params, no convenience wrappers yet),
  `OpenDocs` wrapper enforcing open/close balance, `require_initialized`.
  Deliberately no document-text tracking, no incremental sync, no
  auto-versioning -- nothing in this slice needs to read tracked content back.
  Commit: `f0bac6d3`. Plus a real end-to-end test against `jjpwrgem-lsp` (the
  monorepo's own server, via `jjp lsp`) in the root package's own test suite
  (`tests/integration/lsp.rs`, commit `71770add`) -- not part of the lspresso
  crate itself, but validates it against a real server rather than only
  synthetic fixtures.
- **Slice 3 -- protocol macro system**: `lsp_request!`/`lsp_notify!` macros
  (`pre:`/`post:`/`hook:` arms, matching the reference exactly) centralizing
  the `require_initialized` check so future slices get it automatically
  instead of hand-adding it per method. Renamed slice 2's `did_open`/
  `did_change`/`did_close` to `protocol_did_open_notification`/
  `protocol_did_change_notification`/`protocol_did_close_notification`
  (matching the reference's raw/protocol-layer naming) and refactored them
  onto the macro -- this frees up the plain `did_open`/`did_change`/
  `did_close` names for slice 5's convenience layer, avoiding a second
  breaking rename later. Added `with_uri.rs` (`WithUri` trait, minimal --
  only the 3 params types slice 2/3 actually have, not the reference's full
  set) so `pre: require_open`/`require_not_open` generalize over any future
  params type. Also wired `protocol_set_trace_notification`,
  `protocol_exit_notification`, `protocol_shutdown_request` through the
  macro to prove it out (swapped in for the raw `notify`/`request` escape
  hatch calls two tests were using). Commit: `8fbccf99`.

## Proposed remaining slices

Later boundaries are provisional -- expect them to move as earlier slices land
and clarify what the next one actually needs. Reference file: mostly
`crates/lspresso/src/client/protocol.rs` (1109 lines in the full
implementation) plus `convenience.rs` (629 lines, ~45 thin wrapper methods),
`diagnostics.rs` (307 lines), `edits.rs` (296 lines), `with_uri.rs` (129
lines).

- **Slice 4 -- capability gating helpers**: `server_capabilities()`-based
  query methods (`supports_formatting`, `supports_pull_diagnostics`,
  `supports_code_action`, etc., see reference `capabilities.rs`). Small,
  self-contained, no wire behavior. Restores `lspclient[init-capabilities-
gate-features]`, which was removed from `spec/lsp-client.md` in slice 2
  since nothing implemented it yet.
- **Slice 5 -- incremental sync + edits**: `did_open`/`did_change_full`/
  `did_change_incremental`/`did_close` convenience wrappers (reclaiming the
  plain names slice 3 freed up, delegating to the `protocol_did_*` macro-
  generated methods) plus `apply_edits`/`TextEdit` application (reference
  `edits.rs`), negotiating full vs. incremental via
  `text_document_sync_kind()`. Needs document-text tracking
  (`DocumentTracker` growing beyond slice 2/3's bare `OpenDocs`
  `HashSet<Uri>` into an actual `HashMap<Uri, OpenDoc>` with version+text) --
  first slice that needs to read tracked content back.
- **Slice 6 -- diagnostics**: push + pull diagnostics, `wait_for_diagnostics`/
  `wait_for_diagnostics_cleared`, result-id threading
  (`resultid-pull-diagnostics-threaded`). The most complex remaining slice --
  settle/timeout/pacing logic lives here in the reference and the test file
  bulk (proptest_concurrency.rs-style) reflects that. Should also introduce
  `FakeTransportHandle` (reference `client/transport.rs`) + the `proptest`
  dev-dependency here, not earlier -- it's an in-memory `Transport` stand-in
  built specifically so property tests over message-ordering/buffering can
  run fast and deterministically without a real subprocess, and nothing
  before diagnostics/progress has buffering complex enough to need it.
  `refresh-client-recomputes`'s `verify` tag was dropped from slice 3's
  `tolerates_and_acknowledges_server_messages` test (it wasn't actually
  proven there) pending this -- the reference verifies it via
  `proptest_concurrency.rs`, not a hand-rolled fixture test; re-add the tag
  once equivalent coverage lands here.
- **Slice 7 -- language features, batch 1**: the mechanical `convenience.rs`
  sweep for simple no-special-state requests once slice 3's macro exists --
  `hover`, `completion`, `goto_definition`/`declaration`/`type_definition`/
  `implementation`, `references`, `rename`/`prepare_rename`, `document_symbol`,
  `document_highlight`, `folding_range`, `selection_range`, `signature_help`,
  `formatting`/`range_formatting`. Each is close to a one-line
  `lsp_request!` + wrapper once the macro exists.
- **Slice 8 -- language features, batch 2 (result-id / resolve pairs)**:
  `semantic_tokens_full`/`_delta`/`_range` (result-id threading, mirrors
  diagnostics' pattern), `code_action`/`code_action_resolve`, `code_lens`/
  `code_lens_resolve`, `document_link`/`_resolve`, `inlay_hint`/`_resolve` --
  grouped because each pairs a request with a `*/resolve` request sharing
  state.
- **Slice 9 -- progress & cancellation**: `wait_for_progress` (begin/report/end
  aggregation, `ProgressToken` uniqueness), `cancel_request`
  (`$/cancelRequest`, `cancel-still-responds`), `work_done_progress_cancel`.
- **Slice 10 -- workspace edits & save lifecycle**: `save_applying_will_save_edits`
  (`will_save`/`willSaveWaitUntil` before `didSave`), `workspace/applyEdit`
  handling (atomic, no-overlap, version-checked), command execution
  (`workspace/executeCommand`).
- **Slice 11 -- everything else / cleanup**: dynamic registration, file
  operations (`will*`/`did*` create/rename/delete), file watching,
  `workspace/configuration`, workspace folders, notebook documents (if in
  scope at all -- reference explicitly doesn't support them, see the
  commented-out `lsp_notify!` calls in `protocol.rs`), position encoding
  negotiation, completion item defaults, call hierarchy, moniker, linked
  editing range, document color, inline completion/value. Reassess at slice
  10 whether this is one slice or several -- likely several once the actual
  remaining surface is visible.

## Notes for whoever picks this up

- lspresso is a standalone, server-agnostic library (see project memory
  `project_lspresso_standalone`) -- never add a dependency on another
  monorepo crate to lspresso itself. The reverse direction (another crate
  depending on lspresso as a dev-dependency for its own tests, as
  `jjpwrgem`'s root package now does) is fine and encouraged.
- Client-side spec verification only: lspresso's own tests must exercise
  lspresso's behavior, not real-server/`gen_lsp_types` compliance.
- Comments: extremely sparse by design (see prior review rounds this repo has
  been through) -- only where a WHY is genuinely non-obvious. Public
  functions that can panic get a `# Panics` doc comment; everything else
  documents fallibility via `Result`/`Option` in the signature.
- When a review finding turns out to match the reference byte-for-byte, it's
  not a bug to fix -- verify against `git show lsp-bench:<path>` before
  "fixing" anything a review flags, since several false positives this way
  have already been caught (duplicate `didopen-max-one-open`/
  `didopen-close-open-balanced` spec rules, an "unused" `AlreadyShutdown`
  test path, a "too-loose" `uri-consistent` test) -- the reference itself
  has some accepted rough edges that are fine to carry forward faithfully.
  `uri-identifies-document` is the same story: `impl`-tagged only, never
  `verify`-tagged anywhere in the mature reference either -- don't add a
  `verify` tag for it unless a test genuinely asserts on the `uri` field
  itself (checked and reverted once already, see slice 3/4 history).
